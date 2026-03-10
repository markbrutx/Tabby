use tabby_workspace::PaneId;

/// Application-owned callback interface for receiving raw observation facts
/// from infrastructure (PTY read thread, browser surface adapter).
///
/// Dependency direction: infrastructure → this trait (infra calls the methods).
/// `RuntimeApplicationService` implements this trait to receive and process observations.
///
/// All parameters use domain types only — no Tauri `AppHandle`, no DTO types,
/// no transport event shapes.
///
/// `on_terminal_exited` is wired to PTY infrastructure (US-005).
/// Remaining methods will be wired to infrastructure in subsequent stories.
#[allow(dead_code)]
pub trait RuntimeObservationReceiver: Send + Sync {
    /// Called by the PTY read thread when terminal output data is available.
    ///
    /// **Note:** This method is currently a no-op. Terminal output bypasses this
    /// trait and is emitted directly to the frontend for performance reasons.
    /// Reserved for future OSC sequence detection.
    /// See `docs/adr/001-terminal-output-hot-path.md`.
    fn on_terminal_output_received(&self, pane_id: &PaneId, data: &[u8]);

    /// Called by the PTY read thread when the terminal process has exited.
    /// `exit_code` is `None` when the exit status could not be determined.
    fn on_terminal_exited(
        &self,
        pane_id: &PaneId,
        runtime_session_id: &str,
        exit_code: Option<i32>,
    );

    /// Called by the browser surface adapter when the URL changes.
    fn on_browser_location_changed(&self, pane_id: &PaneId, url: &str);

    /// Called by the PTY read thread (via OSC sequence detection or shell integration)
    /// when the terminal's working directory changes.
    fn on_terminal_cwd_changed(&self, pane_id: &PaneId, cwd: &str);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Records all observations for test assertions.
    #[derive(Debug, Default)]
    struct ObservationLog {
        terminal_outputs: Vec<(String, Vec<u8>)>,
        terminal_exits: Vec<(String, Option<i32>)>,
        browser_locations: Vec<(String, String)>,
        terminal_cwds: Vec<(String, String)>,
    }

    /// Mock receiver that captures calls for verification.
    struct MockObservationReceiver {
        log: Mutex<ObservationLog>,
    }

    impl MockObservationReceiver {
        fn new() -> Self {
            Self {
                log: Mutex::new(ObservationLog::default()),
            }
        }

        fn into_log(self) -> ObservationLog {
            self.log.into_inner().unwrap_or_default()
        }
    }

    impl RuntimeObservationReceiver for MockObservationReceiver {
        fn on_terminal_output_received(&self, pane_id: &PaneId, data: &[u8]) {
            if let Ok(mut log) = self.log.lock() {
                log.terminal_outputs
                    .push((pane_id.to_string(), data.to_vec()));
            }
        }

        fn on_terminal_exited(
            &self,
            pane_id: &PaneId,
            _runtime_session_id: &str,
            exit_code: Option<i32>,
        ) {
            if let Ok(mut log) = self.log.lock() {
                log.terminal_exits.push((pane_id.to_string(), exit_code));
            }
        }

        fn on_browser_location_changed(&self, pane_id: &PaneId, url: &str) {
            if let Ok(mut log) = self.log.lock() {
                log.browser_locations
                    .push((pane_id.to_string(), String::from(url)));
            }
        }

        fn on_terminal_cwd_changed(&self, pane_id: &PaneId, cwd: &str) {
            if let Ok(mut log) = self.log.lock() {
                log.terminal_cwds
                    .push((pane_id.to_string(), String::from(cwd)));
            }
        }
    }

    fn pane(id: &str) -> PaneId {
        PaneId::from(String::from(id))
    }

    #[test]
    fn mock_infra_can_call_on_terminal_output_received() {
        let receiver = MockObservationReceiver::new();
        let pane_id = pane("pane-1");

        receiver.on_terminal_output_received(&pane_id, b"hello world");

        let log = receiver.into_log();
        assert_eq!(log.terminal_outputs.len(), 1);
        assert_eq!(log.terminal_outputs[0].0, "pane-1");
        assert_eq!(log.terminal_outputs[0].1, b"hello world");
    }

