use std::sync::Arc;

use tauri::{AppHandle, Manager, State};
use tracing::warn;

use crate::domain::error::TabbyError;
use crate::domain::types::AppSettings;
use crate::managers::settings::SettingsManager;

#[tauri::command]
#[specta::specta]
pub fn get_app_settings(
    settings_manager: State<'_, Arc<SettingsManager>>,
) -> Result<AppSettings, TabbyError> {
    settings_manager.get_settings()
}

#[tauri::command]
#[specta::specta]
pub fn update_app_settings(
    app: AppHandle,
    settings_manager: State<'_, Arc<SettingsManager>>,
    settings: AppSettings,
) -> Result<AppSettings, TabbyError> {
    let updated = settings_manager.update_settings(settings)?;

    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.set_fullscreen(updated.launch_fullscreen) {
            warn!(?error, "Failed to apply fullscreen preference");
        }
    }

    Ok(updated)
}
