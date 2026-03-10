use std::sync::Mutex;

use tracing::warn;

use tabby_contracts::BrowserSurfaceCommandDto;

use crate::application::commands::RuntimeCommand;
use crate::application::ports::{BrowserSurfacePort, ProjectionPublisherPort, TerminalProcessPort};
use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
use tabby_runtime::{PaneRuntime, RuntimeRegistry, RuntimeSessionId, RuntimeStatus};
use tabby_settings::{resolve_terminal_profile, SettingsError, UserPreferences};
use tabby_workspace::{PaneId, PaneSpec};

use crate::shell::error::ShellError;

pub struct RuntimeApplicationService {
    runtimes: Mutex<RuntimeRegistry>,
    terminal_port: Box<dyn TerminalProcessPort>,
    browser_port: Box<dyn BrowserSurfacePort>,
    emitter: Box<dyn ProjectionPublisherPort>,
}

impl std::fmt::Debug for RuntimeApplicationService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeApplicationService")
            .field("terminal_port", &self.terminal_port)
            .field("browser_port", &self.browser_port)
            .field("emitter", &self.emitter)
            .finish_non_exhaustive()
    }
}

impl RuntimeApplicationService {
    pub fn new(
        terminal_port: Box<dyn TerminalProcessPort>,
        browser_port: Box<dyn BrowserSurfacePort>,
        emitter: Box<dyn ProjectionPublisherPort>,
    ) -> Self {
        Self {
            runtimes: Mutex::new(RuntimeRegistry::default()),
            terminal_port,
            browser_port,
            emitter,
        }
    }

    pub fn start_runtime(
        &self,
        pane_id: &PaneId,
        spec: &PaneSpec,
        preferences: &UserPreferences,
        observation_receiver: std::sync::Arc<dyn RuntimeObservationReceiver>,
    ) -> Result<(), ShellError> {
        let runtime = match spec {
            PaneSpec::Terminal(spec) => {
                let resolved = resolve_terminal_profile(
                    &spec.launch_profile_id,
                    spec.command_override.clone(),
                    &preferences.default_custom_command,
                )
                .map_err(settings_error_to_shell)?;
                let pty_session_id = self.terminal_port.spawn(
                    pane_id.as_ref(),
                    &spec.working_directory,
                    resolved.command.as_ref().map(|c| c.as_str()),
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
            PaneSpec::Git(spec) => {
                let synthetic_session_id =
                    RuntimeSessionId::from(format!("git-{}", uuid::Uuid::new_v4()));
                let repo_path = tabby_kernel::WorkingDirectory::new(&spec.working_directory)
                    .map_err(|error| {
                        ShellError::Validation(format!("invalid git repo path: {error}"))
                    })?;
                self.lock_runtimes()?
                    .register_git(pane_id, synthetic_session_id, repo_path)
            }
        };
        self.emitter.publish_runtime_status(&runtime);
        Ok(())
    }

    pub fn stop_runtime(&self, pane_id: &PaneId) {
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
                    if let Err(error) = self.terminal_port.kill(runtime_session_id.as_ref()) {
                        warn!(
                            ?error,
                            pane_id = pane_id.as_ref(),
                            "Failed to kill terminal runtime"
                        );
                    }
                }
                tabby_runtime::RuntimeKind::Browser => {
                    if let Err(error) = self.browser_port.close_surface(pane_id.as_ref()) {
                        warn!(
                            ?error,
                            pane_id = pane_id.as_ref(),
                            "Failed to close browser surface"
                        );
                    }
                }
                tabby_runtime::RuntimeKind::Git => {
                    // Git runtimes have no external process to kill
                }
            }
        }

