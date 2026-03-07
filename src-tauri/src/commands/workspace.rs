use std::sync::{Arc, Mutex};

use tauri::State;

use crate::cli::CliArgs;
use crate::domain::commands::{
    LaunchRequest, NewTabRequest, SplitPaneRequest, UpdatePaneCwdRequest, UpdatePaneProfileRequest,
};
use crate::domain::error::TabbyError;
use crate::domain::profiles::built_in_profiles;
use crate::domain::settings::AppSettings;
use crate::domain::snapshot::{BootstrapSnapshot, WorkspaceSnapshot};
use crate::managers::coordinator::Coordinator;
use crate::managers::settings::SettingsManager;
use crate::managers::tab::TabManager;

#[derive(Debug)]
pub struct LaunchOverrides(pub Mutex<Option<CliArgs>>);

#[tauri::command]
#[specta::specta]
pub fn bootstrap_workspace(
    tab_manager: State<'_, Arc<TabManager>>,
    settings_manager: State<'_, Arc<SettingsManager>>,
) -> Result<BootstrapSnapshot, TabbyError> {
    Ok(BootstrapSnapshot {
        workspace: tab_manager.snapshot()?,
        settings: settings_manager.get_settings()?,
        profiles: built_in_profiles(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn create_tab(
    coordinator: State<'_, Arc<Coordinator>>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    request: NewTabRequest,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let settings = settings_manager.get_settings()?;
    coordinator.create_tab(&request, &settings)
}

#[tauri::command]
#[specta::specta]
pub fn close_tab(
    coordinator: State<'_, Arc<Coordinator>>,
    tab_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    coordinator.close_tab(&tab_id)
}

#[tauri::command]
#[specta::specta]
pub fn set_active_tab(
    coordinator: State<'_, Arc<Coordinator>>,
    tab_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    coordinator.set_active_tab(&tab_id)
}

#[tauri::command]
#[specta::specta]
pub fn focus_pane(
    coordinator: State<'_, Arc<Coordinator>>,
    tab_id: String,
    pane_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    coordinator.focus_pane(&tab_id, &pane_id)
}

#[tauri::command]
#[specta::specta]
pub fn restart_pane(
    coordinator: State<'_, Arc<Coordinator>>,
    pane_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    coordinator.restart_pane(&pane_id)
}

#[tauri::command]
#[specta::specta]
pub fn update_pane_profile(
    coordinator: State<'_, Arc<Coordinator>>,
    request: UpdatePaneProfileRequest,
) -> Result<WorkspaceSnapshot, TabbyError> {
    coordinator.update_pane_profile(
        &request.pane_id,
        &request.profile_id,
        request.startup_command,
    )
}

#[tauri::command]
#[specta::specta]
pub fn update_pane_cwd(
    coordinator: State<'_, Arc<Coordinator>>,
    request: UpdatePaneCwdRequest,
) -> Result<WorkspaceSnapshot, TabbyError> {
    coordinator.update_pane_cwd(&request.pane_id, &request.cwd)
}

#[tauri::command]
#[specta::specta]
pub fn split_pane(
    coordinator: State<'_, Arc<Coordinator>>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    request: SplitPaneRequest,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let settings = settings_manager.get_settings()?;
    coordinator.split_pane(&request, &settings)
}

#[tauri::command]
#[specta::specta]
pub fn close_pane(
    coordinator: State<'_, Arc<Coordinator>>,
    pane_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    coordinator.close_pane(&pane_id)
}

#[tauri::command]
#[specta::specta]
pub fn track_pane_cwd(
    coordinator: State<'_, Arc<Coordinator>>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    pane_id: String,
    cwd: String,
) -> Result<(), TabbyError> {
    coordinator.track_pane_cwd(&pane_id, &cwd, &settings_manager)
}

#[tauri::command]
#[specta::specta]
pub fn swap_panes(
    coordinator: State<'_, Arc<Coordinator>>,
    pane_id_a: String,
    pane_id_b: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    coordinator.swap_panes(&pane_id_a, &pane_id_b)
}

// TODO(phase-4): re-enable when single-instance CLI routing is wired
#[allow(dead_code)]
fn consume_launch_request(
    launch_overrides: &LaunchOverrides,
    settings: &AppSettings,
) -> Result<LaunchRequest, TabbyError> {
    let mut overrides = launch_overrides
        .0
        .lock()
        .map_err(|_| TabbyError::State(String::from("CLI overrides lock poisoned")))?;
    let cli_args = overrides.take().unwrap_or_default();

    LaunchRequest::from_cli_args(cli_args, settings)
}
