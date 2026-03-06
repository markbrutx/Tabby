use std::sync::{Arc, Mutex};

use tauri::State;
use tracing::warn;

use crate::cli::CliArgs;
use crate::domain::error::TabbyError;
use crate::domain::types::{
    built_in_profiles, create_pane_id, resolve_profile, BootstrapSnapshot, NewTabRequest, PaneSeed,
    UpdatePaneCwdRequest, UpdatePaneProfileRequest, WorkspaceSnapshot,
};
use crate::managers::grid::GridManager;
use crate::managers::pty::{PtyManager, SpawnRequest};
use crate::managers::settings::SettingsManager;
use crate::managers::tab::TabManager;

#[derive(Debug)]
pub struct LaunchOverrides(pub Mutex<Option<CliArgs>>);

#[tauri::command]
pub fn bootstrap_workspace(
    tab_manager: State<'_, Arc<TabManager>>,
    grid_manager: State<'_, Arc<GridManager>>,
    pty_manager: State<'_, Arc<PtyManager>>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    launch_overrides: State<'_, LaunchOverrides>,
) -> Result<BootstrapSnapshot, TabbyError> {
    if tab_manager.is_empty()? {
        let settings = settings_manager.get_settings()?;
        let request = consume_launch_request(&launch_overrides, &settings)?;
        let workspace = create_tab_internal(
            &tab_manager,
            &grid_manager,
            &pty_manager,
            &settings,
            request,
        )?;

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
pub fn create_tab(
    tab_manager: State<'_, Arc<TabManager>>,
    grid_manager: State<'_, Arc<GridManager>>,
    pty_manager: State<'_, Arc<PtyManager>>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    request: NewTabRequest,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let settings = settings_manager.get_settings()?;
    create_tab_internal(
        &tab_manager,
        &grid_manager,
        &pty_manager,
        &settings,
        request,
    )
}

#[tauri::command]
pub fn close_tab(
    tab_manager: State<'_, Arc<TabManager>>,
    grid_manager: State<'_, Arc<GridManager>>,
    pty_manager: State<'_, Arc<PtyManager>>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    tab_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let (snapshot, session_ids) = tab_manager.close_tab(&tab_id)?;
    pty_manager.kill_many(&session_ids);

    if snapshot.tabs.is_empty() {
        let settings = settings_manager.get_settings()?;
        return create_tab_internal(
            &tab_manager,
            &grid_manager,
            &pty_manager,
            &settings,
            NewTabRequest {
                preset: settings.default_layout,
                cwd: Some(settings.default_working_directory.clone()),
                profile_id: Some(settings.default_profile_id.clone()),
                startup_command: Some(settings.default_custom_command.clone()),
            },
        );
    }

    Ok(snapshot)
}

#[tauri::command]
pub fn set_active_tab(
    tab_manager: State<'_, Arc<TabManager>>,
    tab_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    tab_manager.set_active_tab(&tab_id)
}

#[tauri::command]
pub fn focus_pane(
    tab_manager: State<'_, Arc<TabManager>>,
    tab_id: String,
    pane_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    tab_manager.focus_pane(&tab_id, &pane_id)
}

#[tauri::command]
pub fn restart_pane(
    tab_manager: State<'_, Arc<TabManager>>,
    pty_manager: State<'_, Arc<PtyManager>>,
    pane_id: String,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let located = tab_manager.locate_pane(&pane_id)?;
    let resolved = resolve_profile(
        &located.pane.profile_id,
        located.pane.startup_command.clone(),
    )?;
    let session_id = pty_manager.spawn(SpawnRequest {
        pane_id: pane_id.clone(),
        cwd: located.pane.cwd.clone(),
        profile: resolved.clone(),
    })?;

    if let Err(error) = pty_manager.kill(&located.pane.session_id) {
        warn!(
            ?error,
            "Failed to kill previous pane session during restart"
        );
    }

    tab_manager.replace_pane(
        &pane_id,
        session_id,
        resolved.id,
        resolved.label,
        resolved.startup_command,
        located.pane.cwd,
    )
}

#[tauri::command]
pub fn update_pane_profile(
    tab_manager: State<'_, Arc<TabManager>>,
    pty_manager: State<'_, Arc<PtyManager>>,
    request: UpdatePaneProfileRequest,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let located = tab_manager.locate_pane(&request.pane_id)?;
    let resolved = resolve_profile(&request.profile_id, request.startup_command)?;
    let session_id = pty_manager.spawn(SpawnRequest {
        pane_id: request.pane_id.clone(),
        cwd: located.pane.cwd.clone(),
        profile: resolved.clone(),
    })?;

    if let Err(error) = pty_manager.kill(&located.pane.session_id) {
        warn!(
            ?error,
            "Failed to kill previous pane session after profile swap"
        );
    }

    tab_manager.replace_pane(
        &request.pane_id,
        session_id,
        resolved.id,
        resolved.label,
        resolved.startup_command,
        located.pane.cwd,
    )
}

#[tauri::command]
pub fn update_pane_cwd(
    tab_manager: State<'_, Arc<TabManager>>,
    pty_manager: State<'_, Arc<PtyManager>>,
    request: UpdatePaneCwdRequest,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let next_cwd = request.cwd.trim().to_string();
    if next_cwd.is_empty() {
        return Err(TabbyError::Validation(String::from(
            "Working directory cannot be empty",
        )));
    }

    let located = tab_manager.locate_pane(&request.pane_id)?;
    let resolved = resolve_profile(
        &located.pane.profile_id,
        located.pane.startup_command.clone(),
    )?;
    let session_id = pty_manager.spawn(SpawnRequest {
        pane_id: request.pane_id.clone(),
        cwd: next_cwd.clone(),
        profile: resolved.clone(),
    })?;

    if let Err(error) = pty_manager.kill(&located.pane.session_id) {
        warn!(
            ?error,
            "Failed to kill previous pane session after cwd change"
        );
    }

    tab_manager.replace_pane(
        &request.pane_id,
        session_id,
        resolved.id,
        resolved.label,
        resolved.startup_command,
        next_cwd,
    )
}

fn create_tab_internal(
    tab_manager: &TabManager,
    grid_manager: &GridManager,
    pty_manager: &PtyManager,
    settings: &crate::domain::types::AppSettings,
    request: NewTabRequest,
) -> Result<WorkspaceSnapshot, TabbyError> {
    let preset = request.preset;
    let profile_id = request
        .profile_id
        .clone()
        .unwrap_or_else(|| settings.default_profile_id.clone());
    let startup_command = request
        .startup_command
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            if profile_id == "custom" && !settings.default_custom_command.trim().is_empty() {
                Some(settings.default_custom_command.clone())
            } else {
                None
            }
        });
    let resolved_profile = resolve_profile(&profile_id, startup_command)?;
    let cwd = request
        .cwd
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| settings.default_working_directory.clone());

    let pane_count = grid_manager.definition(preset).pane_count;
    let mut pane_seeds = Vec::with_capacity(pane_count);

    for _ in 0..pane_count {
        let pane_id = create_pane_id();
        let session_id = pty_manager.spawn(SpawnRequest {
            pane_id: pane_id.clone(),
            cwd: cwd.clone(),
            profile: resolved_profile.clone(),
        })?;
        pane_seeds.push(PaneSeed {
            pane_id,
            session_id,
            cwd: cwd.clone(),
            profile_id: resolved_profile.id.clone(),
            profile_label: resolved_profile.label.clone(),
            startup_command: resolved_profile.startup_command.clone(),
        });
    }

    tab_manager.create_tab(preset, pane_seeds)
}

fn consume_launch_request(
    launch_overrides: &LaunchOverrides,
    settings: &crate::domain::types::AppSettings,
) -> Result<NewTabRequest, TabbyError> {
    let mut overrides = launch_overrides
        .0
        .lock()
        .map_err(|_| TabbyError::State(String::from("CLI overrides lock poisoned")))?;
    let cli_args = overrides.take().unwrap_or_default();

    let preset = match cli_args.layout.as_deref() {
        Some("1x1") => crate::domain::types::LayoutPreset::OneByOne,
        Some("1x2") => crate::domain::types::LayoutPreset::OneByTwo,
        Some("2x2") => crate::domain::types::LayoutPreset::TwoByTwo,
        Some("2x3") => crate::domain::types::LayoutPreset::TwoByThree,
        Some("3x3") => crate::domain::types::LayoutPreset::ThreeByThree,
        Some(other) => {
            return Err(TabbyError::Validation(format!(
                "Unsupported layout override: {other}"
            )))
        }
        None => settings.default_layout,
    };

    Ok(NewTabRequest {
        preset,
        cwd: cli_args.cwd,
        profile_id: cli_args.profile,
        startup_command: cli_args.command,
    })
}
