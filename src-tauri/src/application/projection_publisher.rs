use tauri::{AppHandle, Emitter};
use tracing::warn;

use tabby_contracts::{
    RuntimeStatusChangedEvent, SettingsProjectionUpdatedEvent, WorkspaceProjectionUpdatedEvent,
    WorkspaceView,
};
use tabby_runtime::PaneRuntime;
use tabby_settings::{built_in_profile_catalog, UserPreferences};

use crate::mapping::dto_mappers;
use crate::shell::{
    RUNTIME_STATUS_CHANGED_EVENT, SETTINGS_PROJECTION_UPDATED_EVENT,
    WORKSPACE_PROJECTION_UPDATED_EVENT,
};

#[derive(Debug)]
pub struct ProjectionPublisher {
    app: AppHandle,
}

impl ProjectionPublisher {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn emit_workspace_projection_from_view(&self, workspace: &WorkspaceView) {
        if let Err(error) = self.app.emit(
            WORKSPACE_PROJECTION_UPDATED_EVENT,
            WorkspaceProjectionUpdatedEvent {
                workspace: workspace.clone(),
            },
        ) {
            warn!(?error, "Failed to emit workspace projection update");
        }
    }

    pub fn emit_settings_projection(&self, preferences: &UserPreferences) {
        let settings = dto_mappers::settings_view_from_preferences(preferences);
        let profile_catalog =
            dto_mappers::profile_catalog_view_from_catalog(&built_in_profile_catalog());
        if let Err(error) = self.app.emit(
            SETTINGS_PROJECTION_UPDATED_EVENT,
            SettingsProjectionUpdatedEvent {
                settings,
                profile_catalog,
            },
        ) {
            warn!(?error, "Failed to emit settings projection update");
        }
    }

    pub fn emit_runtime_status(&self, runtime: &PaneRuntime) {
        let view = dto_mappers::pane_runtime_to_view(runtime);
        if let Err(error) = self.app.emit(
            RUNTIME_STATUS_CHANGED_EVENT,
            RuntimeStatusChangedEvent { runtime: view },
        ) {
            warn!(?error, "Failed to emit runtime status update");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectionPublisher;

    #[test]
    fn projection_publisher_is_constructible() {
        // ProjectionPublisher requires a real AppHandle which needs a Tauri runtime.
        // This test validates the type exists and has the expected public API surface
        // by asserting it is Send + Sync (required for Tauri managed state).
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ProjectionPublisher>();
        assert_sync::<ProjectionPublisher>();
    }
}
