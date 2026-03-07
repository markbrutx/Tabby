use std::sync::{Arc, Mutex};

use tauri::State;

use crate::cli::CliArgs;
use crate::domain::commands::{
    LaunchRequest, NewTabRequest, SplitPaneRequest, UpdatePaneCwdRequest,
    UpdatePaneProfileRequest,
};
use crate::domain::error::TabbyError;
use crate::domain::snapshot::{BootstrapSnapshot, WorkspaceSnapshot};
use crate::domain::types::built_in_profiles;
use crate::managers::coordinator::Coordinator;
use crate::managers::settings::SettingsManager;
use crate::managers::tab::TabManager;

#[derive(Debug)]
pub struct LaunchOverrides(pub Mutex<Option<CliArgs>>);

#[tauri::command]
#[specta::specta]
pub fn bootstrap_workspace(
    tab_manager: State<'_, Arc<TabManager>>,
    coordinator: State<'_, Arc<Coordinator>>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    launch_overrides: State<'_, LaunchOverrides>,
) -> Result<BootstrapSnapshot, TabbyError> {
    if tab_manager.is_empty()? {
        let settings = settings_manager.get_settings()?;
        let request = consume_launch_request(&launch_overrides, &settings)?;
        let workspace = coordinator.create_tab(&request.into(), &settings)?;

        return Ok(BootstrapSnapshot {
            workspace,
            settings,
            profiles: built_in_profiles(),
        });
    }

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
    settings_manager: State<'_, Arc<SettingsManager>>,
    tab_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let settings = settings_manager.get_settings()?;
    coordinator.close_tab(&tab_id, &settings)
}

#[tauri::command]
#[specta::specta]
pub fn set_active_tab(
    tab_manager: State<'_, Arc<TabManager>>,
    tab_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    tab_manager.set_active_tab(&tab_id)
}

#[tauri::command]
#[specta::specta]
pub fn focus_pane(
    tab_manager: State<'_, Arc<TabManager>>,
    tab_id: String,
    pane_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    tab_manager.focus_pane(&tab_id, &pane_id)
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
    coordinator.update_pane_profile(&request.pane_id, &request.profile_id, request.startup_command)
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
    settings_manager: State<'_, Arc<SettingsManager>>,
    pane_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let settings = settings_manager.get_settings()?;
    coordinator.close_pane(&pane_id, &settings)
}

fn consume_launch_request(
    launch_overrides: &LaunchOverrides,
    settings: &crate::domain::types::AppSettings,
) -> Result<LaunchRequest, TabbyError> {
    let mut overrides = launch_overrides
        .0
        .lock()
        .map_err(|_| TabbyError::State(String::from("CLI overrides lock poisoned")))?;
    let cli_args = overrides.take().unwrap_or_default();

    LaunchRequest::from_cli_args(cli_args, settings)
}
