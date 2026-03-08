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

    pub fn track_terminal_working_directory(
        &self,
        pane_id: &PaneId,
        working_directory: &str,
    ) -> Result<(), ShellError> {
        self.lock_workspace()?
            .track_terminal_working_directory(pane_id, working_directory)
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
}