    #[test]
    fn mock_infra_can_call_on_terminal_exited_with_code() {
        let receiver = MockObservationReceiver::new();
        let pane_id = pane("pane-2");

        receiver.on_terminal_exited(&pane_id, "session-2", Some(0));

        let log = receiver.into_log();
        assert_eq!(log.terminal_exits.len(), 1);
        assert_eq!(log.terminal_exits[0].0, "pane-2");
        assert_eq!(log.terminal_exits[0].1, Some(0));
    }

    #[test]
    fn mock_infra_can_call_on_terminal_exited_without_code() {
        let receiver = MockObservationReceiver::new();
        let pane_id = pane("pane-3");

        receiver.on_terminal_exited(&pane_id, "session-3", None);

        let log = receiver.into_log();
        assert_eq!(log.terminal_exits.len(), 1);
        assert_eq!(log.terminal_exits[0].1, None);
    }

    #[test]
    fn mock_infra_can_call_on_browser_location_changed() {
        let receiver = MockObservationReceiver::new();
        let pane_id = pane("pane-4");

        receiver.on_browser_location_changed(&pane_id, "https://example.com");

        let log = receiver.into_log();
        assert_eq!(log.browser_locations.len(), 1);
        assert_eq!(log.browser_locations[0].0, "pane-4");
        assert_eq!(log.browser_locations[0].1, "https://example.com");
    }

    #[test]
    fn mock_infra_can_call_on_terminal_cwd_changed() {
        let receiver = MockObservationReceiver::new();
        let pane_id = pane("pane-5");

        receiver.on_terminal_cwd_changed(&pane_id, "/home/user/projects");

        let log = receiver.into_log();
        assert_eq!(log.terminal_cwds.len(), 1);
        assert_eq!(log.terminal_cwds[0].0, "pane-5");
        assert_eq!(log.terminal_cwds[0].1, "/home/user/projects");
    }

    #[test]
    fn receiver_trait_is_object_safe_behind_arc() {
        let receiver: Arc<dyn RuntimeObservationReceiver> =
            Arc::new(MockObservationReceiver::new());
        let pane_id = pane("pane-arc");

        receiver.on_terminal_output_received(&pane_id, b"test");
        receiver.on_terminal_exited(&pane_id, "session-arc", Some(1));
        receiver.on_browser_location_changed(&pane_id, "https://test.com");
        receiver.on_terminal_cwd_changed(&pane_id, "/tmp");
    }

    #[test]
    fn multiple_observations_accumulate_in_order() {
        let receiver = MockObservationReceiver::new();
        let pane_id = pane("pane-multi");

        receiver.on_terminal_output_received(&pane_id, b"chunk1");
        receiver.on_terminal_output_received(&pane_id, b"chunk2");
        receiver.on_terminal_cwd_changed(&pane_id, "/a");
        receiver.on_terminal_cwd_changed(&pane_id, "/b");

        let log = receiver.into_log();
        assert_eq!(log.terminal_outputs.len(), 2);
        assert_eq!(log.terminal_outputs[0].1, b"chunk1");
        assert_eq!(log.terminal_outputs[1].1, b"chunk2");
        assert_eq!(log.terminal_cwds.len(), 2);
        assert_eq!(log.terminal_cwds[0].1, "/a");
        assert_eq!(log.terminal_cwds[1].1, "/b");
    }

    // --- Integration-style test: PTY exit → on_terminal_exited → registry update → projection ---

    use tabby_runtime::{RuntimeRegistry, RuntimeSessionId, RuntimeStatus};

    /// A receiver backed by a real `RuntimeRegistry`, simulating what
    /// `RuntimeApplicationService.on_terminal_exited()` does without
    /// requiring a Tauri `AppHandle`.
    struct RegistryBackedReceiver {
        registry: Mutex<RuntimeRegistry>,
        projections_emitted: Mutex<Vec<(String, RuntimeStatus)>>,
    }

    impl RegistryBackedReceiver {
        fn new(registry: RuntimeRegistry) -> Self {
            Self {
                registry: Mutex::new(registry),
                projections_emitted: Mutex::new(Vec::new()),
            }
        }
    }

    impl RuntimeObservationReceiver for RegistryBackedReceiver {
        fn on_terminal_output_received(&self, _pane_id: &PaneId, _data: &[u8]) {}

