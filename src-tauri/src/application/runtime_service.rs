use std::sync::Mutex;

use tauri::{AppHandle, Manager};
use tracing::warn;

use crate::application::commands::RuntimeCommand;
use tabby_runtime::{PaneRuntime, RuntimeRegistry, RuntimeStatus};
use tabby_settings::{resolve_terminal_profile, SettingsError, UserPreferences};
use tabby_workspace::PaneSpec;

use crate::application::{
    ProjectionPublisher, SettingsApplicationService, WorkspaceApplicationService,
};
use crate::shell::browser_surface;
use crate::shell::error::ShellError;
use crate::shell::pty::PtyManager;

#[derive(Debug)]
pub struct RuntimeApplicationService {
    app: AppHandle,
    runtimes: Mutex<RuntimeRegistry>,
    pty_manager: PtyManager,
    publisher: ProjectionPublisher,
}

impl RuntimeApplicationService {
    pub fn new(app: AppHandle) -> Self {
        Self {
            runtimes: Mutex::new(RuntimeRegistry::default()),
            pty_manager: PtyManager::new(app.clone()),
            publisher: ProjectionPublisher::new(app.clone()),
            app,
        }
    }

    pub fn start_runtime(
        &self,
        pane_id: &str,
        spec: &PaneSpec,
        preferences: &UserPreferences,
    ) -> Result<(), ShellError> {
        let runtime = match spec {
            PaneSpec::Terminal(spec) => {
                let resolved = resolve_terminal_profile(
                    &spec.launch_profile_id,
                    spec.command_override.clone(),
                    &preferences.default_custom_command,
                )
                .map_err(settings_error_to_shell)?;
                let runtime_session_id = self.pty_manager.spawn(
                    pane_id,
                    &spec.working_directory,
                    resolved.command.as_deref(),
                )?;
                self.lock_runtimes()?
                    .register_terminal(pane_id, runtime_session_id)
            }
            PaneSpec::Browser(spec) => self.lock_runtimes()?.register_browser(
                pane_id,
                format!("browser-{}", uuid::Uuid::new_v4()),
                spec.initial_url.clone(),
            ),
        };
        self.publisher.emit_runtime_status(&runtime);
        Ok(())
    }

    pub fn stop_runtime(&self, pane_id: &str) {
        let runtime = match self.lock_runtimes() {
            Ok(mut runtimes) => runtimes.remove(pane_id),
            Err(error) => {
                warn!(?error, "Failed to lock runtime registry during stop");
                return;
            }
        };

        let Some(runtime) = runtime else {
            return;
        };

        if let Some(runtime_session_id) = runtime.runtime_session_id.clone() {
            match runtime.kind {
                tabby_runtime::RuntimeKind::Terminal => {
                    if let Err(error) = self.pty_manager.kill(&runtime_session_id) {
                        warn!(?error, pane_id, "Failed to kill terminal runtime");
                    }
                }
                tabby_runtime::RuntimeKind::Browser => {
                    if let Some(window) = self.app.get_webview_window("main") {
                        if let Some(webview) =
                            window.get_webview(&browser_surface::webview_label(pane_id))
                        {
                            if let Err(error) = webview.close() {
                                warn!(?error, pane_id, "Failed to close browser surface");
                            }
                        }
                    }
                }
            }
        }

        let mut exited = runtime;
        exited.status = RuntimeStatus::Exited;
        self.publisher.emit_runtime_status(&exited);
    }

    pub fn restart_runtime(
        &self,
        pane_id: &str,
        spec: &PaneSpec,
        preferences: &UserPreferences,
    ) -> Result<(), ShellError> {
        self.stop_runtime(pane_id);
        self.start_runtime(pane_id, spec, preferences)
    }

    pub fn dispatch_runtime_command(
        &self,
        window: &tauri::Window,
        command: RuntimeCommand,
    ) -> Result<(), ShellError> {
        match command {
            RuntimeCommand::WriteTerminalInput { pane_id, input } => {
                let runtime_session_id = self
                    .lock_runtimes()?
                    .terminal_session_id(&pane_id)
                    .ok_or_else(|| ShellError::NotFound(format!("runtime for pane {pane_id}")))?;
                self.pty_manager.write(&runtime_session_id, &input)?;
            }
            RuntimeCommand::ResizeTerminal {
                pane_id,
                cols,
                rows,
            } => {
                let runtime_session_id = self
                    .lock_runtimes()?
                    .terminal_session_id(&pane_id)
                    .ok_or_else(|| ShellError::NotFound(format!("runtime for pane {pane_id}")))?;
                self.pty_manager.resize(&runtime_session_id, cols, rows)?;
            }
            RuntimeCommand::NavigateBrowser { pane_id, url } => {
                browser_surface::navigate_browser(window, &pane_id, &url)?;
                let maybe_runtime = self
                    .lock_runtimes()?
                    .update_browser_location(&pane_id, url)
                    .ok();
                if let Some(runtime) = maybe_runtime {
                    self.publisher.emit_runtime_status(&runtime);
                }
            }
            RuntimeCommand::ObserveTerminalCwd { .. }
            | RuntimeCommand::ObserveBrowserLocation { .. } => {
                // Observation commands are handled by the shell layer via
                // observe_terminal_cwd / observe_browser_location methods.
                // They should not reach dispatch_runtime_command.
                return Err(ShellError::Validation(String::from(
                    "Observation commands must be routed through the shell layer",
                )));
            }
        }

        Ok(())
    }

