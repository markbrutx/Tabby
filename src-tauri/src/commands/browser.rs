use tauri::{Emitter, Manager, Webview};
use tracing::{info, warn};

use crate::domain::error::TabbyError;
use crate::domain::events::{BrowserUrlChangedEvent, BROWSER_URL_CHANGED};

fn webview_label(pane_id: &str) -> String {
    format!("browser-{pane_id}")
}

fn find_webview(window: &tauri::Window, pane_id: &str) -> Result<Webview, TabbyError> {
    let label = webview_label(pane_id);
    window
        .app_handle()
        .webview_windows()
        .get(&label)
        .map(|ww| ww.as_ref().clone())
        .ok_or_else(|| TabbyError::NotFound(format!("Browser webview not found: {label}")))
}

#[tauri::command]
#[specta::specta]
pub fn create_browser_webview(
    window: tauri::Window,
    pane_id: String,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), TabbyError> {
    let label = webview_label(&pane_id);

    let parsed_url: url::Url = url
        .parse()
        .map_err(|err| TabbyError::Validation(format!("Invalid URL: {err}")))?;

    let webview_url = tauri::WebviewUrl::External(parsed_url);

    let app_handle = window.app_handle().clone();
    let pane_id_for_nav = pane_id.clone();

    let builder = tauri::webview::WebviewBuilder::new(&label, webview_url).on_navigation(
        move |nav_url: &url::Url| {
            let event = BrowserUrlChangedEvent {
                pane_id: pane_id_for_nav.clone(),
                url: nav_url.to_string(),
            };
            if let Err(err) = app_handle.emit(BROWSER_URL_CHANGED, event) {
                warn!(?err, "Failed to emit browser-url-changed");
            }
            true
        },
    );

    let position = tauri::Position::Logical(tauri::LogicalPosition::new(x, y));
    let size = tauri::Size::Logical(tauri::LogicalSize::new(width, height));

    window
        .add_child(builder, position, size)
        .map_err(|err| TabbyError::Io(format!("Failed to create webview: {err}")))?;

    info!(label, "Browser webview created");
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn navigate_browser(
    window: tauri::Window,
    pane_id: String,
    url: String,
) -> Result<(), TabbyError> {
    let webview = find_webview(&window, &pane_id)?;

    let parsed_url: url::Url = url
        .parse()
        .map_err(|err| TabbyError::Validation(format!("Invalid URL: {err}")))?;

    webview
        .navigate(parsed_url)
        .map_err(|err| TabbyError::Io(format!("Failed to navigate: {err}")))?;

    info!(pane_id, url, "Browser navigated");
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn close_browser_webview(window: tauri::Window, pane_id: String) -> Result<(), TabbyError> {
    let label = webview_label(&pane_id);

    match window.app_handle().webview_windows().get(&label) {
        Some(ww) => {
            if let Err(err) = ww.close() {
                warn!(?err, label, "Failed to close browser webview");
            }
            info!(label, "Browser webview closed");
        }
        None => {
            // Idempotent — already gone
        }
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_browser_webview_bounds(
    window: tauri::Window,
    pane_id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), TabbyError> {
    let webview = find_webview(&window, &pane_id)?;

    webview
        .set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)))
        .map_err(|err| TabbyError::Io(format!("Failed to set position: {err}")))?;

    webview
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)))
        .map_err(|err| TabbyError::Io(format!("Failed to set size: {err}")))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_browser_webview_visible(
    window: tauri::Window,
    pane_id: String,
    visible: bool,
) -> Result<(), TabbyError> {
    let label = webview_label(&pane_id);

    match window.app_handle().webview_windows().get(&label) {
        Some(ww) => {
            let result = if visible { ww.show() } else { ww.hide() };
            result.map_err(|err| TabbyError::Io(format!("Failed to set visibility: {err}")))?;
        }
        None => {
            // Webview not found — no-op (may have been closed)
        }
    }

    Ok(())
}