        fn on_terminal_exited(
            &self,
            pane_id: &PaneId,
            runtime_session_id: &str,
            exit_code: Option<i32>,
        ) {
            let failed = exit_code.is_some_and(|code| code != 0);
            let message = exit_code
                .filter(|code| *code != 0)
                .map(|code| format!("Process exited with code {code}"));

            let session_id = RuntimeSessionId::from(String::from(runtime_session_id));
            if let Ok(mut runtimes) = self.registry.lock() {
                if let Ok(runtime) =
                    runtimes.mark_terminal_exit(pane_id, Some(&session_id), failed, message)
                {
                    // Simulate projection emit
                    if let Ok(mut projections) = self.projections_emitted.lock() {
                        projections.push((runtime.pane_id.to_string(), runtime.status));
                    }
                }
            }
        }

        fn on_browser_location_changed(&self, _pane_id: &PaneId, _url: &str) {}
        fn on_terminal_cwd_changed(&self, _pane_id: &PaneId, _cwd: &str) {}
    }

    #[test]
    fn pty_exit_triggers_registry_update_and_projection_via_receiver() {
        // Setup: register a terminal runtime in the registry
        let mut registry = RuntimeRegistry::default();
        let session_id = RuntimeSessionId::from(String::from("pty-session-1"));
        registry.register_terminal(&PaneId::from(String::from("pane-1")), session_id);

        // Create receiver backed by registry (simulates RuntimeApplicationService)
        let receiver = RegistryBackedReceiver::new(registry);

        // Simulate PTY exit: infrastructure calls on_terminal_exited
        let pane_id = pane("pane-1");
        receiver.on_terminal_exited(&pane_id, "pty-session-1", Some(0));

        // Verify: registry was updated with Exited status
        let snapshot = receiver.registry.lock().expect("lock").snapshot();
        assert_eq!(snapshot.len(), 1);
        assert!(
            matches!(snapshot[0].status, RuntimeStatus::Exited),
            "registry should reflect Exited status after normal exit"
        );

        // Verify: projection was emitted
        let projections = receiver.projections_emitted.lock().expect("lock");
        assert_eq!(projections.len(), 1);
        assert_eq!(projections[0].0, "pane-1");
        assert!(matches!(projections[0].1, RuntimeStatus::Exited));
    }

    #[test]
    fn pty_exit_with_nonzero_code_marks_failed_and_emits_projection() {
        let mut registry = RuntimeRegistry::default();
        let session_id = RuntimeSessionId::from(String::from("pty-session-2"));
        registry.register_terminal(&PaneId::from(String::from("pane-f")), session_id);

        let receiver = RegistryBackedReceiver::new(registry);

        // Simulate PTY exit with failure code
        let pane_id = pane("pane-f");
        receiver.on_terminal_exited(&pane_id, "pty-session-2", Some(1));

        // Verify: registry records Failed status
        let snapshot = receiver.registry.lock().expect("lock").snapshot();
        assert_eq!(snapshot.len(), 1);
        assert!(
            matches!(snapshot[0].status, RuntimeStatus::Failed),
            "registry should reflect Failed status after non-zero exit"
        );
        assert_eq!(
            snapshot[0].last_error,
            Some(String::from("Process exited with code 1")),
        );

        // Verify: projection was emitted with Failed status
        let projections = receiver.projections_emitted.lock().expect("lock");
        assert_eq!(projections.len(), 1);
        assert!(matches!(projections[0].1, RuntimeStatus::Failed));
    }

    #[test]
    fn pty_exit_with_unknown_code_marks_exited() {
        let mut registry = RuntimeRegistry::default();
        let session_id = RuntimeSessionId::from(String::from("pty-session-3"));
        registry.register_terminal(&PaneId::from(String::from("pane-u")), session_id);

        let receiver = RegistryBackedReceiver::new(registry);

        // Simulate PTY exit with unknown exit code (None)
        let pane_id = pane("pane-u");
        receiver.on_terminal_exited(&pane_id, "pty-session-3", None);

        let snapshot = receiver.registry.lock().expect("lock").snapshot();
        assert_eq!(snapshot.len(), 1);
        assert!(
            matches!(snapshot[0].status, RuntimeStatus::Exited),
            "unknown exit code should default to Exited (not Failed)"
        );
    }
}
