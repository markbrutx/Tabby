use tauri::{Emitter, Manager, Webview};
use tracing::{info, warn};

use crate::shared::error::TabbyError;
use crate::shared::events::{BrowserUrlChangedEvent, BROWSER_URL_CHANGED};

pub(crate) fn webview_label(pane_id: &str) -> String {
    format!("browser-{pane_id}")
}

fn find_webview(window: &tauri::Window, pane_id: &str) -> Result<Webview, TabbyError> {
    let label = webview_label(pane_id);
    window
        .get_webview(&label)
        .ok_or_else(|| TabbyError::NotFound(format!("Browser webview not found: {label}")))
}

fn logical_child_position(
    x: f64,
    y: f64,
    scale_factor: f64,
    inner_position: tauri::PhysicalPosition<i32>,
    outer_position: tauri::PhysicalPosition<i32>,
) -> tauri::LogicalPosition<f64> {
    let inset_x = f64::from(inner_position.x - outer_position.x) / scale_factor;
    let inset_y = f64::from(inner_position.y - outer_position.y) / scale_factor;

    tauri::LogicalPosition::new(x + inset_x, y + inset_y)
}

fn child_webview_position(
    window: &tauri::Window,
    x: f64,
    y: f64,
) -> Result<tauri::Position, TabbyError> {
    let scale_factor = window
        .scale_factor()
        .map_err(|err| TabbyError::Io(format!("Failed to read scale factor: {err}")))?;
    let inner_position = window
        .inner_position()
        .map_err(|err| TabbyError::Io(format!("Failed to read inner position: {err}")))?;
    let outer_position = window
        .outer_position()
        .map_err(|err| TabbyError::Io(format!("Failed to read outer position: {err}")))?;

    Ok(tauri::Position::Logical(logical_child_position(
        x,
        y,
        scale_factor,
        inner_position,
        outer_position,
    )))
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

    let position = child_webview_position(&window, x, y)?;
    let size = tauri::Size::Logical(tauri::LogicalSize::new(width, height));

    if let Some(existing) = window.get_webview(&label) {
        existing
            .set_position(position)
            .map_err(|err| TabbyError::Io(format!("Failed to set position: {err}")))?;

        existing
            .set_size(size)
            .map_err(|err| TabbyError::Io(format!("Failed to set size: {err}")))?;

        existing
            .show()
            .map_err(|err| TabbyError::Io(format!("Failed to show webview: {err}")))?;

        info!(label, "Browser webview reused");
        return Ok(());
    }

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

    match window.get_webview(&label) {
        Some(wv) => {
            if let Err(err) = wv.close() {
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

    let position = child_webview_position(&window, x, y)?;

    webview
        .set_position(position)
        .map_err(|err| TabbyError::Io(format!("Failed to set position: {err}")))?;

    webview
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)))
        .map_err(|err| TabbyError::Io(format!("Failed to set size: {err}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{logical_child_position, webview_label};

    #[test]
    fn webview_label_format() {
        assert_eq!(webview_label("pane-abc-123"), "browser-pane-abc-123");
        assert_eq!(webview_label(""), "browser-");
    }

    #[test]
    fn adjusts_child_position_for_window_chrome() {
        let position = logical_child_position(
            24.0,
            32.0,
            2.0,
            tauri::PhysicalPosition::new(300, 228),
            tauri::PhysicalPosition::new(280, 180),
        );

        assert_eq!(position.x, 34.0);
        assert_eq!(position.y, 56.0);
    }
}

#[tauri::command]
#[specta::specta]
pub fn set_browser_webview_visible(
    window: tauri::Window,
    pane_id: String,
    visible: bool,
) -> Result<(), TabbyError> {
    let label = webview_label(&pane_id);

    match window.get_webview(&label) {
        Some(wv) => {
            let result = if visible { wv.show() } else { wv.hide() };
            result.map_err(|err| TabbyError::Io(format!("Failed to set visibility: {err}")))?;
        }
        None => {
            // Webview not found — no-op (may have been closed)
        }
    }

    Ok(())
}
