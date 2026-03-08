use std::sync::Arc;

use tabby_runtime::PaneRuntime;
use tabby_settings::UserPreferences;

use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
use crate::shell::error::ShellError;

/// Port for persisting and loading user preferences.
///
/// Infrastructure adapters implement this trait to decouple
/// `SettingsApplicationService` from any specific storage backend.
pub trait PreferencesRepository: Send + Sync + std::fmt::Debug {
    /// Load persisted preferences, or `None` if no preferences have been saved yet.
    fn load(&self) -> Result<Option<serde_json::Value>, ShellError>;

    /// Persist the given preferences.
    fn save(&self, preferences: &UserPreferences) -> Result<(), ShellError>;
}

/// Port for managing terminal process (PTY) lifecycle.
///
/// Infrastructure adapters implement this trait to decouple
/// `RuntimeApplicationService` from any specific PTY backend.
pub trait TerminalProcessPort: Send + Sync + std::fmt::Debug {
    /// Spawn a new terminal process and return the runtime session ID.
    fn spawn(
        &self,
        pane_id: &str,
        working_directory: &str,
        startup_command: Option<&str>,
        observation_receiver: Arc<dyn RuntimeObservationReceiver>,
    ) -> Result<String, ShellError>;

    /// Terminate a terminal process by its runtime session ID.
    fn kill(&self, runtime_session_id: &str) -> Result<(), ShellError>;

    /// Resize a terminal process by its runtime session ID.
    fn resize(&self, runtime_session_id: &str, cols: u16, rows: u16) -> Result<(), ShellError>;

    /// Write user input to a terminal process by its runtime session ID.
    fn write_input(&self, runtime_session_id: &str, data: &str) -> Result<(), ShellError>;
}

/// Port for managing browser surface (webview) lifecycle.
///
/// Infrastructure adapters implement this trait to decouple
/// `RuntimeApplicationService` from any specific webview backend.
///
/// Some methods (`ensure_surface`, `set_bounds`, `set_visible`) are not yet
/// called from application code — they will be wired in a future story when
/// the browser surface command handler is routed through RuntimeApplicationService.
#[allow(dead_code)]
pub trait BrowserSurfacePort: Send + Sync + std::fmt::Debug {
    /// Ensure a browser surface exists for the given pane, creating it if needed.
    fn ensure_surface(
        &self,
        pane_id: &str,
        url: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<(), ShellError>;

    /// Update the position and size of an existing browser surface.
    fn set_bounds(
        &self,
        pane_id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<(), ShellError>;

    /// Show or hide a browser surface.
    fn set_visible(&self, pane_id: &str, visible: bool) -> Result<(), ShellError>;

    /// Close and destroy a browser surface.
    fn close_surface(&self, pane_id: &str) -> Result<(), ShellError>;

    /// Navigate an existing browser surface to a new URL.
    fn navigate(&self, pane_id: &str, url: &str) -> Result<(), ShellError>;
}

/// Port for emitting runtime status projections to the frontend.
///
/// Infrastructure adapters implement this trait to decouple
/// `RuntimeApplicationService` from any specific event transport.
pub trait RuntimeProjectionEmitter: Send + Sync + std::fmt::Debug {
    /// Emit a runtime status change event for a single pane runtime.
    fn emit_runtime_status(&self, runtime: &PaneRuntime);
}
