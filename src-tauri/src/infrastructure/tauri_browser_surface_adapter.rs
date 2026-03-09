use tauri::{AppHandle, Manager};

use tabby_contracts::{BrowserSurfaceBoundsDto, BrowserSurfaceCommandDto};

use crate::application::ports::BrowserSurfacePort;
use crate::shell::browser_surface;
use crate::shell::error::ShellError;

#[derive(Debug)]
pub struct TauriBrowserSurfaceAdapter {
    app: AppHandle,
}

impl TauriBrowserSurfaceAdapter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn main_window(&self) -> Result<tauri::Window, ShellError> {
        self.app
            .get_window("main")
            .ok_or_else(|| ShellError::NotFound(String::from("main window")))
    }

    fn bounds_dto(x: f64, y: f64, width: f64, height: f64) -> BrowserSurfaceBoundsDto {
        BrowserSurfaceBoundsDto {
            x,
            y,
            width,
            height,
        }
    }
}

impl BrowserSurfacePort for TauriBrowserSurfaceAdapter {
    fn ensure_surface(
        &self,
        pane_id: &str,
        url: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<(), ShellError> {
        let window = self.main_window()?;
        let command = BrowserSurfaceCommandDto::Ensure {
            pane_id: String::from(pane_id),
            url: String::from(url),
            bounds: Self::bounds_dto(x, y, width, height),
        };
        browser_surface::execute_browser_surface_command(&window, command)
    }

    fn set_bounds(
        &self,
        pane_id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<(), ShellError> {
        let window = self.main_window()?;
        let command = BrowserSurfaceCommandDto::SetBounds {
            pane_id: String::from(pane_id),
            bounds: Self::bounds_dto(x, y, width, height),
        };
        browser_surface::execute_browser_surface_command(&window, command)
    }

    fn set_visible(&self, pane_id: &str, visible: bool) -> Result<(), ShellError> {
        let window = self.main_window()?;
        let command = BrowserSurfaceCommandDto::SetVisible {
            pane_id: String::from(pane_id),
            visible,
        };
        browser_surface::execute_browser_surface_command(&window, command)
    }

    fn close_surface(&self, pane_id: &str) -> Result<(), ShellError> {
        let window = self.main_window()?;
        browser_surface::close_browser_surface(&window, pane_id)
    }

    fn navigate(&self, pane_id: &str, url: &str) -> Result<(), ShellError> {
        let window = self.main_window()?;
        browser_surface::navigate_browser(&window, pane_id, url)
    }
}
