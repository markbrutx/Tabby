//! Integration tests for the full runtime lifecycle through ports (US-029).
//!
//! These tests exercise the real `RuntimeCoordinator::handle_workspace_events`
//! calling the real `RuntimeApplicationService`, which delegates to mock ports.
//! No real Tauri, PTY, or browser surface is involved.
//!
//! Coverage:
//! - AC#1: PaneAdded(terminal) → TerminalProcessPort.spawn called
//! - AC#2: PaneAdded(browser) → browser runtime registered, projection emitted
//! - AC#3: on_terminal_exited → registry updated → ProjectionPublisherPort.publish_runtime_status
//! - AC#4: PaneContentChanged → old runtime stopped via port → new runtime started via port
//! - AC#5: observe_terminal_cwd → updates runtime registry only (settings persistence handled by AppShell)

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tabby_runtime::{RuntimeKind, RuntimeStatus};
    use tabby_settings::UserPreferences;
    use tabby_workspace::{
        BrowserUrl, PaneContentDefinition, PaneContentId, PaneId, WorkspaceDomainEvent,
    };

    use crate::application::ports::{
        BrowserSurfacePort, PreferencesRepository, ProjectionPublisherPort, TerminalProcessPort,
    };
    use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
    use crate::application::{
        RuntimeApplicationService, RuntimeCoordinator, SettingsApplicationService,
    };
    use crate::shell::error::ShellError;

    // -----------------------------------------------------------------------
    // Mock ports
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
        fn publish_workspace_projection(&self, _workspace: &tabby_workspace::WorkspaceSession) {}
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
        save_count: Mutex<u32>,
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
            if let Ok(mut count) = self.save_count.lock() {
                *count += 1;
            }
            Ok(())
        }
    }

    struct MockObservationReceiver;

    impl RuntimeObservationReceiver for MockObservationReceiver {
        fn on_terminal_output_received(&self, _pane_id: &PaneId, _data: &[u8]) {}
        fn on_terminal_exited(&self, _pane_id: &PaneId, _exit_code: Option<i32>) {}
        fn on_browser_location_changed(&self, _pane_id: &PaneId, _url: &str) {}
        fn on_terminal_cwd_changed(&self, _pane_id: &PaneId, _cwd: &str) {}
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
        fn publish_workspace_projection(&self, w: &tabby_workspace::WorkspaceSession) {
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
    // Test harness
    // -----------------------------------------------------------------------

    struct TestHarness {
        runtime_service: RuntimeApplicationService,
        settings_service: SettingsApplicationService,
        terminal_port: Arc<MockTerminalPort>,
        browser_port: Arc<MockBrowserPort>,
        emitter: Arc<MockEmitter>,
        preferences_repo: Arc<MockPreferencesRepo>,
    }

    impl TestHarness {
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

            Self {
                runtime_service,
                settings_service,
                terminal_port,
                browser_port,
                emitter,
                preferences_repo,
            }
        }

        fn handle_events(&self, events: Vec<WorkspaceDomainEvent>) -> Result<(), ShellError> {
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

        fn save_count(&self) -> u32 {
            *self.preferences_repo.save_count.lock().expect("lock")
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn terminal_content(cwd: &str) -> PaneContentDefinition {
        PaneContentDefinition::terminal(
            PaneContentId::from(uuid::Uuid::new_v4().to_string()),
            tabby_settings::TERMINAL_PROFILE_ID,
            cwd,
            None,
        )
    }

    fn browser_content(url: &str) -> PaneContentDefinition {
        PaneContentDefinition::browser(
            PaneContentId::from(uuid::Uuid::new_v4().to_string()),
            BrowserUrl::new(url),
        )
    }

    fn pid(id: &str) -> PaneId {
        PaneId::from(String::from(id))
    }

    // =======================================================================
    // AC#1: RuntimeCoordinator → RuntimeApplicationService → mock
    //       TerminalProcessPort: PaneAdded(terminal) → spawn called
    // =======================================================================

    #[test]
    fn coordinator_pane_added_terminal_calls_spawn_on_terminal_port() {
        let h = TestHarness::new();

        let events = vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-t1"),
            content: terminal_content("/projects"),
        }];
        h.handle_events(events).expect("should succeed");

        // TerminalProcessPort.spawn was called
        let spawns = h.spawn_calls();
        assert_eq!(spawns.len(), 1, "spawn should be called once");
        assert_eq!(spawns[0].0, "pane-t1", "pane_id passed to spawn");
        assert_eq!(
            spawns[0].1, "/projects",
            "working_directory passed to spawn"
        );

        // Runtime registered and projection emitted
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].pane_id, pid("pane-t1"));
        assert!(matches!(snapshot[0].kind, RuntimeKind::Terminal));
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));

        let projections = h.runtime_projections();
        assert_eq!(projections.len(), 1);
        assert_eq!(projections[0].0, "pane-t1");
        assert_eq!(projections[0].2, RuntimeStatus::Running);
    }

    #[test]
    fn coordinator_pane_added_terminal_with_custom_cwd() {
        let h = TestHarness::new();

        let events = vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-cwd"),
            content: terminal_content("/home/user/code"),
        }];
        h.handle_events(events).expect("should succeed");

        let spawns = h.spawn_calls();
        assert_eq!(spawns[0].1, "/home/user/code");
    }

    #[test]
    fn coordinator_multiple_pane_added_terminal_calls_spawn_for_each() {
        let h = TestHarness::new();

        let events = vec![
            WorkspaceDomainEvent::PaneAdded {
                pane_id: pid("pane-1"),
                content: terminal_content("/a"),
            },
            WorkspaceDomainEvent::PaneAdded {
                pane_id: pid("pane-2"),
                content: terminal_content("/b"),
            },
        ];
        h.handle_events(events).expect("should succeed");

        let spawns = h.spawn_calls();
        assert_eq!(spawns.len(), 2, "spawn called for each terminal pane");
        assert_eq!(spawns[0].0, "pane-1");
        assert_eq!(spawns[1].0, "pane-2");

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 2);
    }

    // =======================================================================
    // AC#2: RuntimeCoordinator → RuntimeApplicationService → mock
    //       BrowserSurfacePort: PaneAdded(browser) → browser runtime
    //       registered, projection emitted
    //
    // Note: ensure_surface is not called during start_runtime (layout
    // coordinates are managed separately). The browser runtime is registered
    // in the registry and a Running projection is emitted.
    // =======================================================================

    #[test]
    fn coordinator_pane_added_browser_registers_runtime_and_emits_projection() {
        let h = TestHarness::new();

        let events = vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-b1"),
            content: browser_content("https://example.com"),
        }];
        h.handle_events(events).expect("should succeed");

        // Terminal port was NOT called
        assert!(
            h.spawn_calls().is_empty(),
            "terminal port should not be called for browser"
        );

        // Browser runtime registered
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].pane_id, pid("pane-b1"));
        assert!(matches!(snapshot[0].kind, RuntimeKind::Browser));
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));
        assert_eq!(
            snapshot[0].browser_location.as_deref(),
            Some("https://example.com"),
        );

        // Projection emitted
        let projections = h.runtime_projections();
        assert_eq!(projections.len(), 1);
        assert_eq!(projections[0].0, "pane-b1");
        assert_eq!(projections[0].1, RuntimeKind::Browser);
        assert_eq!(projections[0].2, RuntimeStatus::Running);
    }

    #[test]
    fn coordinator_mixed_pane_added_routes_to_correct_ports() {
        let h = TestHarness::new();

        let events = vec![
            WorkspaceDomainEvent::PaneAdded {
                pane_id: pid("pane-t"),
                content: terminal_content("/tmp"),
            },
            WorkspaceDomainEvent::PaneAdded {
                pane_id: pid("pane-b"),
                content: browser_content("https://docs.rs"),
            },
        ];
        h.handle_events(events).expect("should succeed");

        // Terminal port called only for terminal pane
        let spawns = h.spawn_calls();
        assert_eq!(spawns.len(), 1);
        assert_eq!(spawns[0].0, "pane-t");

        // Both runtimes registered
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 2);

        let terminal = snapshot.iter().find(|r| r.pane_id == pid("pane-t"));
        let browser = snapshot.iter().find(|r| r.pane_id == pid("pane-b"));
        assert!(terminal.is_some());
        assert!(browser.is_some());
        assert!(matches!(
            terminal.expect("found").kind,
            RuntimeKind::Terminal
        ));
        assert!(matches!(browser.expect("found").kind, RuntimeKind::Browser));
    }

    // =======================================================================
    // AC#3: Observation flow: RuntimeObservationReceiver.on_terminal_exited
    //       called → registry updated → mock ProjectionPublisherPort
    //       .publish_runtime_status called
    // =======================================================================

    #[test]
    fn on_terminal_exited_updates_registry_and_publishes_projection() {
        let h = TestHarness::new();

        // Start a terminal runtime through the coordinator
        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-exit"),
            content: terminal_content("/tmp"),
        }])
        .expect("start");

        // Clear projections from start
        h.emitter.runtime_statuses.lock().expect("lock").clear();

        // Simulate PTY exit via the observation receiver trait on the real service
        let pane_id = pid("pane-exit");
        RuntimeObservationReceiver::on_terminal_exited(&h.runtime_service, &pane_id, Some(0));

        // Registry updated to Exited
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].status, RuntimeStatus::Exited);

        // ProjectionPublisherPort.publish_runtime_status was called
        let projections = h.runtime_projections();
        assert_eq!(projections.len(), 1, "exactly one projection for exit");
        assert_eq!(projections[0].0, "pane-exit");
        assert_eq!(projections[0].2, RuntimeStatus::Exited);
    }

    #[test]
    fn on_terminal_exited_nonzero_marks_failed_and_publishes() {
        let h = TestHarness::new();

        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-fail"),
            content: terminal_content("/tmp"),
        }])
        .expect("start");
        h.emitter.runtime_statuses.lock().expect("lock").clear();

        let pane_id = pid("pane-fail");
        RuntimeObservationReceiver::on_terminal_exited(&h.runtime_service, &pane_id, Some(127));

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot[0].status, RuntimeStatus::Failed);
        assert_eq!(
            snapshot[0].last_error.as_deref(),
            Some("Process exited with code 127"),
        );

        let projections = h.runtime_projections();
        assert_eq!(projections.len(), 1);
        assert_eq!(projections[0].2, RuntimeStatus::Failed);
    }

    #[test]
    fn on_terminal_exited_unknown_code_marks_exited() {
        let h = TestHarness::new();

        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-u"),
            content: terminal_content("/tmp"),
        }])
        .expect("start");
        h.emitter.runtime_statuses.lock().expect("lock").clear();

        let pane_id = pid("pane-u");
        RuntimeObservationReceiver::on_terminal_exited(&h.runtime_service, &pane_id, None);

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot[0].status, RuntimeStatus::Exited);
        assert!(snapshot[0].last_error.is_none());
    }

    // =======================================================================
    // AC#4: Replace flow: PaneContentChanged → old runtime stopped via port →
    //       new runtime started via port
    // =======================================================================

    #[test]
    fn replace_terminal_with_browser_stops_old_via_port_and_starts_new() {
        let h = TestHarness::new();

        // Start terminal runtime
        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-r"),
            content: terminal_content("/projects"),
        }])
        .expect("start terminal");

        // Capture the terminal session ID for verification
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        let old_session_id = snapshot[0]
            .runtime_session_id
            .as_ref()
            .expect("session id")
            .clone();

        // Replace terminal with browser
        h.handle_events(vec![WorkspaceDomainEvent::PaneContentChanged {
            pane_id: pid("pane-r"),
            old_content: terminal_content("/projects"),
            new_content: browser_content("https://example.com"),
        }])
        .expect("replace");

        // Old terminal was killed via TerminalProcessPort.kill
        let kills = h.kill_calls();
        assert_eq!(
            kills.len(),
            1,
            "kill should be called once for old terminal"
        );
        assert_eq!(
            kills[0],
            old_session_id.as_ref(),
            "killed session should match the old terminal"
        );

        // New browser runtime is running
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].pane_id, pid("pane-r"));
        assert!(matches!(snapshot[0].kind, RuntimeKind::Browser));
        assert!(matches!(snapshot[0].status, RuntimeStatus::Running));
    }

    #[test]
    fn replace_browser_with_terminal_closes_surface_and_spawns_pty() {
        let h = TestHarness::new();

        // Start browser runtime
        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-rb"),
            content: browser_content("https://example.com"),
        }])
        .expect("start browser");

        // Replace browser with terminal
        h.handle_events(vec![WorkspaceDomainEvent::PaneContentChanged {
            pane_id: pid("pane-rb"),
            old_content: browser_content("https://example.com"),
            new_content: terminal_content("/home"),
        }])
        .expect("replace");

        // Browser surface was closed via BrowserSurfacePort.close_surface
        let closes = h.close_surface_calls();
        assert_eq!(closes.len(), 1, "close_surface called for old browser");
        assert_eq!(closes[0], "pane-rb");

        // New terminal was spawned
        let spawns = h.spawn_calls();
        assert_eq!(spawns.len(), 1, "spawn called for new terminal");
        assert_eq!(spawns[0].0, "pane-rb");
        assert_eq!(spawns[0].1, "/home");

        // Registry has terminal runtime
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert!(matches!(snapshot[0].kind, RuntimeKind::Terminal));
    }

    #[test]
    fn replace_terminal_with_terminal_kills_old_spawns_new() {
        let h = TestHarness::new();

        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-tt"),
            content: terminal_content("/a"),
        }])
        .expect("start");

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        let old_session = snapshot[0]
            .runtime_session_id
            .as_ref()
            .expect("session")
            .clone();

        h.handle_events(vec![WorkspaceDomainEvent::PaneContentChanged {
            pane_id: pid("pane-tt"),
            old_content: terminal_content("/a"),
            new_content: terminal_content("/b"),
        }])
        .expect("replace");

        // Old killed, new spawned
        let kills = h.kill_calls();
        assert_eq!(kills.len(), 1);
        assert_eq!(kills[0], old_session.as_ref());

        let spawns = h.spawn_calls();
        assert_eq!(spawns.len(), 2, "2 spawns: original + replacement");
        assert_eq!(spawns[1].1, "/b", "new terminal uses new cwd");

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);
        assert!(matches!(snapshot[0].kind, RuntimeKind::Terminal));
    }

    #[test]
    fn replace_emits_correct_projection_sequence() {
        let h = TestHarness::new();

        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-seq"),
            content: terminal_content("/tmp"),
        }])
        .expect("start");

        h.handle_events(vec![WorkspaceDomainEvent::PaneContentChanged {
            pane_id: pid("pane-seq"),
            old_content: terminal_content("/tmp"),
            new_content: browser_content("https://example.com"),
        }])
        .expect("replace");

        // Projection sequence: Running(terminal) → Exited(stop) → Running(browser)
        let projections = h.runtime_projections();
        let pane_projections: Vec<_> = projections
            .iter()
            .filter(|(id, _, _)| id == "pane-seq")
            .collect();
        assert!(
            pane_projections.len() >= 3,
            "should have at least 3 projections: start, stop, start"
        );

        // First is Running (terminal started)
        assert_eq!(pane_projections[0].2, RuntimeStatus::Running);
        assert_eq!(pane_projections[0].1, RuntimeKind::Terminal);

        // Second is Exited (terminal stopped)
        assert_eq!(pane_projections[1].2, RuntimeStatus::Exited);

        // Third is Running (browser started)
        assert_eq!(pane_projections[2].2, RuntimeStatus::Running);
        assert_eq!(pane_projections[2].1, RuntimeKind::Browser);
    }

    // =======================================================================
    // AC#5: Settings persistence: preferences saved via mock
    //       PreferencesRepository after cwd observation
    // =======================================================================

    #[test]
    fn observe_terminal_cwd_does_not_touch_settings() {
        let h = TestHarness::new();

        // Start a terminal runtime
        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-cwd"),
            content: terminal_content("/initial"),
        }])
        .expect("start");

        // Record save count before observation
        let saves_before = h.save_count();

        // Observe a cwd change — should NOT persist settings (DDD-008)
        h.runtime_service
            .observe_terminal_cwd(&pid("pane-cwd"), "/new/path")
            .expect("observe cwd should succeed");

        // Settings were NOT touched by RuntimeApplicationService
        let saves_after = h.save_count();
        assert_eq!(
            saves_after, saves_before,
            "runtime_service.observe_terminal_cwd must not persist settings (cross-context side effect belongs in AppShell)"
        );
    }

    #[test]
    fn observe_terminal_cwd_updates_runtime_registry() {
        let h = TestHarness::new();

        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-cwd2"),
            content: terminal_content("/initial"),
        }])
        .expect("start");

        h.runtime_service
            .observe_terminal_cwd(&pid("pane-cwd2"), "/updated/cwd")
            .expect("observe");

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        let runtime = snapshot
            .iter()
            .find(|r| r.pane_id == pid("pane-cwd2"))
            .expect("found");
        assert_eq!(
            runtime.terminal_cwd.as_deref(),
            Some("/updated/cwd"),
            "runtime registry should reflect observed cwd"
        );
    }

    #[test]
    fn observe_terminal_cwd_emits_runtime_status_projection() {
        let h = TestHarness::new();

        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-proj"),
            content: terminal_content("/initial"),
        }])
        .expect("start");

        h.emitter.runtime_statuses.lock().expect("lock").clear();

        h.runtime_service
            .observe_terminal_cwd(&pid("pane-proj"), "/observed")
            .expect("observe");

        let projections = h.runtime_projections();
        assert_eq!(
            projections.len(),
            1,
            "cwd observation should emit projection"
        );
        assert_eq!(projections[0].0, "pane-proj");
        assert_eq!(projections[0].2, RuntimeStatus::Running);
    }

    // =======================================================================
    // Edge cases: PaneRemoved stops runtime via correct port
    // =======================================================================

    #[test]
    fn pane_removed_terminal_kills_via_terminal_port() {
        let h = TestHarness::new();

        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-rm"),
            content: terminal_content("/tmp"),
        }])
        .expect("start");

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        let session_id = snapshot[0]
            .runtime_session_id
            .as_ref()
            .expect("session")
            .clone();

        h.handle_events(vec![WorkspaceDomainEvent::PaneRemoved {
            pane_id: pid("pane-rm"),
            content: terminal_content("/tmp"),
        }])
        .expect("remove");

        let kills = h.kill_calls();
        assert_eq!(kills.len(), 1);
        assert_eq!(kills[0], session_id.as_ref());

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert!(snapshot.is_empty(), "registry should be empty after remove");
    }

    #[test]
    fn pane_removed_browser_closes_via_browser_port() {
        let h = TestHarness::new();

        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-br"),
            content: browser_content("https://example.com"),
        }])
        .expect("start");

        h.handle_events(vec![WorkspaceDomainEvent::PaneRemoved {
            pane_id: pid("pane-br"),
            content: browser_content("https://example.com"),
        }])
        .expect("remove");

        let closes = h.close_surface_calls();
        assert_eq!(closes.len(), 1);
        assert_eq!(closes[0], "pane-br");
    }

    // =======================================================================
    // Focus events have no runtime side-effects through coordinator
    // =======================================================================

    #[test]
    fn focus_events_do_not_trigger_port_calls() {
        let h = TestHarness::new();

        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-focus"),
            content: terminal_content("/tmp"),
        }])
        .expect("start");

        let spawns_before = h.spawn_calls().len();
        let kills_before = h.kill_calls().len();

        h.handle_events(vec![
            WorkspaceDomainEvent::ActivePaneChanged {
                pane_id: pid("pane-focus"),
                tab_id: tabby_workspace::TabId::from(String::from("tab-1")),
            },
            WorkspaceDomainEvent::ActiveTabChanged {
                tab_id: tabby_workspace::TabId::from(String::from("tab-1")),
            },
        ])
        .expect("focus events");

        assert_eq!(
            h.spawn_calls().len(),
            spawns_before,
            "focus events must not call spawn"
        );
        assert_eq!(
            h.kill_calls().len(),
            kills_before,
            "focus events must not call kill"
        );
    }

    // =======================================================================
    // Full lifecycle scenario through coordinator and real service
    // =======================================================================

    #[test]
    fn full_lifecycle_through_coordinator_and_service_with_mock_ports() {
        let h = TestHarness::new();

        // 1. Add terminal pane
        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-lc"),
            content: terminal_content("/start"),
        }])
        .expect("add terminal");
        assert_eq!(h.spawn_calls().len(), 1);

        // 2. Add browser pane
        h.handle_events(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: pid("pane-lc2"),
            content: browser_content("https://example.com"),
        }])
        .expect("add browser");
        assert_eq!(h.runtime_service.snapshot().expect("snapshot").len(), 2);

        // 3. Terminal exits naturally
        RuntimeObservationReceiver::on_terminal_exited(
            &h.runtime_service,
            &pid("pane-lc"),
            Some(0),
        );
        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        let terminal_rt = snapshot.iter().find(|r| r.pane_id == pid("pane-lc"));
        assert_eq!(terminal_rt.expect("found").status, RuntimeStatus::Exited,);

        // 4. Replace browser with terminal
        h.handle_events(vec![WorkspaceDomainEvent::PaneContentChanged {
            pane_id: pid("pane-lc2"),
            old_content: browser_content("https://example.com"),
            new_content: terminal_content("/replaced"),
        }])
        .expect("replace");
        assert_eq!(h.close_surface_calls().len(), 1, "browser closed");
        assert_eq!(h.spawn_calls().len(), 2, "new terminal spawned");

        // 5. Remove all panes
        h.handle_events(vec![
            WorkspaceDomainEvent::PaneRemoved {
                pane_id: pid("pane-lc"),
                content: terminal_content("/start"),
            },
            WorkspaceDomainEvent::PaneRemoved {
                pane_id: pid("pane-lc2"),
                content: terminal_content("/replaced"),
            },
        ])
        .expect("remove all");

        let snapshot = h.runtime_service.snapshot().expect("snapshot");
        assert!(snapshot.is_empty(), "all runtimes should be cleaned up");
    }
}
