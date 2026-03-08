use tauri::{Emitter, Manager, Webview};
use tracing::{info, warn};

use tabby_contracts::{
    BrowserLocationObservedEvent, BrowserSurfaceBoundsDto, BrowserSurfaceCommandDto,
};

use crate::shell::error::ShellError;
use crate::shell::BROWSER_LOCATION_OBSERVED_EVENT;

pub fn execute_browser_surface_command(
    window: &tauri::Window,
    command: BrowserSurfaceCommandDto,
) -> Result<(), ShellError> {
    match command {
        BrowserSurfaceCommandDto::Ensure {
            pane_id,
            url,
            bounds,
        } => ensure_browser_surface(window, &pane_id, &url, &bounds),
        BrowserSurfaceCommandDto::SetBounds { pane_id, bounds } => {
            set_browser_surface_bounds(window, &pane_id, &bounds)
        }
        BrowserSurfaceCommandDto::SetVisible { pane_id, visible } => {
            set_browser_surface_visible(window, &pane_id, visible)
        }
        BrowserSurfaceCommandDto::Close { pane_id } => close_browser_surface(window, &pane_id),
    }
}

pub fn navigate_browser(
    window: &tauri::Window,
    pane_id: &str,
    url: &str,
) -> Result<(), ShellError> {
    let webview = find_webview(window, pane_id)?;
    let parsed_url: url::Url = url
        .parse()
        .map_err(|error| ShellError::Validation(format!("Invalid URL: {error}")))?;
    webview
        .navigate(parsed_url)
        .map_err(|error| ShellError::Io(format!("Failed to navigate browser: {error}")))?;
    info!(pane_id, url, "Browser navigated");
    Ok(())
}

pub fn close_browser_surface(window: &tauri::Window, pane_id: &str) -> Result<(), ShellError> {
    let label = webview_label(pane_id);
    match window.get_webview(&label) {
        Some(webview) => {
            if let Err(error) = webview.close() {
                warn!(?error, label, "Failed to close browser surface");
            }
        }
        None => {}
    }
    Ok(())
}

pub fn webview_label(pane_id: &str) -> String {
    format!("browser-{pane_id}")
}

fn ensure_browser_surface(
    window: &tauri::Window,
    pane_id: &str,
    url: &str,
    bounds: &BrowserSurfaceBoundsDto,
) -> Result<(), ShellError> {
    let label = webview_label(pane_id);
    let parsed_url: url::Url = url
        .parse()
        .map_err(|error| ShellError::Validation(format!("Invalid URL: {error}")))?;
    let position = child_webview_position(window, bounds.x, bounds.y)?;
    let size = tauri::Size::Logical(tauri::LogicalSize::new(bounds.width, bounds.height));

    if let Some(existing) = window.get_webview(&label) {
        existing
            .set_position(position)
            .map_err(|error| ShellError::Io(format!("Failed to set browser position: {error}")))?;
        existing
            .set_size(size)
            .map_err(|error| ShellError::Io(format!("Failed to set browser size: {error}")))?;
        existing
            .show()
            .map_err(|error| ShellError::Io(format!("Failed to show browser surface: {error}")))?;
        return Ok(());
    }

    let app = window.app_handle().clone();
    let pane_id_for_nav = String::from(pane_id);
    let builder =
        tauri::webview::WebviewBuilder::new(&label, tauri::WebviewUrl::External(parsed_url))
            .on_navigation(move |next_url: &url::Url| {
                let event = BrowserLocationObservedEvent {
                    pane_id: pane_id_for_nav.clone(),
                    url: next_url.to_string(),
                };
                if let Err(error) = app.emit(BROWSER_LOCATION_OBSERVED_EVENT, event) {
                    warn!(?error, "Failed to emit browser-location-observed");
                }
                true
            });

    window
        .add_child(builder, position, size)
        .map_err(|error| ShellError::Io(format!("Failed to create browser surface: {error}")))?;
    info!(pane_id, "Browser surface created");
    Ok(())
}

fn set_browser_surface_bounds(
    window: &tauri::Window,
    pane_id: &str,
    bounds: &BrowserSurfaceBoundsDto,
) -> Result<(), ShellError> {
    let webview = find_webview(window, pane_id)?;
    let position = child_webview_position(window, bounds.x, bounds.y)?;
    webview
        .set_position(position)
        .map_err(|error| ShellError::Io(format!("Failed to set browser position: {error}")))?;
    webview
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(
            bounds.width,
            bounds.height,
        )))
        .map_err(|error| ShellError::Io(format!("Failed to set browser size: {error}")))?;
    Ok(())
}

fn set_browser_surface_visible(
    window: &tauri::Window,
    pane_id: &str,
    visible: bool,
) -> Result<(), ShellError> {
    match window.get_webview(&webview_label(pane_id)) {
        Some(webview) => {
            let result = if visible {
                webview.show()
            } else {
                webview.hide()
            };
            result.map_err(|error| {
                ShellError::Io(format!("Failed to change browser visibility: {error}"))
            })?;
        }
        None => {}
    }
    Ok(())
}

fn find_webview(window: &tauri::Window, pane_id: &str) -> Result<Webview, ShellError> {
    window
        .get_webview(&webview_label(pane_id))
        .ok_or_else(|| ShellError::NotFound(format!("browser webview for pane {pane_id}")))
}

fn child_webview_position(
    window: &tauri::Window,
    x: f64,
    y: f64,
) -> Result<tauri::Position, ShellError> {
    let scale_factor = window
        .scale_factor()
        .map_err(|error| ShellError::Io(format!("Failed to read scale factor: {error}")))?;
    let inner_position = window
        .inner_position()
        .map_err(|error| ShellError::Io(format!("Failed to read inner position: {error}")))?;
    let outer_position = window
        .outer_position()
        .map_err(|error| ShellError::Io(format!("Failed to read outer position: {error}")))?;
    let inset_x = f64::from(inner_position.x - outer_position.x) / scale_factor;
    let inset_y = f64::from(inner_position.y - outer_position.y) / scale_factor;

    Ok(tauri::Position::Logical(tauri::LogicalPosition::new(
        x + inset_x,
        y + inset_y,
    )))
}
