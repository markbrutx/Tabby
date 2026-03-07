use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

use crate::commands::browser::webview_label;
use crate::domain::commands::{NewTabRequest, SplitPaneRequest};
use crate::domain::error::TabbyError;
use crate::domain::events::{PaneLifecycleEvent, WorkspaceChangedEvent, PANE_LIFECYCLE_EVENT_NAME};
use crate::domain::snapshot::{PaneRuntimeStatus, WorkspaceSnapshot};
use crate::domain::split_tree::{tree_from_count, tree_from_preset};
use crate::domain::pane::{create_pane_id, PaneKind, PaneSeed};
use crate::domain::profiles::{resolve_profile, ResolvedProfile, BROWSER_PROFILE_ID, CUSTOM_PROFILE_ID};
use crate::domain::settings::AppSettings;
use crate::managers::pty::{PtyManager, SpawnRequest};
use crate::managers::tab::TabManager;

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

    pub fn create_tab(
        &self,
        request: &NewTabRequest,
        settings: &AppSettings,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let preset = request.preset;

        let seeds = match &request.pane_configs {
            Some(configs) => {
                if configs.is_empty() || configs.len() > 9 {
                    return Err(TabbyError::Validation(format!(
                        "pane_configs length must be 1–9, got {}",
                        configs.len(),
                    )));
                }
                self.seeds_from_pane_configs(configs, &settings.default_custom_command)?
            }
            None => {
                let pane_count = usize::from(preset.pane_count());
                self.seeds_uniform(request, settings, pane_count)?
            }
        };

        let pane_ids: Vec<String> = seeds.iter().map(|s| s.pane_id.clone()).collect();
        let layout = match &request.pane_configs {
            Some(_) => tree_from_count(&pane_ids),
            None => tree_from_preset(preset, &pane_ids),
        };
        let snapshot = self.tab_manager.create_tab(layout, seeds)?;
        self.emit_workspace_changed(&snapshot);

        info!(
            tabs = snapshot.tabs.len(),
            preset = ?preset,
            "Tab created"
        );

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

        let profile_id = request
            .profile_id
            .as_deref()
            .unwrap_or(&located.pane.profile_id);
        let cwd = request.cwd.as_deref().unwrap_or(&located.pane.cwd);
        let explicit_command = request
            .startup_command
            .clone()
            .or_else(|| located.pane.startup_command.clone());
        let resolved = Self::resolve_effective_profile(
            profile_id,
            explicit_command,
            &settings.default_custom_command,
        )?;

        let new_pane_id = create_pane_id();
        let is_browser = resolved.id == BROWSER_PROFILE_ID;

        let seed = if is_browser {
            self.create_browser_seed(new_pane_id.clone(), cwd, None)
        } else {
            self.create_terminal_seed(new_pane_id.clone(), cwd, &resolved)?
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
            self.kill_session_quiet(&session_id);
        }

        info!(pane_id, session_id, ?removed_tab_id, "Pane closed");

        self.emit_workspace_changed(&snapshot);
        Ok(snapshot)
    }

    pub fn restart_pane(&self, pane_id: &str) -> Result<WorkspaceSnapshot, TabbyError> {
        let located = self.tab_manager.locate_pane(pane_id)?;

        if located.pane.pane_kind == PaneKind::Browser {
            let snapshot = self.tab_manager.snapshot()?;
            return Ok(snapshot);
        }

        let old_session_id = located.pane.session_id.clone();

        self.emit_lifecycle(
            pane_id,
            Some(&old_session_id),
            PaneRuntimeStatus::Restarting,
            None,
        );

        let resolved = resolve_profile(
            &located.pane.profile_id,
            located.pane.startup_command.clone(),
        )?;

        let new_session_id = self.spawn_pty(pane_id, &located.pane.cwd, &resolved)?;

        self.kill_session_quiet(&old_session_id);

        let snapshot = self.tab_manager.replace_pane(
            pane_id,
            new_session_id.clone(),
            resolved,
            located.pane.cwd,
        )?;

        self.emit_lifecycle(
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

        let resolved = resolve_profile(profile_id, startup_command)?;

        if switching_to_browser {
            if !was_browser {
                self.kill_session_quiet(&old_session_id);
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

        // Switching to terminal profile
        let new_session_id = self.spawn_pty(pane_id, &located.pane.cwd, &resolved)?;
        if was_browser {
            self.close_browser_webviews_quiet(&[String::from(pane_id)]);
        }
        if !was_browser {
            self.kill_session_quiet(&old_session_id);
        }

        let snapshot = self.tab_manager.replace_pane_full(
            pane_id,
            new_session_id.clone(),
            resolved,
            located.pane.cwd,
            PaneKind::Terminal,
            None,
        )?;

        self.emit_lifecycle(
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
        let old_session_id = located.pane.session_id.clone();
        let resolved = resolve_profile(
            &located.pane.profile_id,
            located.pane.startup_command.clone(),
        )?;

        let new_session_id = self.spawn_pty(pane_id, &next_cwd, &resolved)?;
        self.kill_session_quiet(&old_session_id);

        let snapshot =
            self.tab_manager
                .replace_pane(pane_id, new_session_id.clone(), resolved, next_cwd)?;

        self.emit_lifecycle(
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

    pub fn focus_pane(
        &self,
        tab_id: &str,
        pane_id: &str,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
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
        settings_manager: &crate::managers::settings::SettingsManager,
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

    fn seeds_uniform(
        &self,
        request: &NewTabRequest,
        settings: &AppSettings,
        pane_count: usize,
    ) -> Result<Vec<PaneSeed>, TabbyError> {
        let profile_id = request
            .profile_id
            .as_deref()
            .unwrap_or(&settings.default_profile_id);
        let resolved = Self::resolve_effective_profile(
            profile_id,
            request.startup_command.clone(),
            &settings.default_custom_command,
        )?;
        let fallback_cwd = if settings.default_working_directory.trim().is_empty() {
            settings.last_working_directory.as_deref().unwrap_or("")
        } else {
            &settings.default_working_directory
        };
        let cwd = request
            .cwd
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or(fallback_cwd);

        let is_browser = resolved.id == BROWSER_PROFILE_ID;

        let mut seeds = Vec::with_capacity(pane_count);
        for _ in 0..pane_count {
            let pane_id = create_pane_id();
            if is_browser {
                seeds.push(self.create_browser_seed(pane_id, cwd, None));
            } else {
                seeds.push(self.create_terminal_seed(pane_id, cwd, &resolved)?);
            }
        }
        Ok(seeds)
    }

    fn seeds_from_pane_configs(
        &self,
        configs: &[crate::domain::commands::PaneConfig],
        fallback_custom_command: &str,
    ) -> Result<Vec<PaneSeed>, TabbyError> {
        let mut seeds = Vec::with_capacity(configs.len());
        for config in configs {
            let resolved = Self::resolve_effective_profile(
                &config.profile_id,
                config.startup_command.clone(),
                fallback_custom_command,
            )?;
            let cwd = if config.cwd.trim().is_empty() {
                return Err(TabbyError::Validation(String::from(
                    "Pane config cwd cannot be empty",
                )));
            } else {
                &config.cwd
            };

            let pane_id = create_pane_id();
            let is_browser = resolved.id == BROWSER_PROFILE_ID;

            if is_browser {
                seeds.push(self.create_browser_seed(pane_id, cwd, config.url.clone()));
            } else {
                seeds.push(self.create_terminal_seed(pane_id, cwd, &resolved)?);
            }
        }
        Ok(seeds)
    }

    fn resolve_effective_profile(
        profile_id: &str,
        explicit_command: Option<String>,
        fallback_custom_command: &str,
    ) -> Result<ResolvedProfile, TabbyError> {
        let startup_command = explicit_command
            .filter(|v| !v.trim().is_empty())
            .or_else(|| {
                if profile_id == CUSTOM_PROFILE_ID && !fallback_custom_command.trim().is_empty() {
                    Some(String::from(fallback_custom_command))
                } else {
                    None
                }
            });
        resolve_profile(profile_id, startup_command)
    }

    fn create_terminal_seed(
        &self,
        pane_id: String,
        cwd: &str,
        resolved: &ResolvedProfile,
    ) -> Result<PaneSeed, TabbyError> {
        let session_id = self.spawn_pty(&pane_id, cwd, resolved)?;
        self.emit_lifecycle(&pane_id, Some(&session_id), PaneRuntimeStatus::Running, None);
        Ok(PaneSeed {
            pane_id,
            session_id,
            cwd: String::from(cwd),
            profile_id: resolved.id.clone(),
            profile_label: resolved.label.clone(),
            startup_command: resolved.startup_command.clone(),
            pane_kind: PaneKind::Terminal,
            url: None,
        })
    }

    fn create_browser_seed(&self, pane_id: String, cwd: &str, url: Option<String>) -> PaneSeed {
        let sentinel_id = format!("browser-{}", uuid::Uuid::new_v4());
        PaneSeed {
            pane_id,
            session_id: sentinel_id,
            cwd: String::from(cwd),
            profile_id: String::from(BROWSER_PROFILE_ID),
            profile_label: String::from("Browser"),
            startup_command: None,
            pane_kind: PaneKind::Browser,
            url,
        }
    }

    fn spawn_pty(
        &self,
        pane_id: &str,
        cwd: &str,
        profile: &crate::domain::profiles::ResolvedProfile,
    ) -> Result<String, TabbyError> {
        self.pty_manager.spawn(SpawnRequest {
            pane_id: String::from(pane_id),
            cwd: String::from(cwd),
            profile: profile.clone(),
        })
    }

    fn kill_session_quiet(&self, session_id: &str) {
        if let Err(error) = self.pty_manager.kill(session_id) {
            warn!(?error, session_id, "Failed to kill previous PTY session");
        }
    }

    fn emit_lifecycle(
        &self,
        pane_id: &str,
        session_id: Option<&str>,
        status: PaneRuntimeStatus,
        error_message: Option<String>,
    ) {
        let event = PaneLifecycleEvent {
            pane_id: String::from(pane_id),
            session_id: session_id.map(String::from),
            status,
            error_message,
        };

        if let Err(error) = self.app.emit(PANE_LIFECYCLE_EVENT_NAME, event) {
            warn!(?error, "Failed to emit pane lifecycle event");
        }
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
