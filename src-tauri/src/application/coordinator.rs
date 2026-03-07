use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

use crate::application::seed_factory::SeedFactory;
use crate::browser::commands::webview_label;
use crate::settings::domain::app_settings::AppSettings;
use crate::settings::domain::profiles::{resolve_profile, BROWSER_PROFILE_ID};
use crate::shared::error::TabbyError;
use crate::shared::events::WorkspaceChangedEvent;
use crate::terminal::service::pty_service::PtyManager;
use crate::workspace::domain::layout::presets::{tree_from_count, tree_from_preset};
use crate::workspace::domain::pane::{create_pane_id, PaneKind};
use crate::workspace::domain::requests::{NewTabRequest, SplitPaneRequest};
use crate::workspace::domain::snapshot::{PaneRuntimeStatus, WorkspaceSnapshot};
use crate::workspace::service::tab_service::TabManager;

const WORKSPACE_CHANGED_EVENT: &str = "workspace-changed";

#[derive(Debug, Clone)]
pub struct Coordinator {
    app: AppHandle,
    tab_manager: Arc<TabManager>,
    pty_manager: Arc<PtyManager>,
}

impl Coordinator {
    pub fn new(app: AppHandle, tab_manager: Arc<TabManager>, pty_manager: Arc<PtyManager>) -> Self {
        Self {
            app,
            tab_manager,
            pty_manager,
        }
    }

    fn seed_factory(&self) -> SeedFactory {
        SeedFactory::new(self.app.clone(), self.pty_manager.clone())
    }

    pub fn create_tab(
        &self,
        request: &NewTabRequest,
        settings: &AppSettings,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let preset = request.preset;
        let factory = self.seed_factory();

        let seeds = match &request.pane_configs {
            Some(configs) => {
                if configs.is_empty() || configs.len() > 9 {
                    return Err(TabbyError::Validation(format!(
                        "pane_configs length must be 1\u{2013}9, got {}",
                        configs.len(),
                    )));
                }
                factory.seeds_from_pane_configs(configs, &settings.default_custom_command)?
            }
            None => {
                let pane_count = usize::from(preset.pane_count());
                factory.seeds_uniform(request, settings, pane_count)?
            }
        };

        let pane_ids: Vec<String> = seeds.iter().map(|s| s.pane_id.clone()).collect();
        let layout = match &request.pane_configs {
            Some(_) => tree_from_count(&pane_ids),
            None => tree_from_preset(preset, &pane_ids),
        };
        let snapshot = self.tab_manager.create_tab(layout, seeds)?;
        self.emit_workspace_changed(&snapshot);

        info!(tabs = snapshot.tabs.len(), preset = ?preset, "Tab created");
        Ok(snapshot)
    }

    pub fn close_tab(&self, tab_id: &str) -> Result<WorkspaceSnapshot, TabbyError> {
        let browser_pane_ids = self.tab_manager.browser_pane_ids_for_tab(tab_id);
        let pty_session_ids = self.tab_manager.terminal_session_ids_for_tab(tab_id)?;
        let (snapshot, _all_session_ids) = self.tab_manager.close_tab(tab_id)?;
        self.pty_manager.kill_many(&pty_session_ids);
        self.close_browser_webviews_quiet(&browser_pane_ids);

        info!(
            tab_id,
            killed_sessions = pty_session_ids.len(),
            "Tab closed"
        );
        self.emit_workspace_changed(&snapshot);
        Ok(snapshot)
    }

    pub fn split_pane(
        &self,
        request: &SplitPaneRequest,
        settings: &AppSettings,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let located = self.tab_manager.locate_pane(&request.pane_id)?;
        let factory = self.seed_factory();

        let profile_id = request
            .profile_id
            .as_deref()
            .unwrap_or(&located.pane.profile_id);
        let cwd = request.cwd.as_deref().unwrap_or(&located.pane.cwd);
        let explicit_command = request
            .startup_command
            .clone()
            .or_else(|| located.pane.startup_command.clone());
        let resolved = SeedFactory::resolve_effective_profile(
            profile_id,
            explicit_command,
            &settings.default_custom_command,
        )?;

        let new_pane_id = create_pane_id();
        let is_browser = resolved.id == BROWSER_PROFILE_ID;

        let seed = if is_browser {
            factory.create_browser_seed(new_pane_id.clone(), cwd, None)
        } else {
            factory.create_terminal_seed(new_pane_id.clone(), cwd, &resolved)?
        };

        let snapshot = self
            .tab_manager
            .split_pane(&request.pane_id, request.direction, seed)?;
        self.emit_workspace_changed(&snapshot);

        info!(
            pane_id = %request.pane_id,
            new_pane_id = %new_pane_id,
            direction = ?request.direction,
            "Pane split"
        );
        Ok(snapshot)
    }

