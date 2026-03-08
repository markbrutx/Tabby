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
/// Infrastructure will be wired to call this trait in a future story.
/// The trait and its implementation on `RuntimeApplicationService` are
/// introduced here; actual infra callers follow in subsequent stories.
#[allow(dead_code)]
pub trait RuntimeObservationReceiver: Send + Sync {
    /// Called by the PTY read thread when terminal output data is available.
    fn on_terminal_output_received(&self, pane_id: &PaneId, data: &[u8]);

    /// Called by the PTY read thread when the terminal process has exited.
    /// `exit_code` is `None` when the exit status could not be determined.
    fn on_terminal_exited(&self, pane_id: &PaneId, exit_code: Option<i32>);

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

        fn on_terminal_exited(&self, pane_id: &PaneId, exit_code: Option<i32>) {
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

        receiver.on_terminal_exited(&pane_id, Some(0));

        let log = receiver.into_log();
        assert_eq!(log.terminal_exits.len(), 1);
        assert_eq!(log.terminal_exits[0].0, "pane-2");
        assert_eq!(log.terminal_exits[0].1, Some(0));
    }

    #[test]
    fn mock_infra_can_call_on_terminal_exited_without_code() {
        let receiver = MockObservationReceiver::new();
        let pane_id = pane("pane-3");

        receiver.on_terminal_exited(&pane_id, None);

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
        receiver.on_terminal_exited(&pane_id, Some(1));
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
}
