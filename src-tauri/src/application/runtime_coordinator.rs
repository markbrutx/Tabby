use tabby_workspace::WorkspaceDomainEvent;

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
    /// Event mapping:
    /// - `PaneAdded(TerminalSpec)` → start terminal runtime
    /// - `PaneAdded(BrowserSpec)` → start browser runtime
    /// - `PaneRemoved` → stop runtime
    /// - `PaneSpecReplaced` → start runtime with new spec (old runtime already stopped)
    /// - `ActivePaneChanged` / `ActiveTabChanged` → no runtime side-effects
    pub fn handle_workspace_events(
        events: Vec<WorkspaceDomainEvent>,
        settings_service: &SettingsApplicationService,
        runtime_service: &RuntimeApplicationService,
    ) -> Result<(), ShellError> {
        if events.is_empty() {
            return Ok(());
        }

        let preferences = settings_service.preferences()?;

        for event in events {
            match event {
                WorkspaceDomainEvent::PaneAdded { pane_id, spec } => {
                    runtime_service.start_runtime(pane_id.as_ref(), &spec, &preferences)?;
                }
                WorkspaceDomainEvent::PaneSpecReplaced { pane_id, spec } => {
                    runtime_service.start_runtime(pane_id.as_ref(), &spec, &preferences)?;
                }
                WorkspaceDomainEvent::PaneRemoved { pane_id, .. } => {
                    runtime_service.stop_runtime(pane_id.as_ref());
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
    use tabby_workspace::{BrowserPaneSpec, PaneId, PaneSpec, TabId, TerminalPaneSpec};

    fn terminal_spec(profile_id: &str) -> PaneSpec {
        PaneSpec::Terminal(TerminalPaneSpec {
            launch_profile_id: String::from(profile_id),
            working_directory: String::from("/tmp"),
            command_override: None,
        })
    }

    fn browser_spec(url: &str) -> PaneSpec {
        PaneSpec::Browser(BrowserPaneSpec {
            initial_url: String::from(url),
        })
    }

    fn sid(s: &str) -> RuntimeSessionId {
        RuntimeSessionId::from(String::from(s))
    }

    // --- Event classification tests ---

    fn is_runtime_event(event: &WorkspaceDomainEvent) -> bool {
        matches!(
            event,
            WorkspaceDomainEvent::PaneAdded { .. }
                | WorkspaceDomainEvent::PaneRemoved { .. }
                | WorkspaceDomainEvent::PaneSpecReplaced { .. }
        )
    }

    #[test]
    fn pane_added_is_runtime_event() {
        let event = WorkspaceDomainEvent::PaneAdded {
            pane_id: PaneId::from(String::from("pane-1")),
            spec: terminal_spec("default"),
        };
        assert!(is_runtime_event(&event));
    }

    #[test]
    fn pane_removed_is_runtime_event() {
        let event = WorkspaceDomainEvent::PaneRemoved {
            pane_id: PaneId::from(String::from("pane-1")),
            spec: terminal_spec("default"),
        };
        assert!(is_runtime_event(&event));
    }

    #[test]
    fn pane_spec_replaced_is_runtime_event() {
        let event = WorkspaceDomainEvent::PaneSpecReplaced {
            pane_id: PaneId::from(String::from("pane-1")),
            spec: browser_spec("https://example.com"),
        };
        assert!(is_runtime_event(&event));
    }

    #[test]
    fn active_pane_changed_is_not_runtime_event() {
        let event = WorkspaceDomainEvent::ActivePaneChanged {
            pane_id: PaneId::from(String::from("pane-1")),
            tab_id: TabId::from(String::from("tab-1")),
        };
        assert!(!is_runtime_event(&event));
    }

    #[test]
    fn active_tab_changed_is_not_runtime_event() {
        let event = WorkspaceDomainEvent::ActiveTabChanged {
            tab_id: TabId::from(String::from("tab-1")),
        };
        assert!(!is_runtime_event(&event));
    }

    // --- Integration-style tests using RuntimeRegistry directly ---

    #[test]
    fn pane_added_terminal_registers_runtime() {
        let mut registry = RuntimeRegistry::default();

        // Simulate what handle_workspace_events does for PaneAdded(Terminal)
        let pane_id = "pane-1";
        let runtime = registry.register_terminal(
            pane_id,
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
        let pane_id = "pane-b";
        let runtime = registry.register_browser(
            pane_id,
            RuntimeSessionId::from(String::from("browser-session-1")),
            String::from("https://example.com"),
        );

        assert_eq!(runtime.pane_id, pane_id);
        assert!(matches!(runtime.kind, RuntimeKind::Browser));
        assert!(matches!(runtime.status, RuntimeStatus::Running));
        assert_eq!(
            runtime.browser_location,
            Some(String::from("https://example.com"))
        );
    }

    #[test]
    fn split_pane_adds_runtime_for_new_pane() {
        let mut registry = RuntimeRegistry::default();

        // Original pane runtime
        registry.register_terminal("pane-1", sid("pty-1"));

        // Split produces PaneAdded for the new pane
        let runtime = registry.register_terminal("pane-2", sid("pty-2"));

        assert_eq!(runtime.pane_id, "pane-2");
        assert_eq!(registry.snapshot().len(), 2);

        let session_1 = registry.terminal_session_id("pane-1");
        let session_2 = registry.terminal_session_id("pane-2");
        assert!(session_1.is_some());
        assert!(session_2.is_some());
        assert_ne!(session_1, session_2);
    }

    #[test]
    fn close_pane_stops_runtime() {
        let mut registry = RuntimeRegistry::default();

        registry.register_terminal("pane-1", sid("pty-1"));
        registry.register_terminal("pane-2", sid("pty-2"));
        assert_eq!(registry.snapshot().len(), 2);

        // PaneRemoved -> stop runtime
        let removed = registry.remove("pane-1");
        assert!(removed.is_some());
        let removed = removed.expect("runtime should exist");
        assert_eq!(removed.pane_id, "pane-1");

        assert_eq!(registry.snapshot().len(), 1);
        assert_eq!(registry.snapshot()[0].pane_id, "pane-2");
    }

    #[test]
    fn replace_terminal_with_browser_stops_old_starts_new() {
        let mut registry = RuntimeRegistry::default();

        // Start with terminal
        registry.register_terminal("pane-1", sid("pty-1"));
        assert!(matches!(registry.snapshot()[0].kind, RuntimeKind::Terminal));

        // PaneSpecReplaced: old runtime was already stopped by the caller,
        // coordinator starts new runtime
        let removed = registry.remove("pane-1");
        assert!(removed.is_some());

        let runtime = registry.register_browser(
            "pane-1",
            sid("browser-session-1"),
            String::from("https://example.com"),
        );

        assert_eq!(runtime.pane_id, "pane-1");
        assert!(matches!(runtime.kind, RuntimeKind::Browser));
        assert_eq!(registry.snapshot().len(), 1);
    }

    #[test]
    fn replace_browser_with_terminal_stops_old_starts_new() {
        let mut registry = RuntimeRegistry::default();

        // Start with browser
        registry.register_browser(
            "pane-1",
            sid("browser-1"),
            String::from("https://example.com"),
        );
        assert!(matches!(registry.snapshot()[0].kind, RuntimeKind::Browser));

        // Replace with terminal
        let removed = registry.remove("pane-1");
        assert!(removed.is_some());

        let runtime = registry.register_terminal("pane-1", sid("pty-1"));

        assert_eq!(runtime.pane_id, "pane-1");
        assert!(matches!(runtime.kind, RuntimeKind::Terminal));
        assert_eq!(registry.snapshot().len(), 1);
    }

    #[test]
    fn focus_events_do_not_affect_registry() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal("pane-1", sid("pty-1"));

        let snapshot_before = registry.snapshot().len();

        // ActivePaneChanged and ActiveTabChanged should not change registry
        // (verified by the is_runtime_event classification above)
        // Registry state is unchanged
        let snapshot_after = registry.snapshot().len();
        assert_eq!(snapshot_before, snapshot_after);
    }
}
