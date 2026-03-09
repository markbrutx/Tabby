use std::sync::Mutex;

use tabby_workspace::layout::{LayoutPreset, SplitDirection};
use tabby_workspace::{
    PaneId, PaneSpec, TabId, TabLayoutStrategy, WorkspaceDomainEvent, WorkspaceError,
    WorkspaceSession,
};

use crate::shell::error::ShellError;

#[derive(Debug)]
pub struct WorkspaceApplicationService {
    workspace: Mutex<WorkspaceSession>,
}

impl Default for WorkspaceApplicationService {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceApplicationService {
    pub fn new() -> Self {
        Self {
            workspace: Mutex::new(WorkspaceSession::default()),
        }
    }

    pub fn with_session<F, R>(&self, f: F) -> Result<R, ShellError>
    where
        F: FnOnce(&WorkspaceSession) -> R,
    {
        let workspace = self.lock_workspace()?;
        Ok(f(&workspace))
    }

    pub fn is_empty(&self) -> Result<bool, ShellError> {
        let workspace = self.lock_workspace()?;
        Ok(workspace.tab_summaries().is_empty())
    }

    pub fn pane_spec(&self, pane_id: &PaneId) -> Result<Option<PaneSpec>, ShellError> {
        let workspace = self.lock_workspace()?;
        Ok(workspace.pane_spec(pane_id))
    }

    pub fn open_tab(
        &self,
        layout: LayoutPreset,
        auto_layout: bool,
        pane_specs: Vec<PaneSpec>,
    ) -> Result<Vec<WorkspaceDomainEvent>, ShellError> {
        let strategy = if auto_layout {
            TabLayoutStrategy::AutoCount
        } else {
            TabLayoutStrategy::Preset(layout)
        };
        self.lock_workspace()?
            .open_tab(strategy, pane_specs)
            .map_err(workspace_error_to_shell)
    }

    pub fn close_tab(&self, tab_id: &TabId) -> Result<Vec<WorkspaceDomainEvent>, ShellError> {
        self.lock_workspace()?
            .close_tab(tab_id)
            .map_err(workspace_error_to_shell)
    }

    pub fn set_active_tab(&self, tab_id: &TabId) -> Result<Vec<WorkspaceDomainEvent>, ShellError> {
        self.lock_workspace()?
            .set_active_tab(tab_id)
            .map_err(workspace_error_to_shell)
    }

    pub fn focus_pane(
        &self,
        tab_id: &TabId,
        pane_id: &PaneId,
    ) -> Result<Vec<WorkspaceDomainEvent>, ShellError> {
        self.lock_workspace()?
            .focus_pane(tab_id, pane_id)
            .map_err(workspace_error_to_shell)
    }

    pub fn split_pane(
        &self,
        pane_id: &PaneId,
        direction: SplitDirection,
        spec: PaneSpec,
    ) -> Result<Vec<WorkspaceDomainEvent>, ShellError> {
        self.lock_workspace()?
            .split_pane(pane_id, direction, spec)
            .map_err(workspace_error_to_shell)
    }

    pub fn close_pane(&self, pane_id: &PaneId) -> Result<Vec<WorkspaceDomainEvent>, ShellError> {
        self.lock_workspace()?
            .close_pane(pane_id)
            .map_err(workspace_error_to_shell)
    }

    pub fn swap_pane_slots(
        &self,
        pane_id_a: &PaneId,
        pane_id_b: &PaneId,
    ) -> Result<(), ShellError> {
        self.lock_workspace()?
            .swap_pane_slots(pane_id_a, pane_id_b)
            .map_err(workspace_error_to_shell)
    }

    pub fn replace_pane_spec(
        &self,
        pane_id: &PaneId,
        spec: PaneSpec,
    ) -> Result<Vec<WorkspaceDomainEvent>, ShellError> {
        self.lock_workspace()?
            .replace_pane_spec(pane_id, spec)
            .map_err(workspace_error_to_shell)
    }

    fn lock_workspace(&self) -> Result<std::sync::MutexGuard<'_, WorkspaceSession>, ShellError> {
        self.workspace
            .lock()
            .map_err(|_| ShellError::State(String::from("Workspace lock poisoned")))
    }
}

