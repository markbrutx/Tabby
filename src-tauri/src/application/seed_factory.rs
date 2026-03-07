use crate::settings::domain::app_settings::AppSettings;
use crate::settings::domain::profiles::{
    resolve_profile, ResolvedProfile, BROWSER_PROFILE_ID, CUSTOM_PROFILE_ID,
};
use crate::shared::error::TabbyError;
use crate::terminal::service::pty_service::{PtyManager, SpawnRequest};
use crate::workspace::domain::pane::{create_pane_id, PaneKind, PaneSeed};
use crate::workspace::domain::requests::{NewTabRequest, PaneConfig};
use crate::workspace::domain::snapshot::PaneRuntimeStatus;

use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tracing::warn;

use crate::shared::events::{PaneLifecycleEvent, PANE_LIFECYCLE_EVENT_NAME};

pub struct SeedFactory {
    app: AppHandle,
    pty_manager: Arc<PtyManager>,
}

impl SeedFactory {
    pub fn new(app: AppHandle, pty_manager: Arc<PtyManager>) -> Self {
        Self { app, pty_manager }
    }

    pub fn seeds_uniform(
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

    pub fn seeds_from_pane_configs(
        &self,
        configs: &[PaneConfig],
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

    pub fn resolve_effective_profile(
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

    pub fn create_terminal_seed(
        &self,
        pane_id: String,
        cwd: &str,
        resolved: &ResolvedProfile,
    ) -> Result<PaneSeed, TabbyError> {
        let session_id = self.spawn_pty(&pane_id, cwd, resolved)?;
        self.emit_lifecycle(
            &pane_id,
            Some(&session_id),
            PaneRuntimeStatus::Running,
            None,
        );
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

    pub fn create_browser_seed(&self, pane_id: String, cwd: &str, url: Option<String>) -> PaneSeed {
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

    pub fn spawn_pty(
        &self,
        pane_id: &str,
        cwd: &str,
        profile: &ResolvedProfile,
    ) -> Result<String, TabbyError> {
        self.pty_manager.spawn(SpawnRequest {
            pane_id: String::from(pane_id),
            cwd: String::from(cwd),
            profile: profile.clone(),
        })
    }

    pub fn kill_session_quiet(&self, session_id: &str) {
        if let Err(error) = self.pty_manager.kill(session_id) {
            warn!(?error, session_id, "Failed to kill previous PTY session");
        }
    }

    pub fn emit_lifecycle(
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
}