    pub fn close_pane(&self, pane_id: &str) -> Result<WorkspaceSnapshot, TabbyError> {
        let located = self.tab_manager.locate_pane(pane_id)?;
        let is_browser = located.pane.pane_kind == PaneKind::Browser;

        let (snapshot, session_id, removed_tab_id) = self.tab_manager.close_pane(pane_id)?;

        if is_browser {
            self.close_browser_webviews_quiet(&[String::from(pane_id)]);
        } else {
            self.seed_factory().kill_session_quiet(&session_id);
        }

        info!(pane_id, session_id, ?removed_tab_id, "Pane closed");
        self.emit_workspace_changed(&snapshot);
        Ok(snapshot)
    }

    pub fn restart_pane(&self, pane_id: &str) -> Result<WorkspaceSnapshot, TabbyError> {
        let located = self.tab_manager.locate_pane(pane_id)?;

        if located.pane.pane_kind == PaneKind::Browser {
            return self.tab_manager.snapshot();
        }

        let factory = self.seed_factory();
        let old_session_id = located.pane.session_id.clone();

        factory.emit_lifecycle(
            pane_id,
            Some(&old_session_id),
            PaneRuntimeStatus::Restarting,
            None,
        );

        let resolved = resolve_profile(
            &located.pane.profile_id,
            located.pane.startup_command.clone(),
        )?;

        let new_session_id = factory.spawn_pty(pane_id, &located.pane.cwd, &resolved)?;
        factory.kill_session_quiet(&old_session_id);

        let snapshot = self.tab_manager.replace_pane(
            pane_id,
            new_session_id.clone(),
            resolved,
            located.pane.cwd,
        )?;

        factory.emit_lifecycle(
            pane_id,
            Some(&new_session_id),
            PaneRuntimeStatus::Running,
            None,
        );
        self.emit_workspace_changed(&snapshot);

        info!(pane_id, "Pane restarted");
        Ok(snapshot)
    }

    pub fn update_pane_profile(
        &self,
        pane_id: &str,
        profile_id: &str,
        startup_command: Option<String>,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let located = self.tab_manager.locate_pane(pane_id)?;
        let old_session_id = located.pane.session_id.clone();
        let was_browser = located.pane.pane_kind == PaneKind::Browser;
        let switching_to_browser = profile_id == BROWSER_PROFILE_ID;
        let factory = self.seed_factory();

        let resolved = resolve_profile(profile_id, startup_command)?;

        if switching_to_browser {
            if !was_browser {
                factory.kill_session_quiet(&old_session_id);
            }
            let sentinel_id = format!("browser-{}", uuid::Uuid::new_v4());
            let snapshot = self.tab_manager.replace_pane_full(
                pane_id,
                sentinel_id,
                resolved,
                located.pane.cwd,
                PaneKind::Browser,
                None,
            )?;
            self.emit_workspace_changed(&snapshot);
            info!(pane_id, profile_id, "Pane switched to browser");
            return Ok(snapshot);
        }

        let new_session_id = factory.spawn_pty(pane_id, &located.pane.cwd, &resolved)?;
        if was_browser {
            self.close_browser_webviews_quiet(&[String::from(pane_id)]);
        }
        if !was_browser {
            factory.kill_session_quiet(&old_session_id);
        }

        let snapshot = self.tab_manager.replace_pane_full(
            pane_id,
            new_session_id.clone(),
            resolved,
            located.pane.cwd,
            PaneKind::Terminal,
            None,
        )?;

        factory.emit_lifecycle(
            pane_id,
            Some(&new_session_id),
            PaneRuntimeStatus::Running,
            None,
        );
        self.emit_workspace_changed(&snapshot);

        info!(pane_id, profile_id, "Pane profile updated");
        Ok(snapshot)
    }

