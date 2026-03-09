//! Integration tests for the full command-to-side-effect dispatch pipeline (US-031).
//!
//! These tests exercise the complete dispatch chain:
//!   WorkspaceApplicationService (command) → domain events
//!     → RuntimeCoordinator → RuntimeApplicationService → mock infrastructure ports
//!
//! Unlike `runtime_integration_tests.rs` (US-029) which starts from pre-built
//! `WorkspaceDomainEvent` vectors, these tests start from workspace commands
//! (`open_tab`, `close_pane`, `close_tab`) — the same entry point used by
//! Tauri command handlers — and assert that the correct side-effects reach
//! the mock ports.
//!
//! No real Tauri, PTY, or browser surface is involved.

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tabby_runtime::{RuntimeKind, RuntimeStatus};
    use tabby_settings::UserPreferences;
    use tabby_workspace::layout::LayoutPreset;
    use tabby_workspace::{BrowserPaneSpec, BrowserUrl, PaneSpec, TerminalPaneSpec};

    use crate::application::ports::{
        BrowserSurfacePort, PreferencesRepository, ProjectionPublisherPort, TerminalProcessPort,
    };
    use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
    use crate::application::{
        RuntimeApplicationService, RuntimeCoordinator, SettingsApplicationService,
        WorkspaceApplicationService,
    };
    use crate::shell::error::ShellError;

    // -----------------------------------------------------------------------
    // Mock ports (same pattern as runtime_integration_tests.rs)
    // -----------------------------------------------------------------------

    #[derive(Debug, Default)]
    struct MockTerminalPort {
        spawn_calls: Mutex<Vec<(String, String, Option<String>)>>,
        kill_calls: Mutex<Vec<String>>,
        next_counter: Mutex<u32>,
    }

    impl TerminalProcessPort for MockTerminalPort {
        fn spawn(
            &self,
            pane_id: &str,
            working_directory: &str,
            startup_command: Option<&str>,
            _observation_receiver: Arc<dyn RuntimeObservationReceiver>,
        ) -> Result<String, ShellError> {
            let mut counter = self
                .next_counter
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?;
            *counter += 1;
            let session_id = format!("mock-pty-{counter}");
            self.spawn_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push((
                    String::from(pane_id),
                    String::from(working_directory),
                    startup_command.map(String::from),
                ));
            Ok(session_id)
        }

        fn kill(&self, runtime_session_id: &str) -> Result<(), ShellError> {
            self.kill_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push(String::from(runtime_session_id));
            Ok(())
        }

        fn resize(&self, _id: &str, _cols: u16, _rows: u16) -> Result<(), ShellError> {
            Ok(())
        }

        fn write_input(&self, _id: &str, _data: &str) -> Result<(), ShellError> {
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct MockBrowserPort {
        ensure_surface_calls: Mutex<Vec<String>>,
        close_calls: Mutex<Vec<String>>,
    }

    impl BrowserSurfacePort for MockBrowserPort {
        fn ensure_surface(
            &self,
            pane_id: &str,
            _url: &str,
            _x: f64,
            _y: f64,
            _w: f64,
            _h: f64,
        ) -> Result<(), ShellError> {
            self.ensure_surface_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push(String::from(pane_id));
            Ok(())
        }

        fn set_bounds(
            &self,
            _p: &str,
            _x: f64,
            _y: f64,
            _w: f64,
            _h: f64,
        ) -> Result<(), ShellError> {
            Ok(())
        }

        fn set_visible(&self, _p: &str, _v: bool) -> Result<(), ShellError> {
            Ok(())
        }

        fn close_surface(&self, pane_id: &str) -> Result<(), ShellError> {
            self.close_calls
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?
                .push(String::from(pane_id));
            Ok(())
        }

        fn navigate(&self, _p: &str, _u: &str) -> Result<(), ShellError> {
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct MockEmitter {
        runtime_statuses: Mutex<Vec<(String, RuntimeKind, RuntimeStatus)>>,
    }

    impl ProjectionPublisherPort for MockEmitter {
        fn publish_workspace_projection(&self, _workspace: &tabby_contracts::WorkspaceView) {}
        fn publish_settings_projection(&self, _preferences: &UserPreferences) {}
        fn publish_runtime_status(&self, runtime: &tabby_runtime::PaneRuntime) {
            if let Ok(mut statuses) = self.runtime_statuses.lock() {
                statuses.push((runtime.pane_id.to_string(), runtime.kind, runtime.status));
            }
        }
    }

    #[derive(Debug, Default)]
    struct MockPreferencesRepo {
        stored: Mutex<Option<serde_json::Value>>,
    }

    impl PreferencesRepository for MockPreferencesRepo {
        fn load(&self) -> Result<Option<serde_json::Value>, ShellError> {
            let guard = self
                .stored
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?;
            Ok(guard.clone())
        }

        fn save(&self, preferences: &UserPreferences) -> Result<(), ShellError> {
            let value = tabby_settings::persistence::serialize_preferences(preferences)
                .map_err(|e| ShellError::Serialization(e.to_string()))?;
            let mut guard = self
                .stored
                .lock()
                .map_err(|_| ShellError::State(String::from("lock")))?;
            *guard = Some(value);
            Ok(())
        }
    }

    struct MockObservationReceiver;

    impl RuntimeObservationReceiver for MockObservationReceiver {
        fn on_terminal_output_received(&self, _pane_id: &tabby_workspace::PaneId, _data: &[u8]) {}
        fn on_terminal_exited(&self, _pane_id: &tabby_workspace::PaneId, _exit_code: Option<i32>) {}
        fn on_browser_location_changed(&self, _pane_id: &tabby_workspace::PaneId, _url: &str) {}
        fn on_terminal_cwd_changed(&self, _pane_id: &tabby_workspace::PaneId, _cwd: &str) {}
    }

    // -----------------------------------------------------------------------
    // Arc wrappers (needed to share mocks between service and test assertions)
    // -----------------------------------------------------------------------

    #[derive(Debug)]
    struct ArcTerminal(Arc<MockTerminalPort>);

    impl TerminalProcessPort for ArcTerminal {
        fn spawn(
            &self,
            pane_id: &str,
            wd: &str,
            cmd: Option<&str>,
            obs: Arc<dyn RuntimeObservationReceiver>,
        ) -> Result<String, ShellError> {
            self.0.spawn(pane_id, wd, cmd, obs)
        }
        fn kill(&self, id: &str) -> Result<(), ShellError> {
            self.0.kill(id)
        }
        fn resize(&self, id: &str, c: u16, r: u16) -> Result<(), ShellError> {
            self.0.resize(id, c, r)
        }
        fn write_input(&self, id: &str, d: &str) -> Result<(), ShellError> {
            self.0.write_input(id, d)
        }
    }

    #[derive(Debug)]
    struct ArcBrowser(Arc<MockBrowserPort>);

    impl BrowserSurfacePort for ArcBrowser {
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
    struct ArcEmitter(Arc<MockEmitter>);

    impl ProjectionPublisherPort for ArcEmitter {
        fn publish_workspace_projection(&self, w: &tabby_contracts::WorkspaceView) {
            self.0.publish_workspace_projection(w);
        }
        fn publish_settings_projection(&self, p: &UserPreferences) {
            self.0.publish_settings_projection(p);
        }
        fn publish_runtime_status(&self, r: &tabby_runtime::PaneRuntime) {
            self.0.publish_runtime_status(r);
        }
    }

    // -----------------------------------------------------------------------
    // Test harness: wires real services with mock ports
    // -----------------------------------------------------------------------

    /// Full-stack test harness that wires real `WorkspaceApplicationService`,
    /// `SettingsApplicationService`, and `RuntimeApplicationService` with
    /// mock infrastructure ports.
    ///
    /// The `dispatch_and_coordinate` method simulates the exact flow that
    /// `AppShell` uses in production: call a workspace command, collect the
    /// domain events, and feed them through `RuntimeCoordinator`.
    struct DispatchHarness {
        workspace_service: WorkspaceApplicationService,
        runtime_service: RuntimeApplicationService,
        settings_service: SettingsApplicationService,
        terminal_port: Arc<MockTerminalPort>,
        browser_port: Arc<MockBrowserPort>,
        emitter: Arc<MockEmitter>,
    }

    impl DispatchHarness {
        fn new() -> Self {
            let terminal_port = Arc::new(MockTerminalPort::default());
            let browser_port = Arc::new(MockBrowserPort::default());
            let emitter = Arc::new(MockEmitter::default());
            let preferences_repo = Arc::new(MockPreferencesRepo::default());

            let runtime_service = RuntimeApplicationService::new(
                Box::new(ArcTerminal(Arc::clone(&terminal_port))),
                Box::new(ArcBrowser(Arc::clone(&browser_port))),
                Box::new(ArcEmitter(Arc::clone(&emitter))),
            );

            #[derive(Debug)]
            struct ArcRepo(Arc<MockPreferencesRepo>);
            impl PreferencesRepository for ArcRepo {
                fn load(&self) -> Result<Option<serde_json::Value>, ShellError> {
                    self.0.load()
                }
                fn save(&self, prefs: &UserPreferences) -> Result<(), ShellError> {
                    self.0.save(prefs)
                }
            }

            let settings_service =
                SettingsApplicationService::new(Box::new(ArcRepo(Arc::clone(&preferences_repo))))
                    .expect("settings service should initialize");

            let workspace_service = WorkspaceApplicationService::new();

            Self {
                workspace_service,
                runtime_service,
                settings_service,
                terminal_port,
                browser_port,
                emitter,
            }
        }

        /// Feed workspace domain events through RuntimeCoordinator, exactly
        /// as AppShell does in production.
        fn coordinate(
            &self,
            events: Vec<tabby_workspace::WorkspaceDomainEvent>,
        ) -> Result<(), ShellError> {
            RuntimeCoordinator::handle_workspace_events(
                events,
                &self.settings_service,
                &self.runtime_service,
                self.observation_receiver(),
            )
        }

        fn observation_receiver(&self) -> Arc<dyn RuntimeObservationReceiver> {
            Arc::new(MockObservationReceiver)
        }

        fn spawn_calls(&self) -> Vec<(String, String, Option<String>)> {
            self.terminal_port.spawn_calls.lock().expect("lock").clone()
        }

        fn kill_calls(&self) -> Vec<String> {
            self.terminal_port.kill_calls.lock().expect("lock").clone()
        }

        fn close_surface_calls(&self) -> Vec<String> {
            self.browser_port.close_calls.lock().expect("lock").clone()
        }

        fn runtime_projections(&self) -> Vec<(String, RuntimeKind, RuntimeStatus)> {
            self.emitter.runtime_statuses.lock().expect("lock").clone()
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn terminal_spec(cwd: &str) -> PaneSpec {
        PaneSpec::Terminal(TerminalPaneSpec {
            launch_profile_id: String::from(tabby_settings::TERMINAL_PROFILE_ID),
            working_directory: String::from(cwd),
            command_override: None,
        })
    }

    fn browser_spec(url: &str) -> PaneSpec {
        PaneSpec::Browser(BrowserPaneSpec {
            initial_url: BrowserUrl::new(url),
        })
    }

    // =======================================================================
    // AC#1: open_tab command → WorkspaceApplicationService → PaneAdded events
    //       → RuntimeCoordinator → RuntimeApplicationService
    //       → mock TerminalProcessPort.spawn() called
    // =======================================================================

    #[test]
    fn open_tab_command_dispatches_through_coordinator_to_spawn_terminal() {
        let h = DispatchHarness::new();

        // Step 1: Workspace command produces domain events
        let events = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByOne,
                false,
                vec![terminal_spec("/projects")],
            )
            .expect("open_tab should succeed");

        // Verify events were produced
        let pane_added_count = events
            .iter()
            .filter(|e| matches!(e, tabby_workspace::WorkspaceDomainEvent::PaneAdded { .. }))
            .count();
        assert!(
            pane_added_count >= 1,
            "open_tab should produce at least one PaneAdded event"
        );

        // Step 2: Feed events through coordinator (as AppShell does)
        h.coordinate(events).expect("coordinate should succeed");

        // Step 3: Assert mock port was called
        let spawns = h.spawn_calls();
        assert_eq!(spawns.len(), 1, "spawn should be called exactly once");
        assert_eq!(
            spawns[0].1, "/projects",
            "working_directory passed to spawn"
        );

        // Step 4: Assert runtime registered correctly
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1, "one runtime should be registered");
        assert!(matches!(snapshot[0].kind, RuntimeKind::Terminal));
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));

        // Step 5: Assert projection emitted
        let projections = h.runtime_projections();
        assert_eq!(projections.len(), 1, "one projection should be emitted");
        assert_eq!(projections[0].1, RuntimeKind::Terminal);
        assert_eq!(projections[0].2, RuntimeStatus::Running);
    }

    #[test]
    fn open_tab_with_multiple_terminal_panes_spawns_each() {
        let h = DispatchHarness::new();

        let events = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByTwo,
                false,
                vec![terminal_spec("/a"), terminal_spec("/b")],
            )
            .expect("open_tab should succeed");

        h.coordinate(events).expect("coordinate should succeed");

        let spawns = h.spawn_calls();
        assert_eq!(spawns.len(), 2, "spawn should be called for each pane");

        let cwds: Vec<&str> = spawns.iter().map(|s| s.1.as_str()).collect();
        assert!(cwds.contains(&"/a"), "first pane cwd should be /a");
        assert!(cwds.contains(&"/b"), "second pane cwd should be /b");

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 2, "two runtimes should be registered");
    }

    #[test]
    fn open_tab_with_browser_pane_registers_browser_runtime() {
        let h = DispatchHarness::new();

        let events = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByOne,
                false,
                vec![browser_spec("https://example.com")],
            )
            .expect("open_tab should succeed");

        h.coordinate(events).expect("coordinate should succeed");

        // Terminal port should NOT be called
        assert!(
            h.spawn_calls().is_empty(),
            "terminal port should not be called for browser pane"
        );

        // Browser runtime registered
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert!(matches!(snapshot[0].kind, RuntimeKind::Browser));
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));
        assert_eq!(
            snapshot[0].browser_location.as_deref(),
            Some("https://example.com"),
        );
    }

    #[test]
    fn open_tab_with_mixed_panes_routes_to_correct_ports() {
        let h = DispatchHarness::new();

        let events = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByTwo,
                false,
                vec![terminal_spec("/home"), browser_spec("https://docs.rs")],
            )
            .expect("open_tab should succeed");

        h.coordinate(events).expect("coordinate should succeed");

        // Terminal port called once (for terminal pane only)
        let spawns = h.spawn_calls();
        assert_eq!(spawns.len(), 1, "spawn called for terminal pane only");
        assert_eq!(spawns[0].1, "/home");

        // Both runtimes registered
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 2);

        let terminal_count = snapshot
            .iter()
            .filter(|r| matches!(r.kind, RuntimeKind::Terminal))
            .count();
        let browser_count = snapshot
            .iter()
            .filter(|r| matches!(r.kind, RuntimeKind::Browser))
            .count();
        assert_eq!(terminal_count, 1);
        assert_eq!(browser_count, 1);
    }

    // =======================================================================
    // AC#2: close_pane command → PaneRemoved event → RuntimeCoordinator
    //       → mock TerminalProcessPort.kill() called
    // =======================================================================

    #[test]
    fn close_pane_command_dispatches_kill_through_coordinator() {
        let h = DispatchHarness::new();

        // Open a tab with a terminal pane
        let open_events = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByTwo,
                false,
                vec![terminal_spec("/a"), terminal_spec("/b")],
            )
            .expect("open_tab should succeed");
        h.coordinate(open_events)
            .expect("coordinate should succeed");

        assert_eq!(h.spawn_calls().len(), 2, "two spawns from open");

        // Get the pane ID from workspace state
        let pane_id = h
            .workspace_service
            .with_session(|session| session.tab_summaries()[0].panes[0].pane_id.clone())
            .expect("with_session");

        // Capture the runtime session ID for the pane about to be closed
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        let runtime_to_close = snapshot
            .iter()
            .find(|r| r.pane_id == pane_id)
            .expect("runtime should exist for pane");
        let session_id = runtime_to_close
            .runtime_session_id
            .as_ref()
            .expect("session id should exist")
            .clone();

        // Close the pane
        let close_events = h
            .workspace_service
            .close_pane(&pane_id)
            .expect("close_pane should succeed");
        h.coordinate(close_events)
            .expect("coordinate should succeed");

        // Assert kill was called with the correct session ID
        let kills = h.kill_calls();
        assert_eq!(kills.len(), 1, "kill should be called exactly once");
        assert_eq!(
            kills[0],
            session_id.as_ref(),
            "killed session should match the closed pane"
        );

        // Assert runtime was removed from registry
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(
            snapshot.len(),
            1,
            "one runtime should remain after closing one pane"
        );
        assert!(
            snapshot.iter().all(|r| r.pane_id != pane_id),
            "closed pane should not have a runtime"
        );
    }

    #[test]
    fn close_tab_command_kills_all_runtimes_in_tab() {
        let h = DispatchHarness::new();

        // Open a tab with two terminal panes
        let open_events = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByTwo,
                false,
                vec![terminal_spec("/a"), terminal_spec("/b")],
            )
            .expect("open_tab should succeed");
        h.coordinate(open_events)
            .expect("coordinate should succeed");

        assert_eq!(h.spawn_calls().len(), 2, "two spawns from open");
        assert_eq!(
            h.runtime_service.snapshot().expect("snapshot").len(),
            2,
            "two runtimes registered"
        );

        // Get the tab ID
        let tab_id = h
            .workspace_service
            .with_session(|session| session.tab_summaries()[0].tab_id.clone())
            .expect("with_session");

        // Close the entire tab
        let close_events = h
            .workspace_service
            .close_tab(&tab_id)
            .expect("close_tab should succeed");
        h.coordinate(close_events)
            .expect("coordinate should succeed");

        // Assert kill was called for both terminals
        let kills = h.kill_calls();
        assert_eq!(
            kills.len(),
            2,
            "kill should be called for each terminal in the tab"
        );

        // Assert all runtimes removed
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert!(
            snapshot.is_empty(),
            "no runtimes should remain after closing the tab"
        );
    }

    #[test]
    fn close_tab_with_browser_pane_calls_close_surface() {
        let h = DispatchHarness::new();

        // Open a tab with a browser pane
        let open_events = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByOne,
                false,
                vec![browser_spec("https://example.com")],
            )
            .expect("open_tab should succeed");
        h.coordinate(open_events)
            .expect("coordinate should succeed");

        // Get the tab ID
        let tab_id = h
            .workspace_service
            .with_session(|session| session.tab_summaries()[0].tab_id.clone())
            .expect("with_session");

        // Close the tab
        let close_events = h
            .workspace_service
            .close_tab(&tab_id)
            .expect("close_tab should succeed");
        h.coordinate(close_events)
            .expect("coordinate should succeed");

        // Assert browser surface was closed
        let closes = h.close_surface_calls();
        assert_eq!(
            closes.len(),
            1,
            "close_surface should be called for browser pane"
        );

        // Assert runtime removed
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert!(snapshot.is_empty(), "no runtimes should remain");
    }

    // =======================================================================
    // AC#3: Full lifecycle scenario — open → close pane → close tab
    //       Verifies the complete dispatch pipeline end-to-end with argument
    //       and call count assertions.
    // =======================================================================

    #[test]
    fn full_lifecycle_open_close_pane_close_tab_through_dispatch() {
        let h = DispatchHarness::new();

        // === Phase 1: Open tab with two terminal panes ===
        let open_events = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByTwo,
                false,
                vec![terminal_spec("/workspace-a"), terminal_spec("/workspace-b")],
            )
            .expect("open_tab should succeed");
        h.coordinate(open_events)
            .expect("coordinate should succeed");

        assert_eq!(h.spawn_calls().len(), 2, "phase 1: two spawns");
        let spawns = h.spawn_calls();
        let cwds: Vec<&str> = spawns.iter().map(|s| s.1.as_str()).collect();
        assert!(cwds.contains(&"/workspace-a"));
        assert!(cwds.contains(&"/workspace-b"));

        // === Phase 2: Close one pane ===
        let pane_to_close = h
            .workspace_service
            .with_session(|session| session.tab_summaries()[0].panes[0].pane_id.clone())
            .expect("with_session");

        let close_pane_events = h
            .workspace_service
            .close_pane(&pane_to_close)
            .expect("close_pane should succeed");
        h.coordinate(close_pane_events)
            .expect("coordinate should succeed");

        assert_eq!(h.kill_calls().len(), 1, "phase 2: one kill from close_pane");
        assert_eq!(
            h.runtime_service.snapshot().expect("snapshot").len(),
            1,
            "phase 2: one runtime remaining"
        );

        // === Phase 3: Open a second tab with a browser pane ===
        let open_events_2 = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByOne,
                false,
                vec![browser_spec("https://example.com")],
            )
            .expect("open second tab");
        h.coordinate(open_events_2)
            .expect("coordinate should succeed");

        assert_eq!(
            h.runtime_service.snapshot().expect("snapshot").len(),
            2,
            "phase 3: two runtimes (1 terminal + 1 browser)"
        );

        // === Phase 4: Close the first tab (remaining terminal pane) ===
        let first_tab_id = h
            .workspace_service
            .with_session(|session| session.tab_summaries()[0].tab_id.clone())
            .expect("with_session");

        let close_tab_events = h
            .workspace_service
            .close_tab(&first_tab_id)
            .expect("close_tab should succeed");
        h.coordinate(close_tab_events)
            .expect("coordinate should succeed");

        assert_eq!(
            h.kill_calls().len(),
            2,
            "phase 4: total 2 kills (1 from close_pane + 1 from close_tab)"
        );
        assert_eq!(
            h.runtime_service.snapshot().expect("snapshot").len(),
            1,
            "phase 4: only browser runtime remains"
        );

        // === Phase 5: Close the second tab (browser pane) ===
        let second_tab_id = h
            .workspace_service
            .with_session(|session| session.tab_summaries()[0].tab_id.clone())
            .expect("with_session");

        let close_tab_events_2 = h
            .workspace_service
            .close_tab(&second_tab_id)
            .expect("close_tab should succeed");
        h.coordinate(close_tab_events_2)
            .expect("coordinate should succeed");

        assert_eq!(
            h.close_surface_calls().len(),
            1,
            "phase 5: browser close_surface called"
        );
        assert!(
            h.runtime_service.snapshot().expect("snapshot").is_empty(),
            "phase 5: all runtimes cleaned up"
        );

        // === Final assertion: projection counts ===
        let projections = h.runtime_projections();
        // Each start emits Running, each stop emits Exited
        let running_count = projections
            .iter()
            .filter(|(_, _, s)| matches!(s, RuntimeStatus::Running))
            .count();
        let exited_count = projections
            .iter()
            .filter(|(_, _, s)| matches!(s, RuntimeStatus::Exited))
            .count();
        assert_eq!(
            running_count, 3,
            "3 Running projections (2 terminals + 1 browser)"
        );
        assert_eq!(
            exited_count, 3,
            "3 Exited projections (2 terminal kills + 1 browser close)"
        );
    }

    // =======================================================================
    // AC#4: replace_pane_spec command → PaneContentChanged → stop old +
    //       start new through full dispatch
    // =======================================================================

    #[test]
    fn replace_pane_spec_command_stops_old_starts_new_through_dispatch() {
        let h = DispatchHarness::new();

        // Open a terminal pane
        let open_events = h
            .workspace_service
            .open_tab(LayoutPreset::OneByOne, false, vec![terminal_spec("/tmp")])
            .expect("open_tab should succeed");
        h.coordinate(open_events)
            .expect("coordinate should succeed");

        assert_eq!(h.spawn_calls().len(), 1, "initial spawn");

        // Get the pane ID
        let pane_id = h
            .workspace_service
            .with_session(|session| session.tab_summaries()[0].panes[0].pane_id.clone())
            .expect("with_session");

        // Capture old session ID
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        let old_session = snapshot[0]
            .runtime_session_id
            .as_ref()
            .expect("session id")
            .clone();

        // Replace terminal with browser
        let replace_events = h
            .workspace_service
            .replace_pane_spec(&pane_id, browser_spec("https://example.com"))
            .expect("replace_pane_spec should succeed");
        h.coordinate(replace_events)
            .expect("coordinate should succeed");

        // Old terminal was killed
        let kills = h.kill_calls();
        assert_eq!(kills.len(), 1, "old terminal should be killed");
        assert_eq!(
            kills[0],
            old_session.as_ref(),
            "killed session should match old terminal"
        );

        // New browser runtime registered
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert!(matches!(snapshot[0].kind, RuntimeKind::Browser));
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));
    }

    // =======================================================================
    // AC#5: split_pane command → PaneAdded → spawn for new pane
    // =======================================================================

    #[test]
    fn split_pane_command_spawns_new_runtime_through_dispatch() {
        let h = DispatchHarness::new();

        // Open a tab
        let open_events = h
            .workspace_service
            .open_tab(
                LayoutPreset::OneByOne,
                false,
                vec![terminal_spec("/initial")],
            )
            .expect("open_tab should succeed");
        h.coordinate(open_events)
            .expect("coordinate should succeed");

        assert_eq!(h.spawn_calls().len(), 1, "initial spawn");

        // Get the pane ID to split
        let pane_id = h
            .workspace_service
            .with_session(|session| session.tab_summaries()[0].panes[0].pane_id.clone())
            .expect("with_session");

        // Split the pane
        let split_events = h
            .workspace_service
            .split_pane(
                &pane_id,
                tabby_workspace::layout::SplitDirection::Horizontal,
                terminal_spec("/split"),
            )
            .expect("split_pane should succeed");
        h.coordinate(split_events)
            .expect("coordinate should succeed");

        // A second spawn should have occurred
        let spawns = h.spawn_calls();
        assert_eq!(spawns.len(), 2, "spawn called for split pane");
        assert_eq!(spawns[1].1, "/split", "split pane should use specified cwd");

        // Two runtimes registered
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 2, "two runtimes after split");
        assert!(snapshot
            .iter()
            .all(|r| matches!(r.kind, RuntimeKind::Terminal)));
    }
}
