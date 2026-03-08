mod application;
mod browser;
pub mod cli;
mod menu;
mod settings;
mod shared;
mod terminal;
mod workspace;

pub use cli::CliArgs;

use std::sync::Arc;

use specta_typescript::Typescript;
use tauri::{Emitter, Manager};
use tauri_specta::{collect_commands, Builder as SpectaBuilder};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

use crate::application::coordinator::Coordinator;
use crate::settings::repository::settings_repository::SettingsManager;
use crate::shared::events::{
    BrowserUrlChangedEvent, PaneLifecycleEvent, PtyOutputEvent, WorkspaceChangedEvent,
};
use crate::terminal::service::pty_service::PtyManager;
use crate::workspace::commands::workspace_commands::{apply_cli_launch_request, LaunchOverrides};
use crate::workspace::domain::layout::SplitNode;
use crate::workspace::domain::pane::PaneKind;
use crate::workspace::service::tab_service::TabManager;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("tabby=info,tao=warn,wry=warn"));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn specta_builder() -> SpectaBuilder<tauri::Wry> {
    SpectaBuilder::<tauri::Wry>::new()
        .commands(collect_commands![
            settings::commands::get_app_settings,
            settings::commands::update_app_settings,
            settings::commands::reset_app_settings,
            workspace::commands::workspace_commands::bootstrap_workspace,
            workspace::commands::workspace_commands::create_tab,
            workspace::commands::workspace_commands::close_tab,
            workspace::commands::workspace_commands::set_active_tab,
            workspace::commands::workspace_commands::focus_pane,
            workspace::commands::workspace_commands::restart_pane,
            workspace::commands::workspace_commands::update_pane_profile,
            workspace::commands::workspace_commands::update_pane_cwd,
            workspace::commands::workspace_commands::split_pane,
            workspace::commands::workspace_commands::close_pane,
            workspace::commands::workspace_commands::track_pane_cwd,
            workspace::commands::workspace_commands::swap_panes,
            browser::commands::create_browser_webview,
            browser::commands::navigate_browser,
            browser::commands::close_browser_webview,
            browser::commands::set_browser_webview_bounds,
            browser::commands::set_browser_webview_visible,
            terminal::commands::write_pty,
            terminal::commands::resize_pty,
        ])
        .typ::<SplitNode>()
        .typ::<PaneKind>()
        .typ::<PtyOutputEvent>()
        .typ::<PaneLifecycleEvent>()
        .typ::<WorkspaceChangedEvent>()
        .typ::<BrowserUrlChangedEvent>()
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
        specta_builder.export(Typescript::default(), "../src/lib/tauri-bindings.ts")
    {
        error!(?export_error, "Failed to export Typescript bindings");
    }

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            focus_main_window(app);

            match CliArgs::from_argv(&argv) {
                Ok(parsed_args) => {
                    if let Err(error) = apply_cli_launch_request(app, parsed_args) {
                        error!(?error, "Failed to apply routed CLI launch request");
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
            let settings_manager = Arc::new(SettingsManager::new(app_handle.clone()));
            let tab_manager = Arc::new(TabManager::new());
            let pty_manager = Arc::new(PtyManager::new(app_handle.clone()));
            let coordinator = Arc::new(Coordinator::new(
                app_handle,
                tab_manager.clone(),
                pty_manager.clone(),
            ));
            let settings = settings_manager
                .get_settings()
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;

            app.manage(settings_manager);
            app.manage(tab_manager);
            app.manage(pty_manager);
            app.manage(coordinator);
            app.manage(LaunchOverrides(std::sync::Mutex::new(Some(
                cli_args.clone(),
            ))));

            if settings.launch_fullscreen {
                if let Some(window) = app.get_webview_window("main") {
                    if let Err(error) = window.set_fullscreen(true) {
                        error!(?error, "Failed to enable fullscreen on startup");
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
            .export(Typescript::default(), "../src/lib/tauri-bindings.ts")
            .expect("bindings should export");
    }
}
