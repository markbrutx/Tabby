//! Regression tests for runtime lifecycle flows (US-008).
//!
//! These tests verify that all runtime lifecycle paths work correctly:
//! natural exit, explicit stop, replace, restart, close_tab, and tab switch.
//!
//! All tests are pure unit/integration — no Tauri runtime dependency.
//! They exercise `RuntimeRegistry`, `WorkspaceSession`, `RuntimeCoordinator`
//! event mappings, and `RuntimeObservationReceiver` to simulate the full
//! lifecycle without infrastructure (PTY, browser surface, Tauri events).

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use tabby_runtime::{RuntimeKind, RuntimeRegistry, RuntimeSessionId, RuntimeStatus};
    use tabby_workspace::{
        spec_from_content, BrowserPaneSpec, PaneId, PaneSpec, TabLayoutStrategy, TerminalPaneSpec,
        WorkspaceDomainEvent, WorkspaceSession,
    };

    use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    fn sid(s: &str) -> RuntimeSessionId {
        RuntimeSessionId::from(String::from(s))
    }

    fn terminal_spec(cwd: &str) -> PaneSpec {
        PaneSpec::Terminal(TerminalPaneSpec {
            launch_profile_id: String::from("default"),
            working_directory: String::from(cwd),
            command_override: None,
        })
    }

    fn browser_spec(url: &str) -> PaneSpec {
        PaneSpec::Browser(BrowserPaneSpec {
            initial_url: String::from(url),
        })
    }

    /// Simulates `RuntimeApplicationService` without Tauri. Backed by a real
    /// `RuntimeRegistry` and records projection emissions for assertions.
    struct TestRuntimeService {
        registry: Mutex<RuntimeRegistry>,
        projections: Mutex<Vec<(String, RuntimeStatus)>>,
        next_session_counter: Mutex<u32>,
    }

    impl TestRuntimeService {
        fn new() -> Self {
            Self {
                registry: Mutex::new(RuntimeRegistry::default()),
                projections: Mutex::new(Vec::new()),
                next_session_counter: Mutex::new(1),
            }
        }

        fn next_session_id(&self, prefix: &str) -> RuntimeSessionId {
            let mut counter = self.next_session_counter.lock().expect("counter lock");
            let id = format!("{prefix}-{counter}");
            *counter += 1;
            sid(&id)
        }

        /// Simulates `RuntimeApplicationService::start_runtime`
        fn start_runtime(&self, pane_id: &str, spec: &PaneSpec) {
            let mut reg = self.registry.lock().expect("registry lock");
            let runtime = match spec {
                PaneSpec::Terminal(_) => {
                    let session = self.next_session_id("pty");
                    reg.register_terminal(pane_id, session)
                }
                PaneSpec::Browser(browser) => {
                    let session = self.next_session_id("browser");
                    reg.register_browser(pane_id, session, browser.initial_url.clone())
                }
            };
            self.emit_projection(&runtime.pane_id, runtime.status);
        }

        /// Simulates `RuntimeApplicationService::stop_runtime`
        fn stop_runtime(&self, pane_id: &str) {
            let removed = self.registry.lock().expect("registry lock").remove(pane_id);
            if let Some(runtime) = removed {
                self.emit_projection(&runtime.pane_id, RuntimeStatus::Exited);
            }
        }

        /// Simulates `RuntimeApplicationService::restart_runtime`
        fn restart_runtime(&self, pane_id: &str, spec: &PaneSpec) {
            self.stop_runtime(pane_id);
            self.start_runtime(pane_id, spec);
        }

        fn emit_projection(&self, pane_id: &str, status: RuntimeStatus) {
            self.projections
                .lock()
                .expect("projections lock")
                .push((String::from(pane_id), status));
        }

        fn snapshot(&self) -> Vec<tabby_runtime::PaneRuntime> {
            self.registry.lock().expect("registry lock").snapshot()
        }

        fn projections(&self) -> Vec<(String, RuntimeStatus)> {
            self.projections.lock().expect("projections lock").clone()
        }

        fn registry_get_status(&self, pane_id: &str) -> Option<RuntimeStatus> {
            self.registry
                .lock()
                .expect("registry lock")
                .get(pane_id)
                .map(|r| r.status)
        }

        fn registry_get_kind(&self, pane_id: &str) -> Option<RuntimeKind> {
            self.registry
                .lock()
                .expect("registry lock")
                .get(pane_id)
                .map(|r| r.kind)
        }
    }

    impl RuntimeObservationReceiver for TestRuntimeService {
        fn on_terminal_output_received(&self, _pane_id: &PaneId, _data: &[u8]) {}

        fn on_terminal_exited(&self, pane_id: &PaneId, exit_code: Option<i32>) {
            let failed = exit_code.is_some_and(|code| code != 0);
            let message = exit_code
                .filter(|code| *code != 0)
                .map(|code| format!("Process exited with code {code}"));

            let result = self
                .registry
                .lock()
                .expect("registry lock")
                .mark_terminal_exit(pane_id.as_ref(), None, failed, message);

            if let Ok(runtime) = result {
                self.emit_projection(&runtime.pane_id, runtime.status);
            }
        }

        fn on_browser_location_changed(&self, _pane_id: &PaneId, _url: &str) {}
        fn on_terminal_cwd_changed(&self, _pane_id: &PaneId, _cwd: &str) {}
    }

    /// Processes workspace domain events through the coordinator pattern,
    /// delegating to `TestRuntimeService` just like the real `RuntimeCoordinator`.
    fn apply_events(service: &TestRuntimeService, events: Vec<WorkspaceDomainEvent>) {
        for event in events {
            match event {
                WorkspaceDomainEvent::PaneAdded { pane_id, content } => {
                    let spec = spec_from_content(&content);
                    service.start_runtime(pane_id.as_ref(), &spec);
                }
                WorkspaceDomainEvent::PaneContentChanged {
                    pane_id,
                    new_content,
                    ..
                } => {
                    service.stop_runtime(pane_id.as_ref());
                    let spec = spec_from_content(&new_content);
                    service.start_runtime(pane_id.as_ref(), &spec);
                }
                WorkspaceDomainEvent::PaneRemoved { pane_id, .. } => {
                    service.stop_runtime(pane_id.as_ref());
                }
                WorkspaceDomainEvent::ActivePaneChanged { .. }
                | WorkspaceDomainEvent::ActiveTabChanged { .. } => {
                    // Focus events have NO runtime side-effects
                }
            }
        }
    }

    // =======================================================================
    // AC#1: Natural terminal exit → registry updated → projection emitted
    //       with Exited status
    // =======================================================================

    #[test]
    fn natural_terminal_exit_updates_registry_and_emits_exited_projection() {
        let service = TestRuntimeService::new();
        service.start_runtime("pane-1", &terminal_spec("/tmp"));

        // Verify Running state before exit
        assert_eq!(
            service.registry_get_status("pane-1"),
            Some(RuntimeStatus::Running)
        );

        // Simulate natural PTY exit (exit code 0) via observation receiver
        let pane_id = PaneId::from(String::from("pane-1"));
        service.on_terminal_exited(&pane_id, Some(0));

        // Registry must reflect Exited status
        assert_eq!(
            service.registry_get_status("pane-1"),
            Some(RuntimeStatus::Exited),
            "registry should be updated to Exited after natural exit"
        );

        // Projection must have been emitted with Exited status
        let projections = service.projections();
        let exit_projection = projections
            .iter()
            .filter(|(id, status)| id == "pane-1" && *status == RuntimeStatus::Exited)
            .count();
        assert_eq!(
            exit_projection, 1,
            "exactly one Exited projection should be emitted for natural exit"
        );
    }

    #[test]
    fn natural_terminal_exit_nonzero_marks_failed_and_emits_projection() {
        let service = TestRuntimeService::new();
        service.start_runtime("pane-fail", &terminal_spec("/tmp"));

        let pane_id = PaneId::from(String::from("pane-fail"));
        service.on_terminal_exited(&pane_id, Some(127));

        assert_eq!(
            service.registry_get_status("pane-fail"),
            Some(RuntimeStatus::Failed),
            "non-zero exit code should mark runtime as Failed"
        );

        let projections = service.projections();
        let fail_projection = projections
            .iter()
            .filter(|(id, status)| id == "pane-fail" && *status == RuntimeStatus::Failed)
            .count();
        assert_eq!(fail_projection, 1);
    }

    #[test]
    fn natural_terminal_exit_unknown_code_marks_exited() {
        let service = TestRuntimeService::new();
        service.start_runtime("pane-u", &terminal_spec("/tmp"));

        let pane_id = PaneId::from(String::from("pane-u"));
        service.on_terminal_exited(&pane_id, None);

        assert_eq!(
            service.registry_get_status("pane-u"),
            Some(RuntimeStatus::Exited),
            "unknown exit code (None) should default to Exited, not Failed"
        );
    }

    // =======================================================================
    // AC#2: Explicit stop_runtime → PTY killed → registry cleaned →
    //       projection emitted
    // =======================================================================

    #[test]
    fn explicit_stop_removes_from_registry_and_emits_exited_projection() {
        let service = TestRuntimeService::new();
        service.start_runtime("pane-1", &terminal_spec("/tmp"));

        assert_eq!(service.snapshot().len(), 1);

        service.stop_runtime("pane-1");

        // Registry must be cleaned (runtime removed)
        assert_eq!(
            service.snapshot().len(),
            0,
            "registry should be empty after stop"
        );

        // Projection with Exited status must have been emitted
        let projections = service.projections();
        let stop_projection = projections
            .iter()
            .filter(|(id, status)| id == "pane-1" && *status == RuntimeStatus::Exited)
            .count();
        assert_eq!(
            stop_projection, 1,
            "stop_runtime must emit an Exited projection"
        );
    }

    #[test]
    fn explicit_stop_browser_removes_from_registry_and_emits_projection() {
        let service = TestRuntimeService::new();
        service.start_runtime("pane-b", &browser_spec("https://example.com"));

        service.stop_runtime("pane-b");

        assert_eq!(service.snapshot().len(), 0);

        let projections = service.projections();
        assert!(
            projections
                .iter()
                .any(|(id, status)| id == "pane-b" && *status == RuntimeStatus::Exited),
            "browser stop must emit Exited projection"
        );
    }

    #[test]
    fn explicit_stop_nonexistent_pane_is_noop() {
        let service = TestRuntimeService::new();

        // Stopping a nonexistent pane should not panic or emit projections
        service.stop_runtime("ghost-pane");

        assert!(service.snapshot().is_empty());
        // Only no projections (no Running emission either since we never started)
        assert!(service.projections().is_empty());
    }

    // =======================================================================
    // AC#3: replace_pane_spec (terminal→browser) → old terminal stopped →
    //       new browser started
    // =======================================================================

    #[test]
    fn replace_terminal_with_browser_stops_old_and_starts_new() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // Open tab with a terminal pane
        let events = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                vec![terminal_spec("/tmp")],
            )
            .expect("open tab");
        apply_events(&service, events);

        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();

        // Verify terminal is running
        assert_eq!(
            service.registry_get_kind(pane_id.as_ref()),
            Some(RuntimeKind::Terminal)
        );
        assert_eq!(
            service.registry_get_status(pane_id.as_ref()),
            Some(RuntimeStatus::Running)
        );

        // Replace terminal with browser
        let replace_events = workspace
            .replace_pane_spec(&pane_id, browser_spec("https://example.com"))
            .expect("replace spec");
        apply_events(&service, replace_events);

        // Old terminal must be stopped, new browser must be running
        assert_eq!(
            service.registry_get_kind(pane_id.as_ref()),
            Some(RuntimeKind::Browser),
            "pane should now be a browser runtime"
        );
        assert_eq!(
            service.registry_get_status(pane_id.as_ref()),
            Some(RuntimeStatus::Running)
        );
        assert_eq!(service.snapshot().len(), 1);

        // Projections: Running(terminal) → Exited(stop) → Running(browser)
        let projections = service.projections();
        let pane_projections: Vec<_> = projections
            .iter()
            .filter(|(id, _)| id == pane_id.as_ref())
            .collect();
        // At minimum: initial Running, Exited from stop, Running from new start
        assert!(
            pane_projections.len() >= 3,
            "replace should emit at least 3 projections (start, stop, start)"
        );
        assert_eq!(
            pane_projections.last().expect("has projections").1,
            RuntimeStatus::Running,
            "last projection should be Running for the new browser"
        );
    }

    // =======================================================================
    // AC#4: replace_pane_spec (browser→terminal) → old browser stopped →
    //       new terminal started
    // =======================================================================

    #[test]
    fn replace_browser_with_terminal_stops_old_and_starts_new() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // Open tab with a browser pane
        let events = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                vec![browser_spec("https://example.com")],
            )
            .expect("open tab");
        apply_events(&service, events);

        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();
        assert_eq!(
            service.registry_get_kind(pane_id.as_ref()),
            Some(RuntimeKind::Browser)
        );

        // Replace browser with terminal
        let replace_events = workspace
            .replace_pane_spec(&pane_id, terminal_spec("/home"))
            .expect("replace spec");
        apply_events(&service, replace_events);

        assert_eq!(
            service.registry_get_kind(pane_id.as_ref()),
            Some(RuntimeKind::Terminal),
            "pane should now be a terminal runtime"
        );
        assert_eq!(
            service.registry_get_status(pane_id.as_ref()),
            Some(RuntimeStatus::Running)
        );
        assert_eq!(service.snapshot().len(), 1);
    }

    // =======================================================================
    // AC#5: restart_runtime → stop + start with same spec → registry
    //       updated correctly
    // =======================================================================

    #[test]
    fn restart_runtime_stops_then_starts_with_same_spec() {
        let service = TestRuntimeService::new();
        let spec = terminal_spec("/projects");

        service.start_runtime("pane-1", &spec);
        assert_eq!(service.snapshot().len(), 1);

        // Restart
        service.restart_runtime("pane-1", &spec);

        // Registry should have exactly one runtime, still Running
        assert_eq!(service.snapshot().len(), 1);
        assert_eq!(
            service.registry_get_status("pane-1"),
            Some(RuntimeStatus::Running)
        );
        assert_eq!(
            service.registry_get_kind("pane-1"),
            Some(RuntimeKind::Terminal)
        );

        // Projections: Running(start) → Exited(stop) → Running(restart start)
        let projections = service.projections();
        let pane_projections: Vec<_> = projections
            .iter()
            .filter(|(id, _)| id == "pane-1")
            .map(|(_, status)| *status)
            .collect();
        assert_eq!(
            pane_projections,
            vec![
                RuntimeStatus::Running,
                RuntimeStatus::Exited,
                RuntimeStatus::Running,
            ],
            "restart should produce Running → Exited → Running projections"
        );
    }

    #[test]
    fn restart_browser_runtime_stops_then_starts() {
        let service = TestRuntimeService::new();
        let spec = browser_spec("https://example.com");

        service.start_runtime("pane-b", &spec);
        service.restart_runtime("pane-b", &spec);

        assert_eq!(service.snapshot().len(), 1);
        assert_eq!(
            service.registry_get_kind("pane-b"),
            Some(RuntimeKind::Browser)
        );
        assert_eq!(
            service.registry_get_status("pane-b"),
            Some(RuntimeStatus::Running)
        );
    }

    // =======================================================================
    // AC#6: close_tab with multiple panes → all pane runtimes stopped
    // =======================================================================

    #[test]
    fn close_tab_stops_all_pane_runtimes() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // Open tab with 2 panes
        let events = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByTwo),
                vec![terminal_spec("/a"), terminal_spec("/b")],
            )
            .expect("open tab");
        apply_events(&service, events);

        let tab_id = workspace.tabs[0].tab_id.clone();
        let pane_1 = workspace.tabs[0].panes[0].pane_id.clone();
        let pane_2 = workspace.tabs[0].panes[1].pane_id.clone();

        // Both panes should have running runtimes
        assert_eq!(service.snapshot().len(), 2);
        assert_eq!(
            service.registry_get_status(pane_1.as_ref()),
            Some(RuntimeStatus::Running)
        );
        assert_eq!(
            service.registry_get_status(pane_2.as_ref()),
            Some(RuntimeStatus::Running)
        );

        // Close the entire tab
        let close_events = workspace.close_tab(&tab_id).expect("close tab");
        apply_events(&service, close_events);

        // All runtimes must be stopped
        assert_eq!(
            service.snapshot().len(),
            0,
            "all runtimes should be removed after close_tab"
        );

        // Verify Exited projections were emitted for both panes
        let projections = service.projections();
        let pane_1_exited = projections
            .iter()
            .any(|(id, status)| id == pane_1.as_ref() && *status == RuntimeStatus::Exited);
        let pane_2_exited = projections
            .iter()
            .any(|(id, status)| id == pane_2.as_ref() && *status == RuntimeStatus::Exited);

        assert!(
            pane_1_exited,
            "pane-1 must have Exited projection after close_tab"
        );
        assert!(
            pane_2_exited,
            "pane-2 must have Exited projection after close_tab"
        );
    }

    #[test]
    fn close_tab_with_mixed_pane_types_stops_all() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // Open tab with terminal + browser
        let events = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByTwo),
                vec![terminal_spec("/tmp"), browser_spec("https://example.com")],
            )
            .expect("open tab");
        apply_events(&service, events);

        assert_eq!(service.snapshot().len(), 2);

        let tab_id = workspace.tabs[0].tab_id.clone();
        let close_events = workspace.close_tab(&tab_id).expect("close tab");
        apply_events(&service, close_events);

        assert_eq!(
            service.snapshot().len(),
            0,
            "both terminal and browser runtimes should be stopped"
        );
    }

    #[test]
    fn close_tab_does_not_affect_other_tabs() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // Open two tabs
        let events_1 = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                vec![terminal_spec("/a")],
            )
            .expect("tab 1");
        apply_events(&service, events_1);

        let events_2 = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                vec![terminal_spec("/b")],
            )
            .expect("tab 2");
        apply_events(&service, events_2);

        assert_eq!(service.snapshot().len(), 2);

        let tab_1_id = workspace.tabs[0].tab_id.clone();
        let tab_2_pane = workspace.tabs[1].panes[0].pane_id.clone();

        // Close first tab
        let close_events = workspace.close_tab(&tab_1_id).expect("close tab 1");
        apply_events(&service, close_events);

        // Only one runtime remains (from tab 2)
        assert_eq!(service.snapshot().len(), 1);
        assert_eq!(
            service.registry_get_status(tab_2_pane.as_ref()),
            Some(RuntimeStatus::Running),
            "tab 2 pane should still be Running after closing tab 1"
        );
    }

    // =======================================================================
    // AC#7: Tab switch does NOT stop/start runtimes (ActiveTabChanged has
    //       no runtime side-effect)
    // =======================================================================

    #[test]
    fn tab_switch_does_not_stop_or_start_runtimes() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // Open two tabs
        let events_1 = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                vec![terminal_spec("/a")],
            )
            .expect("tab 1");
        apply_events(&service, events_1);

        let events_2 = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                vec![terminal_spec("/b")],
            )
            .expect("tab 2");
        apply_events(&service, events_2);

        let tab_1_id = workspace.tabs[0].tab_id.clone();
        let tab_1_pane = workspace.tabs[0].panes[0].pane_id.clone();
        let tab_2_pane = workspace.tabs[1].panes[0].pane_id.clone();

        // Record projection count before tab switch
        let projections_before = service.projections().len();

        // Switch to tab 1
        let switch_events = workspace.set_active_tab(&tab_1_id).expect("switch tab");
        apply_events(&service, switch_events);

        // No new projections should have been emitted
        let projections_after = service.projections().len();
        assert_eq!(
            projections_before, projections_after,
            "tab switch must NOT emit any runtime projections"
        );

        // Both runtimes should still be Running
        assert_eq!(service.snapshot().len(), 2);
        assert_eq!(
            service.registry_get_status(tab_1_pane.as_ref()),
            Some(RuntimeStatus::Running),
            "tab 1 runtime must survive tab switch"
        );
        assert_eq!(
            service.registry_get_status(tab_2_pane.as_ref()),
            Some(RuntimeStatus::Running),
            "tab 2 runtime must survive tab switch"
        );
    }

    #[test]
    fn focus_pane_does_not_affect_runtimes() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // Open tab with 2 panes
        let events = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByTwo),
                vec![terminal_spec("/a"), terminal_spec("/b")],
            )
            .expect("open tab");
        apply_events(&service, events);

        let tab_id = workspace.tabs[0].tab_id.clone();
        let pane_2 = workspace.tabs[0].panes[1].pane_id.clone();

        let projections_before = service.projections().len();

        // Focus the second pane
        let focus_events = workspace.focus_pane(&tab_id, &pane_2).expect("focus pane");
        apply_events(&service, focus_events);

        assert_eq!(
            service.projections().len(),
            projections_before,
            "focus_pane must NOT emit runtime projections"
        );
        assert_eq!(service.snapshot().len(), 2);
    }

    #[test]
    fn rapid_tab_switches_do_not_affect_runtimes() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // Open 3 tabs
        for cwd in &["/a", "/b", "/c"] {
            let events = workspace
                .open_tab(
                    TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                    vec![terminal_spec(cwd)],
                )
                .expect("open tab");
            apply_events(&service, events);
        }

        assert_eq!(service.snapshot().len(), 3);
        let projections_before = service.projections().len();

        // Rapidly switch between all tabs
        for tab in &workspace.tabs.clone() {
            let events = workspace.set_active_tab(&tab.tab_id).expect("switch tab");
            apply_events(&service, events);
        }

        assert_eq!(
            service.projections().len(),
            projections_before,
            "rapid tab switches must not emit any runtime projections"
        );
        assert_eq!(
            service.snapshot().len(),
            3,
            "all 3 runtimes must survive rapid tab switching"
        );
    }

    // =======================================================================
    // End-to-end lifecycle scenario combining multiple flows
    // =======================================================================

    #[test]
    fn full_lifecycle_open_split_replace_restart_close() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // 1. Open tab with terminal
        let events = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                vec![terminal_spec("/tmp")],
            )
            .expect("open tab");
        apply_events(&service, events);

        let pane_1 = workspace.tabs[0].panes[0].pane_id.clone();
        assert_eq!(service.snapshot().len(), 1);

        // 2. Split pane → new pane gets a runtime
        let split_events = workspace
            .split_pane(
                &pane_1,
                tabby_workspace::layout::SplitDirection::Horizontal,
                browser_spec("https://example.com"),
            )
            .expect("split");
        apply_events(&service, split_events);

        let pane_2 = workspace.tabs[0].panes[1].pane_id.clone();
        assert_eq!(service.snapshot().len(), 2);
        assert_eq!(
            service.registry_get_kind(pane_2.as_ref()),
            Some(RuntimeKind::Browser)
        );

        // 3. Replace pane_1 terminal → browser
        let replace_events = workspace
            .replace_pane_spec(&pane_1, browser_spec("https://docs.rs"))
            .expect("replace");
        apply_events(&service, replace_events);

        assert_eq!(
            service.registry_get_kind(pane_1.as_ref()),
            Some(RuntimeKind::Browser)
        );
        assert_eq!(service.snapshot().len(), 2);

        // 4. Restart pane_2
        let pane_2_spec = workspace.pane_spec(&pane_2).expect("pane_2 spec");
        service.restart_runtime(pane_2.as_ref(), &pane_2_spec);
        assert_eq!(service.snapshot().len(), 2);
        assert_eq!(
            service.registry_get_status(pane_2.as_ref()),
            Some(RuntimeStatus::Running)
        );

        // 5. Close entire tab → all runtimes stopped
        let tab_id = workspace.tabs[0].tab_id.clone();
        let close_events = workspace.close_tab(&tab_id).expect("close tab");
        apply_events(&service, close_events);

        assert_eq!(
            service.snapshot().len(),
            0,
            "all runtimes should be cleaned up"
        );
    }

    #[test]
    fn natural_exit_after_tab_switch_still_updates_registry() {
        let service = TestRuntimeService::new();
        let mut workspace = WorkspaceSession::default();

        // Open two tabs
        let events_1 = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                vec![terminal_spec("/a")],
            )
            .expect("tab 1");
        apply_events(&service, events_1);

        let events_2 = workspace
            .open_tab(
                TabLayoutStrategy::Preset(tabby_workspace::layout::LayoutPreset::OneByOne),
                vec![terminal_spec("/b")],
            )
            .expect("tab 2");
        apply_events(&service, events_2);

        let tab_1_id = workspace.tabs[0].tab_id.clone();
        let tab_1_pane = workspace.tabs[0].panes[0].pane_id.clone();

        // Switch away from tab 1
        // (tab 2 is already active since it was opened last)

        // Switch back to tab 1 then away again
        let switch_events = workspace.set_active_tab(&tab_1_id).expect("switch");
        apply_events(&service, switch_events);

        // Natural exit on tab 1's terminal (even though we may be on a different tab)
        service.on_terminal_exited(&tab_1_pane, Some(0));

        // Registry must still be updated even though we switched tabs
        assert_eq!(
            service.registry_get_status(tab_1_pane.as_ref()),
            Some(RuntimeStatus::Exited),
            "natural exit must update registry regardless of which tab is active"
        );
    }
}