    pub fn observe_terminal_cwd(
        &self,
        pane_id: &str,
        working_directory: &str,
        workspace_service: &WorkspaceApplicationService,
        settings_service: &SettingsApplicationService,
    ) -> Result<(), ShellError> {
        workspace_service.track_terminal_working_directory(pane_id, working_directory)?;
        let mut preferences = settings_service.preferences()?;
        preferences.last_working_directory = Some(String::from(working_directory));
        settings_service.persist_preferences(&preferences)?;
        Ok(())
    }

    pub fn observe_browser_location(&self, pane_id: &str, url: &str) -> Result<(), ShellError> {
        let maybe_runtime = self
            .lock_runtimes()?
            .update_browser_location(pane_id, String::from(url))
            .ok();
        if let Some(runtime) = maybe_runtime {
            self.publisher.emit_runtime_status(&runtime);
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Result<Vec<PaneRuntime>, ShellError> {
        Ok(self.lock_runtimes()?.snapshot().to_vec())
    }

    fn lock_runtimes(&self) -> Result<std::sync::MutexGuard<'_, RuntimeRegistry>, ShellError> {
        self.runtimes
            .lock()
            .map_err(|_| ShellError::State(String::from("Runtime lock poisoned")))
    }
}

fn settings_error_to_shell(error: SettingsError) -> ShellError {
    match error {
        SettingsError::Validation(message) => ShellError::Validation(message),
    }
}

#[cfg(test)]
mod tests {
    use tabby_runtime::{RuntimeKind, RuntimeRegistry, RuntimeStatus};

    /// Tests the registry interactions that underpin `start_runtime`.
    /// Verifies that registering a terminal runtime makes it visible in the snapshot
    /// and correctly tracks the runtime session ID.
    #[test]
    fn start_runtime_registers_terminal_in_registry() {
        let mut registry = RuntimeRegistry::default();

        let runtime = registry.register_terminal("pane-1", String::from("pty-session-abc"));

        assert_eq!(runtime.pane_id, "pane-1");
        assert_eq!(
            runtime.runtime_session_id,
            Some(String::from("pty-session-abc"))
        );
        assert!(matches!(runtime.kind, RuntimeKind::Terminal));
        assert!(matches!(runtime.status, RuntimeStatus::Running));

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].pane_id, "pane-1");

        let session_id = registry.terminal_session_id("pane-1");
        assert_eq!(session_id, Some(String::from("pty-session-abc")));
    }

    /// Tests the registry interactions that underpin `stop_runtime`.
    /// Verifies that removing a runtime clears it from the registry and
    /// returns the runtime data for cleanup (PTY kill, status emit).
    #[test]
    fn stop_runtime_removes_from_registry() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal("pane-1", String::from("pty-session-abc"));
        registry.register_browser(
            "pane-2",
            String::from("browser-xyz"),
            String::from("https://example.com"),
        );

        assert_eq!(registry.snapshot().len(), 2);

        let removed = registry.remove("pane-1");
        assert!(removed.is_some());
        let removed = removed.unwrap(); // safe in test
        assert_eq!(removed.pane_id, "pane-1");
        assert!(matches!(removed.kind, RuntimeKind::Terminal));

        assert_eq!(registry.snapshot().len(), 1);
        assert_eq!(registry.snapshot()[0].pane_id, "pane-2");

        let not_found = registry.remove("pane-1");
        assert!(not_found.is_none(), "removing twice should return None");
    }

    /// Tests browser runtime registration that underpins `start_runtime` for browser panes.
    #[test]
    fn start_runtime_registers_browser_in_registry() {
        let mut registry = RuntimeRegistry::default();

        let runtime = registry.register_browser(
            "pane-b",
            String::from("browser-session-1"),
            String::from("https://example.com"),
        );

        assert_eq!(runtime.pane_id, "pane-b");
        assert!(matches!(runtime.kind, RuntimeKind::Browser));
        assert!(matches!(runtime.status, RuntimeStatus::Running));
        assert_eq!(
            runtime.browser_location,
            Some(String::from("https://example.com"))
        );
    }
}
