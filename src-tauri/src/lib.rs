pub mod cli;
mod commands;
mod domain;
mod managers;

pub use cli::CliArgs;

use std::sync::Arc;

use specta_typescript::Typescript;
use tauri::menu::{AboutMetadataBuilder, MenuBuilder, SubmenuBuilder};
use tauri::{Emitter, Manager};
use tauri_specta::{collect_commands, Builder as SpectaBuilder};
use tracing::error;
use tracing_subscriber::EnvFilter;

const MENU_ITEM_SETTINGS: &str = "open-settings";
const EVENT_OPEN_SETTINGS: &str = "menu-open-settings";

use crate::commands::workspace::LaunchOverrides;
use crate::domain::events::{
    BrowserUrlChangedEvent, PaneLifecycleEvent, PtyOutputEvent, WorkspaceChangedEvent,
};
use crate::domain::types::{PaneKind, SplitNode};
use crate::managers::coordinator::Coordinator;
use crate::managers::pty::PtyManager;
use crate::managers::settings::SettingsManager;
use crate::managers::tab::TabManager;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("tabby=info,tao=warn,wry=warn"));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn specta_builder() -> SpectaBuilder<tauri::Wry> {
    SpectaBuilder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::settings::get_app_settings,
            commands::settings::update_app_settings,
            commands::settings::reset_app_settings,
            commands::workspace::bootstrap_workspace,
            commands::workspace::create_tab,
            commands::workspace::close_tab,
            commands::workspace::set_active_tab,
            commands::workspace::focus_pane,
            commands::workspace::restart_pane,
            commands::workspace::update_pane_profile,
            commands::workspace::update_pane_cwd,
            commands::workspace::split_pane,
            commands::workspace::close_pane,
            commands::workspace::track_pane_cwd,
            commands::workspace::swap_panes,
            commands::browser::create_browser_webview,
            commands::browser::navigate_browser,
            commands::browser::close_browser_webview,
            commands::browser::set_browser_webview_bounds,
            commands::browser::set_browser_webview_visible,
            commands::pty::write_pty,
            commands::pty::resize_pty,
        ])
        .typ::<SplitNode>()
        .typ::<PaneKind>()
        .typ::<PtyOutputEvent>()
        .typ::<PaneLifecycleEvent>()
        .typ::<WorkspaceChangedEvent>()
        .typ::<BrowserUrlChangedEvent>()
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .menu(|handle| {
            let about_meta = AboutMetadataBuilder::new()
                .name(Some("Tabby"))
                .version(Some(env!("CARGO_PKG_VERSION")))
                .build();

            let app_submenu = SubmenuBuilder::new(handle, "Tabby")
                .about(Some(about_meta))
                .separator()
                .item(
                    &tauri::menu::MenuItemBuilder::with_id(MENU_ITEM_SETTINGS, "Settings…")
                        .accelerator("CmdOrCtrl+,")
                        .build(handle)?,
                )
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()?;

            MenuBuilder::new(handle).item(&app_submenu).build()
        })
        .on_menu_event(|app_handle, event| {
            if event.id().as_ref() == MENU_ITEM_SETTINGS {
                if let Err(emit_err) = app_handle.emit(EVENT_OPEN_SETTINGS, ()) {
                    error!(?emit_err, "Failed to emit menu-open-settings event");
                }
            }
        })
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
