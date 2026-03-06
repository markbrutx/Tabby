use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

use crate::domain::commands::NewTabRequest;
use crate::domain::error::TabbyError;
use crate::domain::events::{PaneLifecycleEvent, WorkspaceChangedEvent};
use crate::domain::snapshot::{PaneRuntimeStatus, WorkspaceSnapshot};
use crate::domain::types::{create_pane_id, resolve_profile, AppSettings, PaneSeed};
use crate::managers::grid::GridManager;
use crate::managers::pty::{PtyManager, SpawnRequest};
use crate::managers::tab::TabManager;

const PANE_LIFECYCLE_EVENT: &str = "pane-lifecycle";
const WORKSPACE_CHANGED_EVENT: &str = "workspace-changed";

#[derive(Debug, Clone)]
pub struct Coordinator {
    app: AppHandle,
    tab_manager: Arc<TabManager>,
    grid_manager: Arc<GridManager>,
    pty_manager: Arc<PtyManager>,
}

impl Coordinator {
    pub fn new(
        app: AppHandle,
        tab_manager: Arc<TabManager>,
        grid_manager: Arc<GridManager>,
        pty_manager: Arc<PtyManager>,
    ) -> Self {
        Self {
            app,
            tab_manager,
            grid_manager,
            pty_manager,
        }
    }

    pub fn create_tab(
        &self,
        request: &NewTabRequest,
        settings: &AppSettings,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let preset = request.preset;
        let profile_id = request
            .profile_id
            .as_deref()
            .unwrap_or(&settings.default_profile_id);
        let startup_command = request
            .startup_command
            .clone()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| {
                if profile_id == "custom" && !settings.default_custom_command.trim().is_empty() {
                    Some(settings.default_custom_command.clone())
                } else {
                    None
                }
            });
        let resolved = resolve_profile(profile_id, startup_command)?;
        let cwd = request
            .cwd
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or(&settings.default_working_directory);

        let pane_count = usize::from(self.grid_manager.definition(preset).pane_count);
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

        let snapshot = self.tab_manager.create_tab(preset, seeds)?;
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
        settings: &AppSettings,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let (snapshot, session_ids) = self.tab_manager.close_tab(tab_id)?;
        self.pty_manager.kill_many(&session_ids);

        info!(tab_id, killed_sessions = session_ids.len(), "Tab closed");

        if snapshot.tabs.is_empty() {
            let fresh = self.create_tab(
                &NewTabRequest {
                    preset: settings.default_layout,
                    cwd: Some(settings.default_working_directory.clone()),
                    profile_id: Some(settings.default_profile_id.clone()),
                    startup_command: Some(settings.default_custom_command.clone()),
                },
                settings,
            )?;
            return Ok(fresh);
        }

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
            resolved.id,
            resolved.label,
            resolved.startup_command,
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
            resolved.id,
            resolved.label,
            resolved.startup_command,
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
            resolved.id,
            resolved.label,
            resolved.startup_command,
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
