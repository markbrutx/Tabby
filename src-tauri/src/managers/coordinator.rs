use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

use crate::domain::commands::{NewTabRequest, SplitPaneRequest};
use crate::domain::error::TabbyError;
use crate::domain::events::{PaneLifecycleEvent, WorkspaceChangedEvent};
use crate::domain::snapshot::{PaneRuntimeStatus, WorkspaceSnapshot};
use crate::domain::split_tree::{tree_from_count, tree_from_preset};
use crate::domain::types::{
    create_pane_id, resolve_profile, AppSettings, PaneSeed, ResolvedProfile, CUSTOM_PROFILE_ID,
};
use crate::managers::pty::{PtyManager, SpawnRequest};
use crate::managers::tab::TabManager;

const PANE_LIFECYCLE_EVENT: &str = "pane-lifecycle";
const WORKSPACE_CHANGED_EVENT: &str = "workspace-changed";

#[derive(Debug, Clone)]
pub struct Coordinator {
    app: AppHandle,
    tab_manager: Arc<TabManager>,
    pty_manager: Arc<PtyManager>,
}

impl Coordinator {
    pub fn new(
        app: AppHandle,
        tab_manager: Arc<TabManager>,
        pty_manager: Arc<PtyManager>,
    ) -> Self {
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

    pub fn close_tab(
        &self,
        tab_id: &str,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let (snapshot, session_ids) = self.tab_manager.close_tab(tab_id)?;
        self.pty_manager.kill_many(&session_ids);

        info!(tab_id, killed_sessions = session_ids.len(), "Tab closed");

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
        let cwd = request
            .cwd
            .as_deref()
            .unwrap_or(&located.pane.cwd);
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
        let session_id = self.spawn_pty(&new_pane_id, cwd, &resolved)?;

        self.emit_lifecycle(
            &new_pane_id,
            Some(&session_id),
            PaneRuntimeStatus::Running,
            None,
        );

        let seed = PaneSeed {
            pane_id: new_pane_id.clone(),
            session_id,
            cwd: String::from(cwd),
            profile_id: resolved.id,
            profile_label: resolved.label,
            startup_command: resolved.startup_command,
        };

        let snapshot =
            self.tab_manager
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

    pub fn close_pane(
        &self,
        pane_id: &str,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let (snapshot, session_id, removed_tab_id) =
            self.tab_manager.close_pane(pane_id)?;

        self.kill_session_quiet(&session_id);

        info!(pane_id, session_id, ?removed_tab_id, "Pane closed");

        self.emit_workspace_changed(&snapshot);
        Ok(snapshot)
    }

    pub fn restart_pane(&self, pane_id: &str) -> Result<WorkspaceSnapshot, TabbyError> {
        let located = self.tab_manager.locate_pane(pane_id)?;
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

        self.emit_lifecycle(pane_id, Some(&new_session_id), PaneRuntimeStatus::Running, None);
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
        let resolved = resolve_profile(profile_id, startup_command)?;

        let new_session_id = self.spawn_pty(pane_id, &located.pane.cwd, &resolved)?;
        self.kill_session_quiet(&old_session_id);

        let snapshot = self.tab_manager.replace_pane(
            pane_id,
            new_session_id.clone(),
            resolved,
            located.pane.cwd,
        )?;

        self.emit_lifecycle(pane_id, Some(&new_session_id), PaneRuntimeStatus::Running, None);
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

        let snapshot = self.tab_manager.replace_pane(
            pane_id,
            new_session_id.clone(),
            resolved,
            next_cwd,
        )?;

        self.emit_lifecycle(pane_id, Some(&new_session_id), PaneRuntimeStatus::Running, None);
        self.emit_workspace_changed(&snapshot);

        info!(pane_id, cwd, "Pane cwd updated");
        Ok(snapshot)
    }

    pub fn write_pty(&self, pane_id: &str, data: &str) -> Result<(), TabbyError> {
        let session_id = self.tab_manager.session_id_for_pane(pane_id)?;
        self.pty_manager.write(&session_id, data)
    }

    pub fn resize_pty(
        &self,
        pane_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), TabbyError> {
        let session_id = self.tab_manager.session_id_for_pane(pane_id)?;
        self.pty_manager.resize(&session_id, cols, rows)
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
        let cwd = request
            .cwd
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or(&settings.default_working_directory);

        let mut seeds = Vec::with_capacity(pane_count);
        for _ in 0..pane_count {
            let pane_id = create_pane_id();
            let session_id = self.spawn_pty(&pane_id, cwd, &resolved)?;
            self.emit_lifecycle(&pane_id, Some(&session_id), PaneRuntimeStatus::Running, None);
            seeds.push(PaneSeed {
                pane_id,
                session_id,
                cwd: String::from(cwd),
                profile_id: resolved.id.clone(),
                profile_label: resolved.label.clone(),
                startup_command: resolved.startup_command.clone(),
            });
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
            let session_id = self.spawn_pty(&pane_id, cwd, &resolved)?;
            self.emit_lifecycle(&pane_id, Some(&session_id), PaneRuntimeStatus::Running, None);
            seeds.push(PaneSeed {
                pane_id,
                session_id,
                cwd: String::from(cwd),
                profile_id: resolved.id.clone(),
                profile_label: resolved.label.clone(),
                startup_command: resolved.startup_command.clone(),
            });
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
                if profile_id == CUSTOM_PROFILE_ID
                    && !fallback_custom_command.trim().is_empty()
                {
                    Some(String::from(fallback_custom_command))
                } else {
                    None
                }
            });
        resolve_profile(profile_id, startup_command)
    }

    fn spawn_pty(
        &self,
        pane_id: &str,
        cwd: &str,
        profile: &crate::domain::types::ResolvedProfile,
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

        if let Err(error) = self.app.emit(PANE_LIFECYCLE_EVENT, event) {
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
}
