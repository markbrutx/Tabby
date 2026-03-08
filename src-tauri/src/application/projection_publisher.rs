use tauri::{AppHandle, Emitter};
use tracing::warn;

use tabby_contracts::{
    PaneRuntimeView, RuntimeStatusChangedEvent, SettingsProjectionUpdatedEvent, SettingsView,
    WorkspaceProjectionUpdatedEvent, WorkspaceView,
};
use tabby_settings::built_in_profile_catalog;

use crate::shell::mapping::profile_catalog_view_from_catalog;
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

    pub fn emit_workspace_projection(&self, workspace: &WorkspaceView) {
        if let Err(error) = self.app.emit(
            WORKSPACE_PROJECTION_UPDATED_EVENT,
            WorkspaceProjectionUpdatedEvent {
                workspace: workspace.clone(),
            },
        ) {
            warn!(?error, "Failed to emit workspace projection update");
        }
    }

    pub fn emit_settings_projection(&self, settings: &SettingsView) {
        if let Err(error) = self.app.emit(
            SETTINGS_PROJECTION_UPDATED_EVENT,
            SettingsProjectionUpdatedEvent {
                settings: settings.clone(),
                profile_catalog: profile_catalog_view_from_catalog(&built_in_profile_catalog()),
            },
        ) {
            warn!(?error, "Failed to emit settings projection update");
        }
    }

    pub fn emit_runtime_status(&self, runtime: &PaneRuntimeView) {
        if let Err(error) = self.app.emit(
            RUNTIME_STATUS_CHANGED_EVENT,
            RuntimeStatusChangedEvent {
                runtime: runtime.clone(),
            },
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
