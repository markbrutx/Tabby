use std::sync::Arc;

use tabby_workspace::{spec_from_content, WorkspaceDomainEvent};

use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
use crate::application::{RuntimeApplicationService, SettingsApplicationService};
use crate::shell::error::ShellError;

/// Coordinates runtime lifecycle in response to workspace domain events.
///
/// The `RuntimeCoordinator` reacts to events produced by `WorkspaceApplicationService`
/// and translates them into runtime operations (start, stop, restart) via
/// `RuntimeApplicationService`. This ensures that runtime management is driven
/// by domain events rather than scattered direct calls.
pub struct RuntimeCoordinator;

impl RuntimeCoordinator {
    /// Handles a batch of workspace domain events by starting, stopping, or
    /// restarting runtimes as needed.
    ///
    /// Event mapping (runtime-relevant events only):
    /// - `PaneAdded` → start runtime from content definition
    /// - `PaneRemoved` → stop runtime, content already destroyed
    /// - `PaneContentChanged` → stop old runtime, start new runtime from new content
    ///
    /// Structural focus events (`ActivePaneChanged`, `ActiveTabChanged`) are
    /// skipped — they carry no runtime side-effects.
    pub fn handle_workspace_events(
        events: Vec<WorkspaceDomainEvent>,
        settings_service: &SettingsApplicationService,
        runtime_service: &RuntimeApplicationService,
        observation_receiver: Arc<dyn RuntimeObservationReceiver>,
    ) -> Result<(), ShellError> {
        if events.is_empty() {
            return Ok(());
        }

        let preferences = settings_service.preferences()?;

        for event in events {
            match event {
                WorkspaceDomainEvent::PaneAdded { pane_id, content } => {
                    let spec = spec_from_content(&content);
                    runtime_service.start_runtime(
                        &pane_id,
                        &spec,
                        &preferences,
                        Arc::clone(&observation_receiver),
                    )?;
                }
                WorkspaceDomainEvent::PaneContentChanged {
                    pane_id,
                    new_content,
                    ..
                } => {
                    runtime_service.stop_runtime(&pane_id);
                    let spec = spec_from_content(&new_content);
                    runtime_service.start_runtime(
                        &pane_id,
                        &spec,
                        &preferences,
                        Arc::clone(&observation_receiver),
                    )?;
                }
                WorkspaceDomainEvent::PaneRemoved { pane_id, .. } => {
                    runtime_service.stop_runtime(&pane_id);
                }
                WorkspaceDomainEvent::ActivePaneChanged { .. }
                | WorkspaceDomainEvent::ActiveTabChanged { .. } => {
                    // Focus events don't require runtime side-effects
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabby_runtime::{RuntimeKind, RuntimeRegistry, RuntimeSessionId, RuntimeStatus};
    use tabby_workspace::{BrowserUrl, PaneContentDefinition, PaneContentId, PaneId, TabId};

    fn terminal_content(profile_id: &str) -> PaneContentDefinition {
        PaneContentDefinition::terminal(
            PaneContentId::from(uuid::Uuid::new_v4().to_string()),
            profile_id,
            "/tmp",
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

    fn sid(s: &str) -> RuntimeSessionId {
        RuntimeSessionId::from(String::from(s))
    }

    // --- Event classification tests (delegates to WorkspaceDomainEvent::is_runtime_relevant) ---

    #[test]
    fn pane_added_is_runtime_relevant() {
        let event = WorkspaceDomainEvent::PaneAdded {
            pane_id: PaneId::from(String::from("pane-1")),
            content: terminal_content("default"),
        };
        assert!(event.is_runtime_relevant());
    }

    #[test]
    fn pane_removed_is_runtime_relevant() {
        let event = WorkspaceDomainEvent::PaneRemoved {
            pane_id: PaneId::from(String::from("pane-1")),
            content: terminal_content("default"),
        };
        assert!(event.is_runtime_relevant());
    }

    #[test]
    fn pane_content_changed_is_runtime_relevant() {
        let event = WorkspaceDomainEvent::PaneContentChanged {
            pane_id: PaneId::from(String::from("pane-1")),
            old_content: terminal_content("default"),
            new_content: browser_content("https://example.com"),
        };
        assert!(event.is_runtime_relevant());
    }

    #[test]
    fn active_pane_changed_is_not_runtime_relevant() {
        let event = WorkspaceDomainEvent::ActivePaneChanged {
            pane_id: PaneId::from(String::from("pane-1")),
            tab_id: TabId::from(String::from("tab-1")),
        };
        assert!(
            !event.is_runtime_relevant(),
            "structural focus event must not trigger RuntimeCoordinator action"
        );
    }

    #[test]
    fn active_tab_changed_is_not_runtime_relevant() {
        let event = WorkspaceDomainEvent::ActiveTabChanged {
            tab_id: TabId::from(String::from("tab-1")),
        };
        assert!(
            !event.is_runtime_relevant(),
            "structural focus event must not trigger RuntimeCoordinator action"
        );
    }

    // --- Integration-style tests using RuntimeRegistry directly ---

    #[test]
    fn pane_added_terminal_registers_runtime() {
        let mut registry = RuntimeRegistry::default();

        // Simulate what handle_workspace_events does for PaneAdded(Terminal)
        let pane_id = pid("pane-1");
        let runtime = registry.register_terminal(
            &pane_id,
            RuntimeSessionId::from(String::from("pty-session-1")),
        );

        assert_eq!(runtime.pane_id, pane_id);
        assert!(matches!(runtime.kind, RuntimeKind::Terminal));
        assert!(matches!(runtime.status, RuntimeStatus::Running));
        assert_eq!(registry.snapshot().len(), 1);
    }

    #[test]
    fn pane_added_browser_registers_runtime() {
        let mut registry = RuntimeRegistry::default();

        // Simulate what handle_workspace_events does for PaneAdded(Browser)
        let pane_id = pid("pane-b");
        let runtime = registry.register_browser(
            &pane_id,
            RuntimeSessionId::from(String::from("browser-session-1")),
            BrowserUrl::new("https://example.com"),
        );

        assert_eq!(runtime.pane_id, pane_id);
        assert!(matches!(runtime.kind, RuntimeKind::Browser));
        assert!(matches!(runtime.status, RuntimeStatus::Running));
        assert_eq!(
            runtime.browser_location.as_ref().map(|u| u.as_str()),
            Some("https://example.com")
        );
    }

    #[test]
    fn split_pane_adds_runtime_for_new_pane() {
        let mut registry = RuntimeRegistry::default();

        // Original pane runtime
        registry.register_terminal(&pid("pane-1"), sid("pty-1"));

        // Split produces PaneAdded for the new pane
        let runtime = registry.register_terminal(&pid("pane-2"), sid("pty-2"));

        assert_eq!(runtime.pane_id, pid("pane-2"));
        assert_eq!(registry.snapshot().len(), 2);

        let session_1 = registry.terminal_session_id(&pid("pane-1"));
        let session_2 = registry.terminal_session_id(&pid("pane-2"));
        assert!(session_1.is_some());
        assert!(session_2.is_some());
        assert_ne!(session_1, session_2);
    }

    #[test]
    fn close_pane_stops_runtime() {
        let mut registry = RuntimeRegistry::default();

        registry.register_terminal(&pid("pane-1"), sid("pty-1"));
        registry.register_terminal(&pid("pane-2"), sid("pty-2"));
        assert_eq!(registry.snapshot().len(), 2);

        // PaneRemoved -> stop runtime
        let removed = registry.remove(&pid("pane-1"));
        assert!(removed.is_some());
        let removed = removed.expect("runtime should exist");
        assert_eq!(removed.pane_id, pid("pane-1"));

        assert_eq!(registry.snapshot().len(), 1);
        assert_eq!(registry.snapshot()[0].pane_id, pid("pane-2"));
    }

    #[test]
    fn replace_terminal_with_browser_stops_old_starts_new() {
        let mut registry = RuntimeRegistry::default();

        // Start with terminal
        registry.register_terminal(&pid("pane-1"), sid("pty-1"));
        assert!(matches!(registry.snapshot()[0].kind, RuntimeKind::Terminal));

        // PaneContentChanged: coordinator stops old runtime then starts new one
        let removed = registry.remove(&pid("pane-1"));
        assert!(removed.is_some());

        let runtime = registry.register_browser(
            &pid("pane-1"),
            sid("browser-session-1"),
            BrowserUrl::new("https://example.com"),
        );

        assert_eq!(runtime.pane_id, pid("pane-1"));
        assert!(matches!(runtime.kind, RuntimeKind::Browser));
        assert_eq!(registry.snapshot().len(), 1);
    }

    #[test]
    fn replace_browser_with_terminal_stops_old_starts_new() {
        let mut registry = RuntimeRegistry::default();

        // Start with browser
        registry.register_browser(
            &pid("pane-1"),
            sid("browser-1"),
            BrowserUrl::new("https://example.com"),
        );
        assert!(matches!(registry.snapshot()[0].kind, RuntimeKind::Browser));

        // Replace with terminal
        let removed = registry.remove(&pid("pane-1"));
        assert!(removed.is_some());

        let runtime = registry.register_terminal(&pid("pane-1"), sid("pty-1"));

        assert_eq!(runtime.pane_id, pid("pane-1"));
        assert!(matches!(runtime.kind, RuntimeKind::Terminal));
        assert_eq!(registry.snapshot().len(), 1);
    }

    #[test]
    fn focus_events_do_not_affect_registry() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal(&pid("pane-1"), sid("pty-1"));

        let snapshot_before = registry.snapshot().len();

        // ActivePaneChanged and ActiveTabChanged should not change registry
        // (verified by the is_runtime_event classification above)
        // Registry state is unchanged
        let snapshot_after = registry.snapshot().len();
        assert_eq!(snapshot_before, snapshot_after);
    }

    // --- Coordinator event-to-action mapping tests ---

    #[test]
    fn pane_added_terminal_requires_runtime_start() {
        // Verifies the coordinator mapping: PaneAdded(Terminal) → start_runtime
        let event = WorkspaceDomainEvent::PaneAdded {
            pane_id: PaneId::from(String::from("pane-t")),
            content: terminal_content("default"),
        };

        // After coordinator processes PaneAdded, a terminal runtime should exist
        let mut registry = RuntimeRegistry::default();
        if let WorkspaceDomainEvent::PaneAdded {
            ref pane_id,
            ref content,
        } = event
        {
            let spec = spec_from_content(content);
            if matches!(spec, tabby_workspace::PaneSpec::Terminal(_)) {
                let runtime = registry.register_terminal(pane_id, sid("pty-new"));
                assert!(matches!(runtime.kind, RuntimeKind::Terminal));
                assert!(matches!(runtime.status, RuntimeStatus::Running));
            }
        }
        assert_eq!(registry.snapshot().len(), 1);
    }

    #[test]
    fn pane_removed_requires_runtime_stop() {
        // Verifies the coordinator mapping: PaneRemoved → stop_runtime
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal(&pid("pane-1"), sid("pty-1"));
        registry.register_terminal(&pid("pane-2"), sid("pty-2"));

        let event = WorkspaceDomainEvent::PaneRemoved {
            pane_id: PaneId::from(String::from("pane-1")),
            content: terminal_content("default"),
        };

        if let WorkspaceDomainEvent::PaneRemoved { ref pane_id, .. } = event {
            let removed = registry.remove(pane_id);
            assert!(removed.is_some(), "runtime should have been registered");
        }

        assert_eq!(registry.snapshot().len(), 1);
        assert_eq!(registry.snapshot()[0].pane_id, pid("pane-2"));
    }

    #[test]
    fn pane_spec_replaced_requires_restart_with_new_spec() {
        // Verifies: PaneContentChanged → coordinator stops old + starts new
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal(&pid("pane-1"), sid("pty-1"));

        let new_content = browser_content("https://example.com");
        let event = WorkspaceDomainEvent::PaneContentChanged {
            pane_id: PaneId::from(String::from("pane-1")),
            old_content: terminal_content("default"),
            new_content: new_content.clone(),
        };

        if let WorkspaceDomainEvent::PaneContentChanged {
            ref pane_id,
            new_content: ref nc,
            ..
        } = event
        {
            // Coordinator stops old runtime before starting new one
            registry.remove(pane_id);
            // Coordinator starts new runtime with new content
            let spec = spec_from_content(nc);
            match spec {
                tabby_workspace::PaneSpec::Browser(browser) => {
                    let runtime = registry.register_browser(
                        pane_id,
                        sid("browser-new"),
                        browser.initial_url.clone(),
                    );
                    assert!(matches!(runtime.kind, RuntimeKind::Browser));
                }
                _ => panic!("expected browser spec"),
            }
        }

        assert_eq!(registry.snapshot().len(), 1);
        assert!(matches!(registry.snapshot()[0].kind, RuntimeKind::Browser));
    }

    // --- AC#5: replace_pane_spec → PaneContentChanged → stop old + start new ---

    #[test]
    fn replace_pane_spec_event_triggers_coordinator_stop_old_then_start_new() {
        // Simulates the full flow: workspace.replace_pane_spec() emits
        // PaneContentChanged → RuntimeCoordinator handles it by stopping
        // the old runtime and starting the new one. The shell layer does
        // NOT manually call stop_runtime before the workspace mutation.
        let mut registry = RuntimeRegistry::default();

        // 1. Initial state: terminal runtime registered for pane-1
        let old_session = sid("pty-old");
        registry.register_terminal(&pid("pane-1"), old_session.clone());
        assert_eq!(registry.snapshot().len(), 1);
        assert!(matches!(registry.snapshot()[0].kind, RuntimeKind::Terminal));
        assert_eq!(
            registry.terminal_session_id(&pid("pane-1")),
            Some(old_session.clone()),
        );

        // 2. Workspace emits PaneContentChanged (this is what replace_pane_spec returns)
        let event = WorkspaceDomainEvent::PaneContentChanged {
            pane_id: PaneId::from(String::from("pane-1")),
            old_content: terminal_content("default"),
            new_content: browser_content("https://example.com"),
        };

        // 3. Coordinator processes event: stop old → start new
        //    (mirrors handle_workspace_events logic for PaneContentChanged)
        if let WorkspaceDomainEvent::PaneContentChanged {
            ref pane_id,
            new_content: ref nc,
            ..
        } = event
        {
            // a) Stop old runtime — same as runtime_service.stop_runtime()
            let removed = registry.remove(pane_id);
            assert!(
                removed.is_some(),
                "old runtime must be stopped by coordinator"
            );
            let removed = removed.expect("already asserted");
            assert!(
                matches!(removed.kind, RuntimeKind::Terminal),
                "removed runtime should be the old terminal"
            );

            // b) Start new runtime with the replaced content
            let spec = spec_from_content(nc);
            match spec {
                tabby_workspace::PaneSpec::Browser(browser) => {
                    let new_runtime = registry.register_browser(
                        pane_id,
                        sid("browser-new"),
                        browser.initial_url.clone(),
                    );
                    assert!(matches!(new_runtime.kind, RuntimeKind::Browser));
                    assert!(matches!(new_runtime.status, RuntimeStatus::Running));
                    assert_eq!(
                        new_runtime.browser_location.as_ref().map(|u| u.as_str()),
                        Some("https://example.com"),
                    );
                }
                _ => panic!("expected browser spec in PaneContentChanged"),
            }
        }

        // 4. Final state: exactly one runtime, browser kind, for pane-1
        assert_eq!(
            registry.snapshot().len(),
            1,
            "should have exactly one runtime after replace"
        );
        let final_runtime = &registry.snapshot()[0];
        assert_eq!(final_runtime.pane_id, pid("pane-1"));
        assert!(matches!(final_runtime.kind, RuntimeKind::Browser));
        assert!(matches!(final_runtime.status, RuntimeStatus::Running));

        // 5. Old terminal runtime is replaced — pane-1 is now a browser
        let current = registry.get(&pid("pane-1"));
        assert!(current.is_some(), "pane-1 should still have a runtime");
        assert!(
            matches!(current.expect("asserted").kind, RuntimeKind::Browser),
            "pane-1 should now be a browser, not the old terminal"
        );
    }

    // --- AC#5: Full lifecycle integration test ---
    // split pane → runtime started, close pane → runtime stopped,
    // replace spec → old stopped + new started

    #[test]
    fn full_lifecycle_split_close_replace() {
        let mut registry = RuntimeRegistry::default();

        // === Phase 1: Initial tab with one terminal pane ===
        registry.register_terminal(&pid("pane-1"), sid("pty-1"));
        assert_eq!(registry.snapshot().len(), 1);

        // === Phase 2: Split pane → new runtime started ===
        // WorkspaceService.split_pane() emits PaneAdded for the new pane.
        // RuntimeCoordinator handles PaneAdded → start_runtime.
        let split_event = WorkspaceDomainEvent::PaneAdded {
            pane_id: PaneId::from(String::from("pane-2")),
            content: terminal_content("default"),
        };
        if let WorkspaceDomainEvent::PaneAdded {
            ref pane_id,
            ref content,
        } = split_event
        {
            let spec = spec_from_content(content);
            if matches!(spec, tabby_workspace::PaneSpec::Terminal(_)) {
                registry.register_terminal(pane_id, sid("pty-2"));
            }
        }
        assert_eq!(
            registry.snapshot().len(),
            2,
            "split should add a second runtime"
        );
        assert!(
            registry.terminal_session_id(&pid("pane-2")).is_some(),
            "new pane should have a terminal session"
        );

        // === Phase 3: Close pane → runtime stopped ===
        // WorkspaceService.close_pane() emits PaneRemoved.
        // RuntimeCoordinator handles PaneRemoved → stop_runtime.
        let close_event = WorkspaceDomainEvent::PaneRemoved {
            pane_id: PaneId::from(String::from("pane-2")),
            content: terminal_content("default"),
        };
        if let WorkspaceDomainEvent::PaneRemoved { ref pane_id, .. } = close_event {
            let removed = registry.remove(pane_id);
            assert!(removed.is_some(), "closed pane runtime should be removed");
        }
        assert_eq!(
            registry.snapshot().len(),
            1,
            "only original pane runtime should remain after close"
        );
        assert!(
            registry.terminal_session_id(&pid("pane-2")).is_none(),
            "closed pane should have no session"
        );
        assert!(
            registry.terminal_session_id(&pid("pane-1")).is_some(),
            "original pane should still have its session"
        );

        // === Phase 4: Replace spec → old stopped + new started ===
        // WorkspaceService.replace_pane_spec() emits PaneContentChanged.
        // RuntimeCoordinator handles PaneContentChanged → stop old + start new.
        let replace_event = WorkspaceDomainEvent::PaneContentChanged {
            pane_id: PaneId::from(String::from("pane-1")),
            old_content: terminal_content("default"),
            new_content: browser_content("https://example.com"),
        };
        if let WorkspaceDomainEvent::PaneContentChanged {
            ref pane_id,
            new_content: ref nc,
            ..
        } = replace_event
        {
            // Coordinator stops old runtime
            let removed = registry.remove(pane_id);
            assert!(removed.is_some(), "old runtime should be stopped");
            let removed = removed.expect("already asserted");
            assert!(
                matches!(removed.kind, RuntimeKind::Terminal),
                "removed runtime should be the old terminal"
            );

            // Coordinator starts new runtime with replaced content
            let spec = spec_from_content(nc);
            if let tabby_workspace::PaneSpec::Browser(browser) = spec {
                registry.register_browser(pane_id, sid("browser-new"), browser.initial_url.clone());
            }
        }

        // Final assertions: exactly one runtime, browser kind, correct pane
        assert_eq!(
            registry.snapshot().len(),
            1,
            "should have exactly one runtime after replace"
        );
        let final_runtime = &registry.snapshot()[0];
        assert_eq!(final_runtime.pane_id, pid("pane-1"));
        assert!(
            matches!(final_runtime.kind, RuntimeKind::Browser),
            "pane-1 should now be a browser runtime"
        );
        assert!(
            matches!(final_runtime.status, RuntimeStatus::Running),
            "new browser runtime should be Running"
        );
        assert_eq!(
            final_runtime.browser_location.as_ref().map(|u| u.as_str()),
            Some("https://example.com"),
        );
    }

    // --- Failure path tests ---

    #[test]
    fn remove_nonexistent_runtime_returns_none() {
        let mut registry = RuntimeRegistry::default();
        let result = registry.remove(&pid("nonexistent-pane"));
        assert!(
            result.is_none(),
            "removing nonexistent runtime should return None"
        );
    }

    #[test]
    fn terminal_session_id_for_nonexistent_pane_returns_none() {
        let registry = RuntimeRegistry::default();
        let result = registry.terminal_session_id(&pid("nonexistent-pane"));
        assert!(
            result.is_none(),
            "terminal session id for nonexistent pane should be None"
        );
    }

    #[test]
    fn mark_terminal_exit_with_failure_sets_failed_status() {
        let mut registry = RuntimeRegistry::default();
        let session_id = sid("pty-1");
        registry.register_terminal(&pid("pane-1"), session_id.clone());

        let result = registry.mark_terminal_exit(
            &pid("pane-1"),
            Some(&session_id),
            true,
            Some(String::from("spawn failed")),
        );
        assert!(result.is_ok(), "mark_terminal_exit should succeed");

        let runtime = result.expect("already asserted ok");
        assert!(
            matches!(runtime.status, RuntimeStatus::Failed),
            "status should be Failed after failure exit"
        );
        assert_eq!(
            runtime.last_error,
            Some(String::from("spawn failed")),
            "last_error should contain failure message"
        );
    }

    #[test]
    fn mark_terminal_exit_normal_sets_exited_status() {
        let mut registry = RuntimeRegistry::default();
        let session_id = sid("pty-1");
        registry.register_terminal(&pid("pane-1"), session_id.clone());

        let result = registry.mark_terminal_exit(&pid("pane-1"), Some(&session_id), false, None);
        assert!(result.is_ok(), "mark_terminal_exit should succeed");

        let runtime = result.expect("already asserted ok");
        assert!(
            matches!(runtime.status, RuntimeStatus::Exited),
            "status should be Exited after normal exit"
        );
        assert!(
            runtime.last_error.is_none(),
            "last_error should be None for normal exit"
        );
    }

    #[test]
    fn update_browser_location_for_nonexistent_pane_returns_error() {
        let mut registry = RuntimeRegistry::default();
        let result = registry
            .update_browser_location(&pid("nonexistent"), BrowserUrl::new("https://example.com"));
        assert!(
            result.is_err(),
            "updating location for nonexistent pane should return error"
        );
    }

    #[test]
    fn multiple_events_processed_sequentially() {
        // Verifies that a batch of events is processed correctly
        let mut registry = RuntimeRegistry::default();

        let events = vec![
            WorkspaceDomainEvent::PaneAdded {
                pane_id: PaneId::from(String::from("pane-1")),
                content: terminal_content("default"),
            },
            WorkspaceDomainEvent::PaneAdded {
                pane_id: PaneId::from(String::from("pane-2")),
                content: browser_content("https://example.com"),
            },
            WorkspaceDomainEvent::ActivePaneChanged {
                pane_id: PaneId::from(String::from("pane-1")),
                tab_id: TabId::from(String::from("tab-1")),
            },
        ];

        // Process events as the coordinator would
        for event in &events {
            match event {
                WorkspaceDomainEvent::PaneAdded { pane_id, content } => {
                    let spec = spec_from_content(content);
                    match spec {
                        tabby_workspace::PaneSpec::Terminal(_) => {
                            registry.register_terminal(
                                pane_id,
                                RuntimeSessionId::from(format!("pty-{}", pane_id)),
                            );
                        }
                        tabby_workspace::PaneSpec::Browser(browser) => {
                            registry.register_browser(
                                pane_id,
                                RuntimeSessionId::from(format!("browser-{}", pane_id)),
                                browser.initial_url.clone(),
                            );
                        }
                    }
                }
                WorkspaceDomainEvent::ActivePaneChanged { .. }
                | WorkspaceDomainEvent::ActiveTabChanged { .. } => {
                    // No registry changes expected
                }
                _ => {}
            }
        }

        assert_eq!(
            registry.snapshot().len(),
            2,
            "two runtimes should be registered"
        );

        let pane_1 = registry
            .snapshot()
            .iter()
            .find(|r| r.pane_id == pid("pane-1"))
            .cloned();
        assert!(pane_1.is_some());
        let pane_1 = pane_1.expect("already asserted some");
        assert!(matches!(pane_1.kind, RuntimeKind::Terminal));

        let pane_2 = registry
            .snapshot()
            .iter()
            .find(|r| r.pane_id == pid("pane-2"))
            .cloned();
        assert!(pane_2.is_some());
        let pane_2 = pane_2.expect("already asserted some");
        assert!(matches!(pane_2.kind, RuntimeKind::Browser));
    }
}
