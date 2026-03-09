pub mod cli;

pub mod application;
mod commands;
mod infrastructure;
mod mapping;
mod menu;
pub mod shell;

pub use cli::CliArgs;

use std::sync::Arc;

use specta_typescript::Typescript;
use tauri::{Emitter, Manager, Wry};
use tauri_specta::{collect_commands, Builder as SpectaBuilder};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

use tabby_contracts::{
    BrowserSurfaceBoundsDto, BrowserSurfaceCommandDto, LayoutPresetDto, PaneRuntimeView,
    PaneSpecDto, ProfileCatalogView, RuntimeCommandDto, RuntimeStatusChangedEvent,
    SettingsCommandDto, SettingsProjectionUpdatedEvent, SettingsView, SplitDirectionDto,
    SplitNodeDto, TerminalOutputEvent, WorkspaceBootstrapView, WorkspaceCommandDto,
    WorkspaceProjectionUpdatedEvent, WorkspaceView,
};

use crate::shell::AppShell;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("tabby=info,tao=warn,wry=warn"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn specta_builder() -> SpectaBuilder<Wry> {
    SpectaBuilder::<Wry>::new()
        .commands(collect_commands![
            commands::shell::bootstrap_shell,
            commands::shell::dispatch_workspace_command,
            commands::shell::dispatch_settings_command,
            commands::shell::dispatch_runtime_command,
            commands::shell::dispatch_browser_surface_command,
        ])
        .typ::<WorkspaceBootstrapView>()
        .typ::<WorkspaceCommandDto>()
        .typ::<WorkspaceView>()
        .typ::<WorkspaceProjectionUpdatedEvent>()
        .typ::<SettingsCommandDto>()
        .typ::<SettingsView>()
        .typ::<SettingsProjectionUpdatedEvent>()
        .typ::<RuntimeCommandDto>()
        .typ::<PaneRuntimeView>()
        .typ::<RuntimeStatusChangedEvent>()
        .typ::<TerminalOutputEvent>()
        .typ::<BrowserSurfaceCommandDto>()
        .typ::<BrowserSurfaceBoundsDto>()
        .typ::<PaneSpecDto>()
        .typ::<ProfileCatalogView>()
        .typ::<SplitNodeDto>()
        .typ::<SplitDirectionDto>()
        .typ::<LayoutPresetDto>()
        .typ::<shell::error::ShellError>()
}

fn focus_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.show() {
            warn!(?error, "Failed to show main window");
        }
        if let Err(error) = window.set_focus() {
            warn!(?error, "Failed to focus main window");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(cli_args: CliArgs) {
    init_tracing();

    let specta_builder = specta_builder();

    #[cfg(debug_assertions)]
    if let Err(export_error) =
        specta_builder.export(Typescript::default(), "../src/contracts/tauri-bindings.ts")
    {
        error!(?export_error, "Failed to export Typescript bindings");
    }

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            focus_main_window(app);

            match CliArgs::from_argv(&argv) {
                Ok(parsed_args) => {
                    if let Some(shell) = app.try_state::<Arc<AppShell>>() {
                        if let Err(error) = shell.apply_cli_launch_request(parsed_args) {
                            error!(?error, "Failed to apply routed CLI launch request");
                        }
                    }
                }
                Err(error) => {
                    warn!(?error, ?argv, "Failed to parse routed CLI arguments");
                }
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .menu(menu::build_menu)
        .on_menu_event(|app_handle, event| menu::handle_menu_event(app_handle, &event))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.emit("app-close-requested", ());
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let shell = Arc::new(AppShell::new(app_handle.clone(), cli_args.clone()).map_err(
                |error| -> Box<dyn std::error::Error> {
                    Box::new(std::io::Error::other(error.to_string()))
                },
            )?);

            let settings = shell
                .bootstrap()
                .map_err(|error| -> Box<dyn std::error::Error> {
                    Box::new(std::io::Error::other(error.to_string()))
                })?
                .settings;

            app.manage(shell);

            if settings.launch_fullscreen {
                if let Some(window) = app.get_webview_window("main") {
                    if let Err(error) = window.set_fullscreen(true) {
                        warn!(?error, "Failed to enable fullscreen on startup");
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(specta_builder.invoke_handler());

    if let Err(error) = builder.run(tauri::generate_context!()) {
        error!(?error, "Tabby failed to run");
    }
}

#[cfg(test)]
mod tests {
    use super::specta_builder;
    use specta_typescript::Typescript;

    #[test]
    fn exports_typescript_bindings() {
        specta_builder()
            .export(Typescript::default(), "../src/contracts/tauri-bindings.ts")
            .expect("bindings should export");
    }
}
