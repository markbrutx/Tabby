use tauri::menu::{AboutMetadataBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Emitter, Wry};
use tracing::error;

const MENU_ITEM_SETTINGS: &str = "open-settings";
const EVENT_OPEN_SETTINGS: &str = "menu-open-settings";

pub fn build_menu(handle: &AppHandle) -> tauri::Result<tauri::menu::Menu<Wry>> {
    let about_meta = AboutMetadataBuilder::new()
        .name(Some("Tabby"))
        .version(Some(env!("CARGO_PKG_VERSION")))
        .build();

    let app_submenu = SubmenuBuilder::new(handle, "Tabby")
        .about(Some(about_meta))
        .separator()
        .item(
            &MenuItemBuilder::with_id(MENU_ITEM_SETTINGS, "Settings…")
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

    let edit_submenu = SubmenuBuilder::new(handle, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let mut ws = SubmenuBuilder::new(handle, "Workspace")
        .item(
            &MenuItemBuilder::with_id("shortcut-new-tab", "New Tab")
                .accelerator("CmdOrCtrl+T")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("shortcut-close-pane", "Close Pane")
                .accelerator("CmdOrCtrl+W")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("shortcut-close-tab", "Close Workspace")
                .accelerator("CmdOrCtrl+Shift+W")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("shortcut-split-right", "Split Right")
                .accelerator("CmdOrCtrl+D")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("shortcut-split-down", "Split Down")
                .accelerator("CmdOrCtrl+E")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("shortcut-restart-pane", "Restart Pane")
                .accelerator("CmdOrCtrl+Shift+R")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("shortcut-next-pane", "Next Pane")
                .accelerator("CmdOrCtrl+]")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("shortcut-prev-pane", "Previous Pane")
                .accelerator("CmdOrCtrl+[")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("shortcut-shortcuts-help", "Keyboard Shortcuts")
                .accelerator("CmdOrCtrl+/")
                .build(handle)?,
        )
        .separator();

    for i in 1u8..=9 {
        ws = ws.item(
            &MenuItemBuilder::with_id(
                format!("shortcut-tab-{i}"),
                format!("Switch to Tab {i}"),
            )
            .accelerator(format!("CmdOrCtrl+{i}"))
            .build(handle)?,
        );
    }

    let workspace_submenu = ws.build()?;

    MenuBuilder::new(handle)
        .item(&app_submenu)
        .item(&edit_submenu)
        .item(&workspace_submenu)
        .build()
}

pub fn handle_menu_event(app_handle: &AppHandle, event: &tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    if id == MENU_ITEM_SETTINGS {
        if let Err(emit_err) = app_handle.emit(EVENT_OPEN_SETTINGS, ()) {
            error!(?emit_err, "Failed to emit menu-open-settings event");
        }
        return;
    }
    if id.starts_with("shortcut-") {
        if let Err(emit_err) = app_handle.emit(id, ()) {
            error!(?emit_err, "Failed to emit shortcut event");
        }
    }
}
