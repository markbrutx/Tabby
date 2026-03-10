use std::sync::Arc;

use tauri::State;

use tabby_contracts::{
    BrowserSurfaceCommandDto, GitCommandDto, GitResultDto, RuntimeCommandDto, SettingsCommandDto,
    SettingsView, WorkspaceBootstrapView, WorkspaceCommandDto, WorkspaceView,
};

use crate::shell::error::ShellError;
use crate::shell::AppShell;

#[tauri::command]
#[specta::specta]
pub fn bootstrap_shell(
    state: State<'_, Arc<AppShell>>,
) -> Result<WorkspaceBootstrapView, ShellError> {
    state.bootstrap()
}

#[tauri::command]
#[specta::specta]
pub fn dispatch_workspace_command(
    state: State<'_, Arc<AppShell>>,
    command: WorkspaceCommandDto,
) -> Result<WorkspaceView, ShellError> {
    state.dispatch_workspace_command(command)
}

#[tauri::command]
#[specta::specta]
pub fn dispatch_settings_command(
    state: State<'_, Arc<AppShell>>,
    command: SettingsCommandDto,
) -> Result<SettingsView, ShellError> {
    state.dispatch_settings_command(command)
}

#[tauri::command]
#[specta::specta]
pub fn dispatch_runtime_command(
    state: State<'_, Arc<AppShell>>,
    command: RuntimeCommandDto,
) -> Result<(), ShellError> {
    state.dispatch_runtime_command(command)
}

#[tauri::command]
#[specta::specta]
pub fn dispatch_browser_surface_command(
    state: State<'_, Arc<AppShell>>,
    command: BrowserSurfaceCommandDto,
) -> Result<(), ShellError> {
    state.dispatch_browser_surface_command(command)
}

#[tauri::command]
#[specta::specta]
pub fn dispatch_git_command(
    state: State<'_, Arc<AppShell>>,
    command: GitCommandDto,
) -> Result<GitResultDto, ShellError> {
    state.dispatch_git_command(command)
}
