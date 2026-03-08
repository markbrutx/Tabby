use std::sync::Arc;

use tauri::{AppHandle, Manager, State};
use tracing::warn;

use crate::settings::domain::app_settings::AppSettings;
use crate::settings::repository::settings_repository::SettingsManager;
use crate::shared::error::TabbyError;

fn apply_fullscreen(app: &AppHandle, launch_fullscreen: bool) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.set_fullscreen(launch_fullscreen) {
            warn!(?error, "Failed to apply fullscreen preference");
        }
    }
}

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
    apply_fullscreen(&app, updated.launch_fullscreen);
    Ok(updated)
}

#[tauri::command]
#[specta::specta]
pub fn reset_app_settings(
    app: AppHandle,
    settings_manager: State<'_, Arc<SettingsManager>>,
) -> Result<AppSettings, TabbyError> {
    let reset = settings_manager.reset_settings()?;
    apply_fullscreen(&app, reset.launch_fullscreen);
    Ok(reset)
}
