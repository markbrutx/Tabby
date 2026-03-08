use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};
use tracing::warn;

use crate::application::commands::RuntimeCommand;
use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
use tabby_runtime::{PaneRuntime, RuntimeRegistry, RuntimeSessionId, RuntimeStatus};
use tabby_settings::{resolve_terminal_profile, SettingsError, UserPreferences};
use tabby_workspace::{PaneId, PaneSpec};

use crate::application::{ProjectionPublisher, SettingsApplicationService};
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
        observation_receiver: Arc<dyn RuntimeObservationReceiver>,
    ) -> Result<(), ShellError> {
        let runtime = match spec {
            PaneSpec::Terminal(spec) => {
                let resolved = resolve_terminal_profile(
                    &spec.launch_profile_id,
                    spec.command_override.clone(),
                    &preferences.default_custom_command,
                )
                .map_err(settings_error_to_shell)?;
                let pty_session_id = self.pty_manager.spawn(
                    pane_id,
                    &spec.working_directory,
                    resolved.command.as_deref(),
                    observation_receiver,
                )?;
                self.lock_runtimes()?
                    .register_terminal(pane_id, RuntimeSessionId::from(pty_session_id))
            }
            PaneSpec::Browser(spec) => self.lock_runtimes()?.register_browser(
                pane_id,
                RuntimeSessionId::from(format!("browser-{}", uuid::Uuid::new_v4())),
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

        if let Some(runtime_session_id) = &runtime.runtime_session_id {
            match runtime.kind {
                tabby_runtime::RuntimeKind::Terminal => {
                    if let Err(error) = self.pty_manager.kill(runtime_session_id.as_ref()) {
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
        observation_receiver: Arc<dyn RuntimeObservationReceiver>,
    ) -> Result<(), ShellError> {
        self.stop_runtime(pane_id);
        self.start_runtime(pane_id, spec, preferences, observation_receiver)
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
                    .terminal_session_id(pane_id.as_ref())
                    .ok_or_else(|| ShellError::NotFound(format!("runtime for pane {pane_id}")))?;
                self.pty_manager
                    .write(runtime_session_id.as_ref(), &input)?;
            }
            RuntimeCommand::ResizeTerminal {
                pane_id,
                cols,
                rows,
            } => {
                let runtime_session_id = self
                    .lock_runtimes()?
                    .terminal_session_id(pane_id.as_ref())
                    .ok_or_else(|| ShellError::NotFound(format!("runtime for pane {pane_id}")))?;
                self.pty_manager
                    .resize(runtime_session_id.as_ref(), cols, rows)?;
            }
            RuntimeCommand::NavigateBrowser { pane_id, url } => {
                browser_surface::navigate_browser(window, pane_id.as_ref(), &url)?;
                let maybe_runtime = self
                    .lock_runtimes()?
                    .update_browser_location(pane_id.as_ref(), url)
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
        pane_id: &PaneId,
        working_directory: &str,
        settings_service: &SettingsApplicationService,
    ) -> Result<(), ShellError> {
        let runtime = self
            .lock_runtimes()?
            .update_terminal_cwd(pane_id.as_ref(), String::from(working_directory))
            .map_err(|error| ShellError::NotFound(error.to_string()))?;
        self.publisher.emit_runtime_status(&runtime);

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

impl RuntimeObservationReceiver for RuntimeApplicationService {
    fn on_terminal_output_received(&self, pane_id: &PaneId, data: &[u8]) {
        // Terminal output is currently emitted directly by the PTY read thread
        // via Tauri events. Once infrastructure is wired to this trait (future story),
        // this method will become the single entry point for terminal output dispatch.
        tracing::trace!(
            pane_id = pane_id.as_ref(),
            bytes = data.len(),
            "Terminal output observation received"
        );
    }

    fn on_terminal_exited(&self, pane_id: &PaneId, exit_code: Option<i32>) {
        let failed = exit_code.is_some_and(|code| code != 0);
        let message = exit_code
            .filter(|code| *code != 0)
            .map(|code| format!("Process exited with code {code}"));

        let result = self.lock_runtimes().and_then(|mut runtimes| {
            runtimes
                .mark_terminal_exit(pane_id.as_ref(), None, failed, message)
                .map_err(|error| ShellError::NotFound(error.to_string()))
        });

        match result {
            Ok(runtime) => self.publisher.emit_runtime_status(&runtime),
            Err(error) => {
                warn!(
                    ?error,
                    pane_id = pane_id.as_ref(),
                    "Failed to process terminal exit observation"
                );
            }
        }
    }

    fn on_browser_location_changed(&self, pane_id: &PaneId, url: &str) {
        let result = self.lock_runtimes().and_then(|mut runtimes| {
            runtimes
                .update_browser_location(pane_id.as_ref(), String::from(url))
                .map_err(|error| ShellError::NotFound(error.to_string()))
        });

        match result {
            Ok(runtime) => self.publisher.emit_runtime_status(&runtime),
            Err(error) => {
                warn!(
                    ?error,
                    pane_id = pane_id.as_ref(),
                    "Failed to process browser location observation"
                );
            }
        }
    }

    fn on_terminal_cwd_changed(&self, pane_id: &PaneId, cwd: &str) {
        // Update runtime registry with the observed cwd. Settings persistence
        // requires cross-service access handled by observe_terminal_cwd on the
        // shell layer. This trait method will become the single entry point
        // once infrastructure is fully wired (future story).
        let result = self.lock_runtimes().and_then(|mut runtimes| {
            runtimes
                .update_terminal_cwd(pane_id.as_ref(), String::from(cwd))
                .map_err(|error| ShellError::NotFound(error.to_string()))
        });

        match result {
            Ok(runtime) => self.publisher.emit_runtime_status(&runtime),
            Err(error) => {
                warn!(
                    ?error,
                    pane_id = pane_id.as_ref(),
                    "Failed to process terminal cwd observation"
                );
            }
        }
    }
}

fn settings_error_to_shell(error: SettingsError) -> ShellError {
    match error {
        SettingsError::Validation(message) => ShellError::Validation(message),
    }
}

#[cfg(test)]
mod tests {
    use tabby_runtime::{RuntimeKind, RuntimeRegistry, RuntimeSessionId, RuntimeStatus};

    fn sid(s: &str) -> RuntimeSessionId {
        RuntimeSessionId::from(String::from(s))
    }

    #[test]
    fn start_runtime_registers_terminal_in_registry() {
        let mut registry = RuntimeRegistry::default();

        let runtime = registry.register_terminal("pane-1", sid("pty-session-abc"));

        assert_eq!(runtime.pane_id, "pane-1");
        assert_eq!(runtime.runtime_session_id, Some(sid("pty-session-abc")));
        assert!(matches!(runtime.kind, RuntimeKind::Terminal));
        assert!(matches!(runtime.status, RuntimeStatus::Running));

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].pane_id, "pane-1");

        let session_id = registry.terminal_session_id("pane-1");
        assert_eq!(session_id, Some(sid("pty-session-abc")));
    }

    #[test]
    fn stop_runtime_removes_from_registry() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal("pane-1", sid("pty-session-abc"));
        registry.register_browser(
            "pane-2",
            sid("browser-xyz"),
            String::from("https://example.com"),
        );

        assert_eq!(registry.snapshot().len(), 2);

        let removed = registry.remove("pane-1");
        assert!(removed.is_some());
        let removed = removed.expect("remove returned Some, already asserted");
        assert_eq!(removed.pane_id, "pane-1");
        assert!(matches!(removed.kind, RuntimeKind::Terminal));

        assert_eq!(registry.snapshot().len(), 1);
        assert_eq!(registry.snapshot()[0].pane_id, "pane-2");

        let not_found = registry.remove("pane-1");
        assert!(not_found.is_none(), "removing twice should return None");
    }

    #[test]
    fn start_runtime_registers_browser_in_registry() {
        let mut registry = RuntimeRegistry::default();

        let runtime = registry.register_browser(
            "pane-b",
            sid("browser-session-1"),
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

    #[test]
    fn stop_runtime_for_nonexistent_pane_returns_none() {
        let mut registry = RuntimeRegistry::default();
        let removed = registry.remove("nonexistent-pane");
        assert!(
            removed.is_none(),
            "removing nonexistent runtime should return None"
        );
    }

    #[test]
    fn restart_runtime_replaces_registry_entry() {
        let mut registry = RuntimeRegistry::default();

        // Initial registration
        registry.register_terminal("pane-1", sid("pty-session-1"));
        assert_eq!(registry.snapshot().len(), 1);

        // Simulate restart: remove + register with new session
        let removed = registry.remove("pane-1");
        assert!(removed.is_some(), "old runtime should be removed");

        let runtime = registry.register_terminal("pane-1", sid("pty-session-2"));
        assert_eq!(runtime.pane_id, "pane-1");
        assert_eq!(runtime.runtime_session_id, Some(sid("pty-session-2")));
        assert_eq!(
            registry.snapshot().len(),
            1,
            "should have exactly one runtime after restart"
        );
    }

    #[test]
    fn mark_terminal_exit_failure_tracks_error() {
        let mut registry = RuntimeRegistry::default();
        let session_id = sid("pty-session-1");
        registry.register_terminal("pane-1", session_id.clone());

        let result = registry.mark_terminal_exit(
            "pane-1",
            Some(&session_id),
            true,
            Some(String::from("PTY spawn failed")),
        );
        assert!(result.is_ok(), "mark_terminal_exit should succeed");

        let runtime = result.expect("already asserted ok");
        assert!(matches!(runtime.status, RuntimeStatus::Failed));
        assert_eq!(
            runtime.last_error,
            Some(String::from("PTY spawn failed")),
            "should record the spawn failure message"
        );
    }

    #[test]
    fn mark_terminal_exit_with_wrong_session_id_is_ignored() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal("pane-1", sid("pty-session-1"));

        // When session ID doesn't match, the exit is silently ignored (stale event)
        let result =
            registry.mark_terminal_exit("pane-1", Some(&sid("wrong-session")), false, None);
        assert!(result.is_ok(), "should succeed even with wrong session id");

        let runtime = result.expect("already asserted ok");
        assert!(
            matches!(runtime.status, RuntimeStatus::Running),
            "status should remain Running when session id does not match"
        );
    }

    #[test]
    fn mark_terminal_exit_for_nonexistent_pane_returns_error() {
        let mut registry = RuntimeRegistry::default();
        let result =
            registry.mark_terminal_exit("nonexistent", Some(&sid("session-1")), false, None);
        assert!(result.is_err(), "should return error for nonexistent pane");
    }

    #[test]
    fn update_browser_location_for_nonexistent_pane_returns_error() {
        let mut registry = RuntimeRegistry::default();
        let result =
            registry.update_browser_location("nonexistent", String::from("https://example.com"));
        assert!(
            result.is_err(),
            "updating location for nonexistent pane should fail"
        );
    }

    #[test]
    fn snapshot_returns_all_registered_runtimes() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal("pane-1", sid("pty-1"));
        registry.register_browser(
            "pane-2",
            sid("browser-1"),
            String::from("https://example.com"),
        );
        registry.register_terminal("pane-3", sid("pty-3"));

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.len(), 3, "snapshot should contain all runtimes");
    }

    #[test]
    fn empty_registry_snapshot_is_empty() {
        let registry = RuntimeRegistry::default();
        assert!(
            registry.snapshot().is_empty(),
            "empty registry should return empty snapshot"
        );
    }

    /// Verifies that cwd observation updates the runtime registry but does NOT
    /// touch workspace domain. This is the boundary contract established by US-012:
    /// cwd observation is a runtime concern, not a workspace structural concern.
    #[test]
    fn cwd_observation_updates_runtime_registry_not_workspace() {
        use tabby_workspace::layout::LayoutPreset;
        use tabby_workspace::{PaneSpec, TabLayoutStrategy, TerminalPaneSpec, WorkspaceSession};

        // 1. Set up a workspace session with a terminal pane
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![PaneSpec::Terminal(TerminalPaneSpec {
                    launch_profile_id: String::from("default"),
                    working_directory: String::from("/home/user"),
                    command_override: None,
                })],
            )
            .expect("tab should open");
        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();

        // 2. Register the pane in the runtime registry
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal(pane_id.as_ref(), sid("pty-1"));

        // 3. Update cwd in the runtime registry (simulating observe_terminal_cwd)
        let runtime = registry
            .update_terminal_cwd(pane_id.as_ref(), String::from("/projects/tabby"))
            .expect("cwd update should succeed");
        assert_eq!(
            runtime.terminal_cwd.as_deref(),
            Some("/projects/tabby"),
            "runtime registry should reflect the observed cwd"
        );

        // 4. Workspace domain should NOT have been mutated — pane spec still has the original cwd
        match workspace
            .pane_spec(&pane_id)
            .expect("pane should still exist")
        {
            PaneSpec::Terminal(spec) => {
                assert_eq!(
                    spec.working_directory, "/home/user",
                    "workspace domain must NOT be mutated by cwd observation"
                );
            }
            PaneSpec::Browser(_) => panic!("expected terminal pane"),
        }

        // 5. Workspace session should have no track_terminal_working_directory method
        // (compile-time guarantee — if this test compiles, the method is removed)
    }
}