fn workspace_error_to_shell(error: WorkspaceError) -> ShellError {
    match error {
        WorkspaceError::Validation(message) => ShellError::Validation(message),
        WorkspaceError::NotFound(message) => ShellError::NotFound(message),
        WorkspaceError::State(message) => ShellError::State(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabby_workspace::TerminalPaneSpec;

    fn terminal_spec(profile_id: &str) -> PaneSpec {
        PaneSpec::Terminal(TerminalPaneSpec {
            launch_profile_id: String::from(profile_id),
            working_directory: String::from("/tmp"),
            command_override: None,
        })
    }

    #[test]
    fn open_tab_creates_panes_and_emits_events() {
        let service = WorkspaceApplicationService::new();
        let specs = vec![terminal_spec("default")];

        let events = service
            .open_tab(LayoutPreset::OneByOne, false, specs)
            .expect("open_tab should succeed");

        assert!(
            !events.is_empty(),
            "should emit at least one PaneAdded event"
        );
        let tab_count = service
            .with_session(|session| session.tab_summaries().len())
            .expect("with_session should succeed");
        assert_eq!(tab_count, 1, "workspace should have one tab");
    }

    #[test]
    fn close_pane_removes_pane_and_emits_event() {
        let service = WorkspaceApplicationService::new();
        let specs = vec![terminal_spec("a"), terminal_spec("b")];
        service
            .open_tab(LayoutPreset::OneByTwo, false, specs)
            .expect("open_tab should succeed");

        let pane_id = service
            .with_session(|session| session.tab_summaries()[0].panes[0].pane_id.clone())
            .expect("with_session");

        let events = service
            .close_pane(&pane_id)
            .expect("close_pane should succeed");

        assert!(!events.is_empty(), "should emit PaneRemoved event");
        let pane_count = service
            .with_session(|session| session.tab_summaries()[0].panes.len())
            .expect("with_session");
        assert_eq!(pane_count, 1, "should have one pane remaining");
    }

    #[test]
    fn split_pane_adds_new_pane() {
        let service = WorkspaceApplicationService::new();
        let specs = vec![terminal_spec("default")];
        service
            .open_tab(LayoutPreset::OneByOne, false, specs)
            .expect("open_tab should succeed");

        let pane_id = service
            .with_session(|session| session.tab_summaries()[0].panes[0].pane_id.clone())
            .expect("with_session");

        let events = service
            .split_pane(&pane_id, SplitDirection::Horizontal, terminal_spec("split"))
            .expect("split_pane should succeed");

        assert!(!events.is_empty(), "should emit PaneAdded event");
        let pane_count = service
            .with_session(|session| session.tab_summaries()[0].panes.len())
            .expect("with_session");
        assert_eq!(pane_count, 2, "should have two panes after split");
    }

    #[test]
    fn close_pane_not_found_returns_error() {
        let service = WorkspaceApplicationService::new();
        let result = service.close_pane(&PaneId::from(String::from("nonexistent-pane")));
        assert!(result.is_err(), "should return error for nonexistent pane");
    }

    #[test]
    fn close_tab_removes_tab_and_emits_pane_removed_events() {
        let service = WorkspaceApplicationService::new();
        let specs = vec![terminal_spec("a"), terminal_spec("b")];
        service
            .open_tab(LayoutPreset::OneByTwo, false, specs)
            .expect("open_tab should succeed");

        let tab_id = service
            .with_session(|session| session.tab_summaries()[0].tab_id.clone())
            .expect("with_session");

        let events = service
            .close_tab(&tab_id)
            .expect("close_tab should succeed");

        let pane_removed_count = events
            .iter()
            .filter(|e| matches!(e, WorkspaceDomainEvent::PaneRemoved { .. }))
            .count();
        assert_eq!(
            pane_removed_count, 2,
            "should emit PaneRemoved for each pane in the tab"
        );

        let is_empty = service.is_empty().expect("is_empty should succeed");
        assert!(is_empty, "workspace should be empty after closing only tab");
    }

    #[test]
    fn close_tab_not_found_returns_error() {
        let service = WorkspaceApplicationService::new();
        let result = service.close_tab(&TabId::from(String::from("nonexistent-tab")));
        assert!(result.is_err(), "should return error for nonexistent tab");
    }

    #[test]
    fn replace_pane_spec_updates_spec_and_emits_event() {
        let service = WorkspaceApplicationService::new();
        let specs = vec![terminal_spec("default")];
        service
            .open_tab(LayoutPreset::OneByOne, false, specs)
            .expect("open_tab should succeed");

        let pane_id = service
            .with_session(|session| session.tab_summaries()[0].panes[0].pane_id.clone())
            .expect("with_session");

        let new_spec = PaneSpec::Browser(tabby_workspace::BrowserPaneSpec {
            initial_url: tabby_workspace::BrowserUrl::new("https://example.com"),
        });
        let events = service
            .replace_pane_spec(&pane_id, new_spec.clone())
            .expect("replace_pane_spec should succeed");

        assert_eq!(events.len(), 1);
        match &events[0] {
            WorkspaceDomainEvent::PaneContentChanged {
                pane_id: pid,
                old_content,
                new_content,
            } => {
                assert_eq!(*pid, pane_id);
                // Old content was terminal
                assert!(old_content.terminal_profile_id().is_some());
                // New content is browser
                assert!(new_content.browser_url().is_some());
                // Old content id is never reused
                assert_ne!(old_content.content_id(), new_content.content_id());
            }
            other => panic!("expected PaneContentChanged, got {other:?}"),
        }

        let updated_spec = service
            .pane_spec(&pane_id)
            .expect("pane_spec should succeed");
        assert_eq!(
            updated_spec,
            Some(new_spec),
            "pane spec should be updated in workspace"
        );
    }

    #[test]
    fn replace_pane_spec_on_nonexistent_pane_returns_error() {
        let service = WorkspaceApplicationService::new();
        let new_spec = PaneSpec::Browser(tabby_workspace::BrowserPaneSpec {
            initial_url: tabby_workspace::BrowserUrl::new("https://example.com"),
        });
        let result =
            service.replace_pane_spec(&PaneId::from(String::from("nonexistent")), new_spec);
        assert!(
            result.is_err(),
            "should return error for nonexistent pane replacement"
        );
    }

    #[test]
    fn split_nonexistent_pane_returns_error() {
        let service = WorkspaceApplicationService::new();
        let result = service.split_pane(
            &PaneId::from(String::from("nonexistent")),
            SplitDirection::Horizontal,
            terminal_spec("default"),
        );
        assert!(result.is_err(), "should return error for nonexistent pane");
    }

    #[test]
    fn open_tab_with_empty_specs_returns_validation_error() {
        let service = WorkspaceApplicationService::new();
        let result = service.open_tab(LayoutPreset::OneByOne, false, vec![]);
        assert!(
            result.is_err(),
            "should return error when no pane specs provided"
        );
    }

    #[test]
    fn close_last_pane_closes_tab() {
        let service = WorkspaceApplicationService::new();
        let specs = vec![terminal_spec("default")];
        service
            .open_tab(LayoutPreset::OneByOne, false, specs)
            .expect("open_tab should succeed");

        let pane_id = service
            .with_session(|session| session.tab_summaries()[0].panes[0].pane_id.clone())
            .expect("with_session");

        let events = service
            .close_pane(&pane_id)
            .expect("close_pane should succeed");

        assert!(
            events
                .iter()
                .any(|e| matches!(e, WorkspaceDomainEvent::PaneRemoved { .. })),
            "should emit PaneRemoved"
        );

        let is_empty = service.is_empty().expect("is_empty should succeed");
        assert!(
            is_empty,
            "closing last pane should also close the tab, leaving workspace empty"
        );
    }

    #[test]
    fn swap_nonexistent_panes_returns_error() {
        let service = WorkspaceApplicationService::new();
        let result = service.swap_pane_slots(
            &PaneId::from(String::from("fake-a")),
            &PaneId::from(String::from("fake-b")),
        );
        assert!(
            result.is_err(),
            "should return error when swapping nonexistent panes"
        );
    }

    #[test]
    fn set_active_tab_not_found_returns_error() {
        let service = WorkspaceApplicationService::new();
        let result = service.set_active_tab(&TabId::from(String::from("nonexistent")));
        assert!(result.is_err(), "should return error for nonexistent tab");
    }

    #[test]
    fn focus_pane_not_found_returns_error() {
        let service = WorkspaceApplicationService::new();
        let specs = vec![terminal_spec("default")];
        service
            .open_tab(LayoutPreset::OneByOne, false, specs)
            .expect("open_tab should succeed");

        let tab_id = service
            .with_session(|session| session.tab_summaries()[0].tab_id.clone())
            .expect("with_session");

        let result = service.focus_pane(&tab_id, &PaneId::from(String::from("nonexistent-pane")));
        assert!(result.is_err(), "should return error for nonexistent pane");
    }
}