    pub fn update_pane_cwd(
        &self,
        pane_id: &str,
        cwd: &str,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let next_cwd = cwd.trim().to_string();
        if next_cwd.is_empty() {
            return Err(TabbyError::Validation(String::from(
                "Working directory cannot be empty",
            )));
        }

        let located = self.tab_manager.locate_pane(pane_id)?;
        let factory = self.seed_factory();
        let old_session_id = located.pane.session_id.clone();
        let resolved = resolve_profile(
            &located.pane.profile_id,
            located.pane.startup_command.clone(),
        )?;

        let new_session_id = factory.spawn_pty(pane_id, &next_cwd, &resolved)?;
        factory.kill_session_quiet(&old_session_id);

        let snapshot =
            self.tab_manager
                .replace_pane(pane_id, new_session_id.clone(), resolved, next_cwd)?;

        factory.emit_lifecycle(
            pane_id,
            Some(&new_session_id),
            PaneRuntimeStatus::Running,
            None,
        );
        self.emit_workspace_changed(&snapshot);

        info!(pane_id, cwd, "Pane cwd updated");
        Ok(snapshot)
    }

    pub fn set_active_tab(&self, tab_id: &str) -> Result<WorkspaceSnapshot, TabbyError> {
        let snapshot = self.tab_manager.set_active_tab(tab_id)?;
        self.emit_workspace_changed(&snapshot);
        Ok(snapshot)
    }

    pub fn focus_pane(&self, tab_id: &str, pane_id: &str) -> Result<WorkspaceSnapshot, TabbyError> {
        let snapshot = self.tab_manager.focus_pane(tab_id, pane_id)?;
        self.emit_workspace_changed(&snapshot);
        Ok(snapshot)
    }

    pub fn swap_panes(
        &self,
        pane_id_a: &str,
        pane_id_b: &str,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let snapshot = self.tab_manager.swap_panes(pane_id_a, pane_id_b)?;
        self.emit_workspace_changed(&snapshot);
        Ok(snapshot)
    }

    pub fn track_pane_cwd(
        &self,
        pane_id: &str,
        cwd: &str,
        settings_manager: &crate::settings::repository::settings_repository::SettingsManager,
    ) -> Result<(), TabbyError> {
        self.tab_manager.update_tracked_cwd(pane_id, cwd)?;
        settings_manager.update_last_working_directory(cwd)?;
        Ok(())
    }

    pub fn write_pty(&self, pane_id: &str, data: &str) -> Result<(), TabbyError> {
        let located = self.tab_manager.locate_pane(pane_id)?;
        if located.pane.pane_kind == PaneKind::Browser {
            return Ok(());
        }
        self.pty_manager.write(&located.pane.session_id, data)
    }

    pub fn resize_pty(&self, pane_id: &str, cols: u16, rows: u16) -> Result<(), TabbyError> {
        let located = self.tab_manager.locate_pane(pane_id)?;
        if located.pane.pane_kind == PaneKind::Browser {
            return Ok(());
        }
        self.pty_manager
            .resize(&located.pane.session_id, cols, rows)
    }

    fn emit_workspace_changed(&self, snapshot: &WorkspaceSnapshot) {
        let event = WorkspaceChangedEvent {
            workspace: snapshot.clone(),
        };

        if let Err(error) = self.app.emit(WORKSPACE_CHANGED_EVENT, event) {
            warn!(?error, "Failed to emit workspace changed event");
        }
    }

    fn close_browser_webviews_quiet(&self, pane_ids: &[String]) {
        let windows = self.app.webview_windows();
        for pane_id in pane_ids {
            let label = webview_label(pane_id);
            let found = windows.values().find_map(|w| w.get_webview(&label));

            if let Some(wv) = found {
                if let Err(err) = wv.close() {
                    warn!(?err, label, "Failed to close browser webview on cleanup");
                }
            }
        }
    }
}