        let mut exited = runtime;
        exited.status = RuntimeStatus::Exited;
        self.emitter.publish_runtime_status(&exited);
    }

    pub fn restart_runtime(
        &self,
        pane_id: &PaneId,
        spec: &PaneSpec,
        preferences: &UserPreferences,
        observation_receiver: std::sync::Arc<dyn RuntimeObservationReceiver>,
    ) -> Result<(), ShellError> {
        self.stop_runtime(pane_id);
        self.start_runtime(pane_id, spec, preferences, observation_receiver)
    }

    pub fn dispatch_runtime_command(&self, command: RuntimeCommand) -> Result<(), ShellError> {
        match command {
            RuntimeCommand::WriteTerminalInput { pane_id, input } => {
                let runtime_session_id = self
                    .lock_runtimes()?
                    .terminal_session_id(&pane_id)
                    .ok_or_else(|| ShellError::NotFound(format!("runtime for pane {pane_id}")))?;
                self.terminal_port
                    .write_input(runtime_session_id.as_ref(), &input)?;
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
                self.terminal_port
                    .resize(runtime_session_id.as_ref(), cols, rows)?;
            }
            RuntimeCommand::NavigateBrowser { pane_id, url } => {
                self.browser_port.navigate(pane_id.as_ref(), &url)?;
                let maybe_runtime = self
                    .lock_runtimes()?
                    .update_browser_location(&pane_id, tabby_kernel::BrowserUrl::new(url))
                    .ok();
                if let Some(runtime) = maybe_runtime {
                    self.emitter.publish_runtime_status(&runtime);
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

    /// Dispatch a browser surface command through the `BrowserSurfacePort`.
    ///
    /// This keeps `RuntimeApplicationService` as the single owner of all
    /// runtime lifecycle operations, including browser surface management.
    pub fn dispatch_browser_surface_command(
        &self,
        command: BrowserSurfaceCommandDto,
    ) -> Result<(), ShellError> {
        match command {
            BrowserSurfaceCommandDto::Ensure {
                pane_id,
                url,
                bounds,
            } => {
                self.browser_port.ensure_surface(
                    &pane_id,
                    &url,
                    bounds.x,
                    bounds.y,
                    bounds.width,
                    bounds.height,
                )?;
            }
            BrowserSurfaceCommandDto::SetBounds { pane_id, bounds } => {
                self.browser_port.set_bounds(
                    &pane_id,
                    bounds.x,
                    bounds.y,
                    bounds.width,
                    bounds.height,
                )?;
            }
            BrowserSurfaceCommandDto::SetVisible { pane_id, visible } => {
                self.browser_port.set_visible(&pane_id, visible)?;
            }
            BrowserSurfaceCommandDto::Close { pane_id } => {
                self.browser_port.close_surface(&pane_id)?;
            }
        }
        Ok(())
    }

    pub fn observe_terminal_cwd(
        &self,
        pane_id: &PaneId,
        working_directory: &str,
    ) -> Result<(), ShellError> {
        let cwd = tabby_kernel::WorkingDirectory::new(working_directory)
            .map_err(|e| ShellError::Validation(e.to_string()))?;
        let runtime = self
            .lock_runtimes()?
            .update_terminal_cwd(pane_id, cwd)
            .map_err(|error| ShellError::NotFound(error.to_string()))?;
        self.emitter.publish_runtime_status(&runtime);
        Ok(())
    }

    pub fn observe_browser_location(&self, pane_id: &PaneId, url: &str) -> Result<(), ShellError> {
        let maybe_runtime = self
            .lock_runtimes()?
            .update_browser_location(pane_id, tabby_kernel::BrowserUrl::new(url))
            .ok();
        if let Some(runtime) = maybe_runtime {
            self.emitter.publish_runtime_status(&runtime);
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
        // Terminal output bypasses this trait — emitted directly by the PTY read
        // thread to the frontend for performance. This method is reserved for future
        // OSC sequence detection. See docs/adr/001-terminal-output-hot-path.md.
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
                .mark_terminal_exit(pane_id, None, failed, message)
                .map_err(|error| ShellError::NotFound(error.to_string()))
        });

        match result {
            Ok(runtime) => self.emitter.publish_runtime_status(&runtime),
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
                .update_browser_location(pane_id, tabby_kernel::BrowserUrl::new(url))
                .map_err(|error| ShellError::NotFound(error.to_string()))
        });

        match result {
            Ok(runtime) => self.emitter.publish_runtime_status(&runtime),
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
        let wd = match tabby_kernel::WorkingDirectory::new(cwd) {
            Ok(wd) => wd,
            Err(error) => {
                warn!(?error, pane_id = pane_id.as_ref(), "Invalid cwd observed");
                return;
            }
        };
        let result = self.lock_runtimes().and_then(|mut runtimes| {
            runtimes
                .update_terminal_cwd(pane_id, wd)
                .map_err(|error| ShellError::NotFound(error.to_string()))
        });

        match result {
            Ok(runtime) => self.emitter.publish_runtime_status(&runtime),
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
    use tabby_contracts::PaneId as PaneIdType;
    use tabby_kernel::BrowserUrl;
    use tabby_runtime::{RuntimeKind, RuntimeRegistry, RuntimeSessionId, RuntimeStatus};

    fn sid(s: &str) -> RuntimeSessionId {
        RuntimeSessionId::from(String::from(s))
    }

    fn pid(id: &str) -> PaneIdType {
        PaneIdType::from(String::from(id))
    }

    #[test]
    fn start_runtime_registers_terminal_in_registry() {
        let mut registry = RuntimeRegistry::default();

        let runtime = registry.register_terminal(&pid("pane-1"), sid("pty-session-abc"));

        assert_eq!(runtime.pane_id, pid("pane-1"));
        assert_eq!(runtime.runtime_session_id, Some(sid("pty-session-abc")));
        assert!(matches!(runtime.kind, RuntimeKind::Terminal));
        assert!(matches!(runtime.status, RuntimeStatus::Running));

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].pane_id, pid("pane-1"));

        let session_id = registry.terminal_session_id(&pid("pane-1"));
        assert_eq!(session_id, Some(sid("pty-session-abc")));
    }

    #[test]
    fn stop_runtime_removes_from_registry() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal(&pid("pane-1"), sid("pty-session-abc"));
        registry.register_browser(
            &pid("pane-2"),
            sid("browser-xyz"),
            BrowserUrl::new("https://example.com"),
        );

        assert_eq!(registry.snapshot().len(), 2);

        let removed = registry.remove(&pid("pane-1"));
        assert!(removed.is_some());
        let removed = removed.expect("remove returned Some, already asserted");
        assert_eq!(removed.pane_id, pid("pane-1"));
        assert!(matches!(removed.kind, RuntimeKind::Terminal));

        assert_eq!(registry.snapshot().len(), 1);
        assert_eq!(registry.snapshot()[0].pane_id, pid("pane-2"));

        let not_found = registry.remove(&pid("pane-1"));
        assert!(not_found.is_none(), "removing twice should return None");
    }

    #[test]
    fn start_runtime_registers_browser_in_registry() {
        let mut registry = RuntimeRegistry::default();

        let runtime = registry.register_browser(
            &pid("pane-b"),
            sid("browser-session-1"),
            BrowserUrl::new("https://example.com"),
        );

        assert_eq!(runtime.pane_id, pid("pane-b"));
        assert!(matches!(runtime.kind, RuntimeKind::Browser));
        assert!(matches!(runtime.status, RuntimeStatus::Running));
        assert_eq!(
            runtime.browser_location.as_ref().map(|u| u.as_str()),
            Some("https://example.com")
        );
    }

    #[test]
    fn stop_runtime_for_nonexistent_pane_returns_none() {
        let mut registry = RuntimeRegistry::default();
        let removed = registry.remove(&pid("nonexistent-pane"));
        assert!(
            removed.is_none(),
            "removing nonexistent runtime should return None"
        );
    }

    #[test]
    fn restart_runtime_replaces_registry_entry() {
        let mut registry = RuntimeRegistry::default();

        // Initial registration
        registry.register_terminal(&pid("pane-1"), sid("pty-session-1"));
        assert_eq!(registry.snapshot().len(), 1);

        // Simulate restart: remove + register with new session
        let removed = registry.remove(&pid("pane-1"));
        assert!(removed.is_some(), "old runtime should be removed");

        let runtime = registry.register_terminal(&pid("pane-1"), sid("pty-session-2"));
        assert_eq!(runtime.pane_id, pid("pane-1"));
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
        registry.register_terminal(&pid("pane-1"), session_id.clone());

        let result = registry.mark_terminal_exit(
            &pid("pane-1"),
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
        registry.register_terminal(&pid("pane-1"), sid("pty-session-1"));

        // When session ID doesn't match, the exit is silently ignored (stale event)
        let result =
            registry.mark_terminal_exit(&pid("pane-1"), Some(&sid("wrong-session")), false, None);
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
            registry.mark_terminal_exit(&pid("nonexistent"), Some(&sid("session-1")), false, None);
        assert!(result.is_err(), "should return error for nonexistent pane");
    }

    #[test]
    fn update_browser_location_for_nonexistent_pane_returns_error() {
        let mut registry = RuntimeRegistry::default();
        let result = registry
            .update_browser_location(&pid("nonexistent"), BrowserUrl::new("https://example.com"));
        assert!(
            result.is_err(),
            "updating location for nonexistent pane should fail"
        );
    }

    #[test]
    fn snapshot_returns_all_registered_runtimes() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal(&pid("pane-1"), sid("pty-1"));
        registry.register_browser(
            &pid("pane-2"),
            sid("browser-1"),
            BrowserUrl::new("https://example.com"),
        );
        registry.register_terminal(&pid("pane-3"), sid("pty-3"));

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
        registry.register_terminal(&pane_id, sid("pty-1"));

        // 3. Update cwd in the runtime registry (simulating observe_terminal_cwd)
        let runtime = registry
            .update_terminal_cwd(
                &pane_id,
                tabby_kernel::WorkingDirectory::new("/projects/tabby").expect("valid path"),
            )
            .expect("cwd update should succeed");
        assert_eq!(
            runtime.terminal_cwd.as_ref().map(|w| w.as_str()),
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
            other => panic!("expected terminal pane, got {other:?}"),
        }

        // 5. Workspace session should have no track_terminal_working_directory method
        // (compile-time guarantee — if this test compiles, the method is removed)
    }

    // -----------------------------------------------------------------------
    // Mock ports for testing RuntimeApplicationService with abstractions
    // -----------------------------------------------------------------------

    use std::sync::{Arc, Mutex};

    use crate::application::ports::{
        BrowserSurfacePort, ProjectionPublisherPort, TerminalProcessPort,
    };
    use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
    use crate::shell::error::ShellError;
    use tabby_workspace::PaneId;

    #[derive(Debug, Default)]
    struct MockTerminalProcess {
        spawn_calls: Mutex<Vec<(String, String)>>,
        kill_calls: Mutex<Vec<String>>,
        resize_calls: Mutex<Vec<(String, u16, u16)>>,
        write_calls: Mutex<Vec<(String, String)>>,
        next_session_counter: Mutex<u32>,
    }

    impl TerminalProcessPort for MockTerminalProcess {
        fn spawn(
            &self,
            pane_id: &str,
            working_directory: &str,
            _startup_command: Option<&str>,
            _observation_receiver: Arc<dyn RuntimeObservationReceiver>,
        ) -> Result<String, ShellError> {
            let mut counter = self
                .next_session_counter
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?;
            *counter += 1;
            let session_id = format!("mock-pty-{counter}");
            self.spawn_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push((String::from(pane_id), String::from(working_directory)));
            Ok(session_id)
        }

        fn kill(&self, runtime_session_id: &str) -> Result<(), ShellError> {
            self.kill_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push(String::from(runtime_session_id));
            Ok(())
        }

        fn resize(&self, runtime_session_id: &str, cols: u16, rows: u16) -> Result<(), ShellError> {
            self.resize_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push((String::from(runtime_session_id), cols, rows));
            Ok(())
        }

        fn write_input(&self, runtime_session_id: &str, data: &str) -> Result<(), ShellError> {
            self.write_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push((String::from(runtime_session_id), String::from(data)));
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct MockBrowserSurface {
        close_calls: Mutex<Vec<String>>,
        navigate_calls: Mutex<Vec<(String, String)>>,
    }

    impl BrowserSurfacePort for MockBrowserSurface {
        fn ensure_surface(
            &self,
            _pane_id: &str,
            _url: &str,
            _x: f64,
            _y: f64,
            _width: f64,
            _height: f64,
        ) -> Result<(), ShellError> {
            Ok(())
        }

        fn set_bounds(
            &self,
            _pane_id: &str,
            _x: f64,
            _y: f64,
            _width: f64,
            _height: f64,
        ) -> Result<(), ShellError> {
            Ok(())
        }

        fn set_visible(&self, _pane_id: &str, _visible: bool) -> Result<(), ShellError> {
            Ok(())
        }

        fn close_surface(&self, pane_id: &str) -> Result<(), ShellError> {
            self.close_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push(String::from(pane_id));
            Ok(())
        }

        fn navigate(&self, pane_id: &str, url: &str) -> Result<(), ShellError> {
            self.navigate_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push((String::from(pane_id), String::from(url)));
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct MockProjectionEmitter {
        emitted: Mutex<Vec<(String, RuntimeStatus)>>,
        workspace_calls: Mutex<u32>,
        settings_calls: Mutex<u32>,
    }

    impl ProjectionPublisherPort for MockProjectionEmitter {
        fn publish_workspace_projection(&self, _workspace: &tabby_workspace::WorkspaceSession) {
            if let Ok(mut count) = self.workspace_calls.lock() {
                *count += 1;
            }
        }
        fn publish_settings_projection(&self, _preferences: &UserPreferences) {
            if let Ok(mut count) = self.settings_calls.lock() {
                *count += 1;
            }
        }
        fn publish_runtime_status(&self, runtime: &tabby_runtime::PaneRuntime) {
            if let Ok(mut emitted) = self.emitted.lock() {
                emitted.push((runtime.pane_id.to_string(), runtime.status));
            }
        }
    }

    use super::RuntimeApplicationService;
    use tabby_settings::UserPreferences;
    use tabby_workspace::{BrowserPaneSpec, GitPaneSpec, PaneSpec, TerminalPaneSpec};

    fn default_preferences() -> UserPreferences {
        tabby_settings::default_preferences()
    }

    struct MockObservationReceiver;

    impl RuntimeObservationReceiver for MockObservationReceiver {
        fn on_terminal_output_received(&self, _pane_id: &PaneId, _data: &[u8]) {}
        fn on_terminal_exited(&self, _pane_id: &PaneId, _exit_code: Option<i32>) {}
        fn on_browser_location_changed(&self, _pane_id: &PaneId, _url: &str) {}
        fn on_terminal_cwd_changed(&self, _pane_id: &PaneId, _cwd: &str) {}
    }

    fn mock_receiver() -> Arc<dyn RuntimeObservationReceiver> {
        Arc::new(MockObservationReceiver)
    }

    fn build_service() -> (
        RuntimeApplicationService,
        Arc<MockTerminalProcess>,
        Arc<MockBrowserSurface>,
        Arc<MockProjectionEmitter>,
    ) {
        let terminal = Arc::new(MockTerminalProcess::default());
        let browser = Arc::new(MockBrowserSurface::default());
        let emitter = Arc::new(MockProjectionEmitter::default());

        // Use Arc-based sharing so tests can inspect mocks after service calls.
        // The service takes Box<dyn Trait>, so we clone the Arc into the box.
        let service = RuntimeApplicationService::new(
            Box::new(ArcTerminalPort(Arc::clone(&terminal))),
            Box::new(ArcBrowserPort(Arc::clone(&browser))),
            Box::new(ArcEmitter(Arc::clone(&emitter))),
        );

        (service, terminal, browser, emitter)
    }

    // Thin wrappers to pass Arc<Mock> as Box<dyn Trait>

    #[derive(Debug)]
    struct ArcTerminalPort(Arc<MockTerminalProcess>);

    impl TerminalProcessPort for ArcTerminalPort {
        fn spawn(
            &self,
            pane_id: &str,
            working_directory: &str,
            startup_command: Option<&str>,
            observation_receiver: Arc<dyn RuntimeObservationReceiver>,
        ) -> Result<String, ShellError> {
            self.0.spawn(
                pane_id,
                working_directory,
                startup_command,
                observation_receiver,
            )
        }
        fn kill(&self, id: &str) -> Result<(), ShellError> {
            self.0.kill(id)
        }
        fn resize(&self, id: &str, cols: u16, rows: u16) -> Result<(), ShellError> {
            self.0.resize(id, cols, rows)
        }
        fn write_input(&self, id: &str, data: &str) -> Result<(), ShellError> {
            self.0.write_input(id, data)
        }
    }

    #[derive(Debug)]
    struct ArcBrowserPort(Arc<MockBrowserSurface>);

    impl BrowserSurfacePort for ArcBrowserPort {
        fn ensure_surface(
            &self,
            p: &str,
            u: &str,
            x: f64,
            y: f64,
            w: f64,
            h: f64,
        ) -> Result<(), ShellError> {
            self.0.ensure_surface(p, u, x, y, w, h)
        }
        fn set_bounds(&self, p: &str, x: f64, y: f64, w: f64, h: f64) -> Result<(), ShellError> {
            self.0.set_bounds(p, x, y, w, h)
        }
        fn set_visible(&self, p: &str, v: bool) -> Result<(), ShellError> {
            self.0.set_visible(p, v)
        }
        fn close_surface(&self, p: &str) -> Result<(), ShellError> {
            self.0.close_surface(p)
        }
        fn navigate(&self, p: &str, u: &str) -> Result<(), ShellError> {
            self.0.navigate(p, u)
        }
    }

    #[derive(Debug)]
    struct ArcEmitter(Arc<MockProjectionEmitter>);

    impl ProjectionPublisherPort for ArcEmitter {
        fn publish_workspace_projection(&self, _workspace: &tabby_workspace::WorkspaceSession) {}
        fn publish_settings_projection(&self, _preferences: &UserPreferences) {}
        fn publish_runtime_status(&self, runtime: &tabby_runtime::PaneRuntime) {
            self.0.publish_runtime_status(runtime);
        }
    }

    // -----------------------------------------------------------------------
    // AC: RuntimeApplicationService start/stop works with mock ports
    // -----------------------------------------------------------------------

    fn terminal_spec_for_test(cwd: &str) -> PaneSpec {
        PaneSpec::Terminal(TerminalPaneSpec {
            launch_profile_id: String::from(tabby_settings::TERMINAL_PROFILE_ID),
            working_directory: String::from(cwd),
            command_override: None,
        })
    }

    #[test]
    fn start_terminal_runtime_calls_terminal_port_spawn() {
        let (service, terminal, _browser, emitter) = build_service();
        let prefs = default_preferences();
        let spec = terminal_spec_for_test("/tmp");

        service
            .start_runtime(&pid("pane-1"), &spec, &prefs, mock_receiver())
            .expect("start should succeed");

        // Terminal port spawn was called
        let spawn_calls = terminal.spawn_calls.lock().expect("lock");
        assert_eq!(spawn_calls.len(), 1);
        assert_eq!(spawn_calls[0].0, "pane-1");
        assert_eq!(spawn_calls[0].1, "/tmp");

        // Runtime registered in registry
        let snapshot = service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].pane_id.as_ref(), "pane-1");
        assert!(matches!(snapshot[0].kind, RuntimeKind::Terminal));
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));

        // Projection emitted
        let emitted = emitter.emitted.lock().expect("lock");
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].0, "pane-1");
        assert_eq!(emitted[0].1, RuntimeStatus::Running);
    }

    #[test]
    fn stop_terminal_runtime_calls_terminal_port_kill() {
        let (service, terminal, _browser, emitter) = build_service();
        let prefs = default_preferences();
        let spec = terminal_spec_for_test("/tmp");

        service
            .start_runtime(&pid("pane-1"), &spec, &prefs, mock_receiver())
            .expect("start");

        // Get the session ID that was assigned
        let snapshot = service.snapshot().expect("snapshot");
        let session_id = snapshot[0]
            .runtime_session_id
            .as_ref()
            .expect("should have session id")
            .clone();

        service.stop_runtime(&pid("pane-1"));

        // Terminal port kill was called with the correct session ID
        let kill_calls = terminal.kill_calls.lock().expect("lock");
        assert_eq!(kill_calls.len(), 1);
        assert_eq!(kill_calls[0], session_id.as_ref());

        // Registry is empty
        let snapshot = service.snapshot().expect("snapshot");
        assert!(snapshot.is_empty(), "registry should be empty after stop");

        // Exited projection emitted
        let emitted = emitter.emitted.lock().expect("lock");
        let exited = emitted
            .iter()
            .filter(|(id, status)| id == "pane-1" && *status == RuntimeStatus::Exited)
            .count();
        assert_eq!(exited, 1, "Exited projection should be emitted");
    }

    #[test]
    fn start_browser_runtime_does_not_call_terminal_port() {
        let (service, terminal, _browser, emitter) = build_service();
        let prefs = default_preferences();
        let spec = PaneSpec::Browser(BrowserPaneSpec {
            initial_url: BrowserUrl::new("https://example.com"),
        });

        service
            .start_runtime(&pid("pane-b"), &spec, &prefs, mock_receiver())
            .expect("start");

        // Terminal port was NOT called
        let spawn_calls = terminal.spawn_calls.lock().expect("lock");
        assert!(
            spawn_calls.is_empty(),
            "terminal port should not be called for browser runtime"
        );

        // Browser runtime registered
        let snapshot = service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert!(matches!(snapshot[0].kind, RuntimeKind::Browser));

        // Projection emitted
        let emitted = emitter.emitted.lock().expect("lock");
        assert_eq!(emitted.len(), 1);
    }

    #[test]
    fn stop_browser_runtime_calls_browser_port_close() {
        let (service, _terminal, browser, _emitter) = build_service();
        let prefs = default_preferences();
        let spec = PaneSpec::Browser(BrowserPaneSpec {
            initial_url: BrowserUrl::new("https://example.com"),
        });

        service
            .start_runtime(&pid("pane-b"), &spec, &prefs, mock_receiver())
            .expect("start");

        service.stop_runtime(&pid("pane-b"));

        // Browser port close was called
        let close_calls = browser.close_calls.lock().expect("lock");
        assert_eq!(close_calls.len(), 1);
        assert_eq!(close_calls[0], "pane-b");
    }

    #[test]
    fn restart_runtime_stops_then_starts_via_ports() {
        let (service, terminal, _browser, emitter) = build_service();
        let prefs = default_preferences();
        let spec = terminal_spec_for_test("/projects");

        service
            .start_runtime(&pid("pane-1"), &spec, &prefs, mock_receiver())
            .expect("start");

        service
            .restart_runtime(&pid("pane-1"), &spec, &prefs, mock_receiver())
            .expect("restart");

        // Terminal port: 2 spawns, 1 kill
        let spawn_calls = terminal.spawn_calls.lock().expect("lock");
        assert_eq!(spawn_calls.len(), 2, "spawn should be called twice");
        let kill_calls = terminal.kill_calls.lock().expect("lock");
        assert_eq!(kill_calls.len(), 1, "kill should be called once");

        // Registry has exactly one runtime, Running
        let snapshot = service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));

        // Projections: Running, Exited, Running
        let emitted = emitter.emitted.lock().expect("lock");
        let statuses: Vec<_> = emitted
            .iter()
            .filter(|(id, _)| id == "pane-1")
            .map(|(_, s)| *s)
            .collect();
        assert_eq!(
            statuses,
            vec![
                RuntimeStatus::Running,
                RuntimeStatus::Exited,
                RuntimeStatus::Running,
            ]
        );
    }

    #[test]
    fn dispatch_navigate_browser_calls_browser_port() {
        use crate::application::commands::RuntimeCommand;

        let (service, _terminal, browser, _emitter) = build_service();
        let prefs = default_preferences();
        let spec = PaneSpec::Browser(BrowserPaneSpec {
            initial_url: BrowserUrl::new("https://example.com"),
        });

        service
            .start_runtime(&pid("pane-b"), &spec, &prefs, mock_receiver())
            .expect("start");

        service
            .dispatch_runtime_command(RuntimeCommand::NavigateBrowser {
                pane_id: PaneId::from(String::from("pane-b")),
                url: String::from("https://docs.rs"),
            })
            .expect("navigate");

        let nav_calls = browser.navigate_calls.lock().expect("lock");
        assert_eq!(nav_calls.len(), 1);
        assert_eq!(nav_calls[0].0, "pane-b");
        assert_eq!(nav_calls[0].1, "https://docs.rs");
    }

    #[test]
    fn dispatch_write_terminal_calls_terminal_port() {
        use crate::application::commands::RuntimeCommand;

        let (service, terminal, _browser, _emitter) = build_service();
        let prefs = default_preferences();
        let spec = terminal_spec_for_test("/tmp");

        service
            .start_runtime(&pid("pane-t"), &spec, &prefs, mock_receiver())
            .expect("start");

        let snapshot = service.snapshot().expect("snapshot");
        let _session_id = snapshot[0].runtime_session_id.as_ref().expect("session id");

        service
            .dispatch_runtime_command(RuntimeCommand::WriteTerminalInput {
                pane_id: PaneId::from(String::from("pane-t")),
                input: String::from("ls\n"),
            })
            .expect("write");

        let write_calls = terminal.write_calls.lock().expect("lock");
        assert_eq!(write_calls.len(), 1);
        assert_eq!(write_calls[0].1, "ls\n");
    }

    #[test]
    fn dispatch_resize_terminal_calls_terminal_port() {
        use crate::application::commands::RuntimeCommand;

        let (service, terminal, _browser, _emitter) = build_service();
        let prefs = default_preferences();
        let spec = terminal_spec_for_test("/tmp");

        service
            .start_runtime(&pid("pane-t"), &spec, &prefs, mock_receiver())
            .expect("start");

        service
            .dispatch_runtime_command(RuntimeCommand::ResizeTerminal {
                pane_id: PaneId::from(String::from("pane-t")),
                cols: 120,
                rows: 40,
            })
            .expect("resize");

        let resize_calls = terminal.resize_calls.lock().expect("lock");
        assert_eq!(resize_calls.len(), 1);
        assert_eq!(resize_calls[0].1, 120);
        assert_eq!(resize_calls[0].2, 40);
    }

    // -----------------------------------------------------------------------
    // US-025: Browser location observation → unified RuntimeStatusChangedEvent
    // -----------------------------------------------------------------------

    /// US-025: Browser location observation flows through RuntimeStatusChangedEvent
    /// (not a separate BrowserLocationObservedEvent). Verifies: browser surface triggers
    /// location change → registry updated → RuntimeStatusChangedEvent emitted with
    /// updated browser_location.
    #[test]
    fn browser_location_observation_emits_runtime_status_changed() {
        let (service, _terminal, _browser, emitter) = build_service();
        let prefs = default_preferences();
        let spec = PaneSpec::Browser(BrowserPaneSpec {
            initial_url: BrowserUrl::new("https://example.com"),
        });

        service
            .start_runtime(&pid("pane-b"), &spec, &prefs, mock_receiver())
            .expect("start browser runtime");

        // Clear initial emission from start_runtime
        emitter.emitted.lock().expect("lock").clear();

        // Simulate browser navigation observation (as browser_surface.rs would call)
        service
            .observe_browser_location(&pid("pane-b"), "https://docs.rs/tauri")
            .expect("observe should succeed");

        // Verify: registry updated
        let snapshot = service.snapshot().expect("snapshot");
        let browser_runtime = snapshot
            .iter()
            .find(|r| r.pane_id.as_ref() == "pane-b")
            .expect("found");
        assert_eq!(
            browser_runtime
                .browser_location
                .as_ref()
                .map(|u| u.as_str()),
            Some("https://docs.rs/tauri"),
            "registry should reflect observed browser location"
        );

        // Verify: RuntimeStatusChangedEvent was emitted (via publish_runtime_status)
        let emitted = emitter.emitted.lock().expect("lock");
        assert_eq!(emitted.len(), 1, "exactly one emission for location change");
        assert_eq!(emitted[0].0, "pane-b");
        assert_eq!(emitted[0].1, RuntimeStatus::Running);
    }

    /// US-025: on_browser_location_changed trait method also flows through RuntimeStatusChangedEvent
    #[test]
    fn on_browser_location_changed_trait_emits_runtime_status_changed() {
        let (service, _terminal, _browser, emitter) = build_service();
        let prefs = default_preferences();
        let spec = PaneSpec::Browser(BrowserPaneSpec {
            initial_url: BrowserUrl::new("https://example.com"),
        });

        service
            .start_runtime(&pid("pane-b"), &spec, &prefs, mock_receiver())
            .expect("start");
        emitter.emitted.lock().expect("lock").clear();

        // Call the trait method directly (as browser_surface.rs would via AppShell)
        let pane_id = PaneId::from(String::from("pane-b"));
        RuntimeObservationReceiver::on_browser_location_changed(
            &service,
            &pane_id,
            "https://github.com",
        );

        let snapshot = service.snapshot().expect("snapshot");
        let runtime = snapshot
            .iter()
            .find(|r| r.pane_id.as_ref() == "pane-b")
            .expect("found");
        assert_eq!(
            runtime.browser_location.as_ref().map(|u| u.as_str()),
            Some("https://github.com"),
        );

        let emitted = emitter.emitted.lock().expect("lock");
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].0, "pane-b");
    }

    // -----------------------------------------------------------------------
    // AC#5: Projection publishing works with mock publisher
    // -----------------------------------------------------------------------

    #[test]
    fn mock_publisher_receives_all_three_projection_types() {
        let publisher = MockProjectionEmitter::default();

        // 1. publish_workspace_projection
        let session = tabby_workspace::WorkspaceSession::default();
        publisher.publish_workspace_projection(&session);
        assert_eq!(
            *publisher.workspace_calls.lock().expect("lock"),
            1,
            "workspace projection should be published"
        );

        // 2. publish_settings_projection
        let prefs = tabby_settings::default_preferences();
        publisher.publish_settings_projection(&prefs);
        assert_eq!(
            *publisher.settings_calls.lock().expect("lock"),
            1,
            "settings projection should be published"
        );

        // 3. publish_runtime_status
        let runtime = tabby_runtime::PaneRuntime {
            pane_id: PaneId::from(String::from("pane-1")),
            kind: tabby_runtime::RuntimeKind::Terminal,
            status: RuntimeStatus::Running,
            runtime_session_id: Some(RuntimeSessionId::from(String::from("pty-1"))),
            browser_location: None,
            last_error: None,
            terminal_cwd: None,
            git_repo_path: None,
        };
        publisher.publish_runtime_status(&runtime);
        let emitted = publisher.emitted.lock().expect("lock");
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].0, "pane-1");
        assert_eq!(emitted[0].1, RuntimeStatus::Running);
    }

    #[test]
    fn projection_publisher_port_is_object_safe_behind_box() {
        // Verify the trait can be used as Box<dyn ProjectionPublisherPort>
        let publisher: Box<dyn ProjectionPublisherPort> =
            Box::new(MockProjectionEmitter::default());

        let session = tabby_workspace::WorkspaceSession::default();
        publisher.publish_workspace_projection(&session);
        publisher.publish_settings_projection(&tabby_settings::default_preferences());

        let runtime = tabby_runtime::PaneRuntime {
            pane_id: PaneId::from(String::from("pane-x")),
            kind: tabby_runtime::RuntimeKind::Browser,
            status: RuntimeStatus::Running,
            runtime_session_id: None,
            browser_location: Some(BrowserUrl::new("https://example.com")),
            last_error: None,
            terminal_cwd: None,
            git_repo_path: None,
        };
        publisher.publish_runtime_status(&runtime);
    }

    // -----------------------------------------------------------------------
    // GIT-012: Git pane start/stop/restart lifecycle
    // -----------------------------------------------------------------------

    fn git_spec_for_test(path: &str) -> PaneSpec {
        PaneSpec::Git(GitPaneSpec {
            working_directory: String::from(path),
        })
    }

    #[test]
    fn start_git_runtime_registers_in_registry_without_spawning_process() {
        let (service, terminal, _browser, emitter) = build_service();
        let prefs = default_preferences();
        let spec = git_spec_for_test("/projects/tabby");

        service
            .start_runtime(&pid("git-pane-1"), &spec, &prefs, mock_receiver())
            .expect("start git runtime should succeed");

        // Terminal port was NOT called (no OS process)
        let spawn_calls = terminal.spawn_calls.lock().expect("lock");
        assert!(
            spawn_calls.is_empty(),
            "terminal port should not be called for git runtime"
        );

        // Runtime registered in registry
        let snapshot = service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].pane_id.as_ref(), "git-pane-1");
        assert!(matches!(snapshot[0].kind, RuntimeKind::Git));
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));
        assert!(
            snapshot[0].runtime_session_id.is_some(),
            "git runtime should have a synthetic session id"
        );
        assert_eq!(
            snapshot[0].git_repo_path.as_ref().map(|p| p.as_str()),
            Some("/projects/tabby"),
            "git runtime should record repo path"
        );

        // Projection emitted
        let emitted = emitter.emitted.lock().expect("lock");
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].0, "git-pane-1");
        assert_eq!(emitted[0].1, RuntimeStatus::Running);
    }

    #[test]
    fn stop_git_runtime_removes_from_registry_without_killing_process() {
        let (service, terminal, browser, emitter) = build_service();
        let prefs = default_preferences();
        let spec = git_spec_for_test("/projects/tabby");

        service
            .start_runtime(&pid("git-pane-1"), &spec, &prefs, mock_receiver())
            .expect("start");

        service.stop_runtime(&pid("git-pane-1"));

        // No OS process kill calls
        let kill_calls = terminal.kill_calls.lock().expect("lock");
        assert!(
            kill_calls.is_empty(),
            "terminal kill should not be called for git runtime"
        );
        let close_calls = browser.close_calls.lock().expect("lock");
        assert!(
            close_calls.is_empty(),
            "browser close should not be called for git runtime"
        );

        // Registry is empty
        let snapshot = service.snapshot().expect("snapshot");
        assert!(snapshot.is_empty(), "registry should be empty after stop");

        // Exited projection emitted
        let emitted = emitter.emitted.lock().expect("lock");
        let exited = emitted
            .iter()
            .filter(|(id, status)| id == "git-pane-1" && *status == RuntimeStatus::Exited)
            .count();
        assert_eq!(
            exited, 1,
            "Exited projection should be emitted for git runtime"
        );
    }

    #[test]
    fn restart_git_runtime_stops_then_starts() {
        let (service, terminal, _browser, emitter) = build_service();
        let prefs = default_preferences();
        let spec = git_spec_for_test("/projects/tabby");

        service
            .start_runtime(&pid("git-pane-1"), &spec, &prefs, mock_receiver())
            .expect("start");

        service
            .restart_runtime(&pid("git-pane-1"), &spec, &prefs, mock_receiver())
            .expect("restart");

        // No OS process interactions
        let spawn_calls = terminal.spawn_calls.lock().expect("lock");
        assert!(spawn_calls.is_empty(), "no terminal spawns for git runtime");
        let kill_calls = terminal.kill_calls.lock().expect("lock");
        assert!(kill_calls.is_empty(), "no terminal kills for git runtime");

        // Registry has exactly one runtime, Running
        let snapshot = service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert!(matches!(snapshot[0].kind, RuntimeKind::Git));
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));

        // Projections: Running, Exited, Running
        let emitted = emitter.emitted.lock().expect("lock");
        let statuses: Vec<_> = emitted
            .iter()
            .filter(|(id, _)| id == "git-pane-1")
            .map(|(_, s)| *s)
            .collect();
        assert_eq!(
            statuses,
            vec![
                RuntimeStatus::Running,
                RuntimeStatus::Exited,
                RuntimeStatus::Running,
            ]
        );
    }

    #[test]
    fn stop_nonexistent_git_runtime_is_noop() {
        let (service, _terminal, _browser, emitter) = build_service();

        // Stopping a pane that was never started should not panic
        service.stop_runtime(&pid("git-pane-nonexistent"));

        // No projections emitted
        let emitted = emitter.emitted.lock().expect("lock");
        assert!(emitted.is_empty(), "no projection for nonexistent pane");
    }

    #[test]
    fn git_runtime_coexists_with_terminal_and_browser() {
        let (service, _terminal, _browser, _emitter) = build_service();
        let prefs = default_preferences();

        service
            .start_runtime(
                &pid("term-1"),
                &terminal_spec_for_test("/tmp"),
                &prefs,
                mock_receiver(),
            )
            .expect("start terminal");

        service
            .start_runtime(
                &pid("browser-1"),
                &PaneSpec::Browser(BrowserPaneSpec {
                    initial_url: BrowserUrl::new("https://example.com"),
                }),
                &prefs,
                mock_receiver(),
            )
            .expect("start browser");

        service
            .start_runtime(
                &pid("git-1"),
                &git_spec_for_test("/repos/my-project"),
                &prefs,
                mock_receiver(),
            )
            .expect("start git");

        let snapshot = service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 3, "all three runtimes should coexist");

        let kinds: Vec<_> = {
            let mut k: Vec<_> = snapshot.iter().map(|r| r.kind).collect();
            k.sort_by_key(|k| match k {
                RuntimeKind::Terminal => 0,
                RuntimeKind::Browser => 1,
                RuntimeKind::Git => 2,
            });
            k
        };
        assert_eq!(
            kinds,
            vec![
                RuntimeKind::Terminal,
                RuntimeKind::Browser,
                RuntimeKind::Git
            ]
        );
    }
}
