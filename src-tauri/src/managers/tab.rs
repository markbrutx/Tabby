use std::sync::Mutex;

use crate::domain::error::TabbyError;
use crate::domain::types::{
    create_tab_id, LayoutPreset, PaneSeed, PaneSnapshot, TabSnapshot, WorkspaceSnapshot,
};

#[derive(Debug, Default)]
struct WorkspaceState {
    tabs: Vec<TabSnapshot>,
    active_tab_id: Option<String>,
    next_tab_index: usize,
}

#[derive(Debug, Clone)]
pub struct LocatedPane {
    pub pane: PaneSnapshot,
}

#[derive(Debug, Default)]
pub struct TabManager {
    state: Mutex<WorkspaceState>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(WorkspaceState {
                tabs: Vec::new(),
                active_tab_id: None,
                next_tab_index: 1,
            }),
        }
    }

    pub fn is_empty(&self) -> Result<bool, TabbyError> {
        Ok(self.lock_state()?.tabs.is_empty())
    }

    pub fn snapshot(&self) -> Result<WorkspaceSnapshot, TabbyError> {
        let state = self.lock_state()?;
        Ok(Self::workspace_snapshot(&state))
    }

    pub fn create_tab(
        &self,
        preset: LayoutPreset,
        pane_seeds: Vec<PaneSeed>,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        if pane_seeds.is_empty() {
            return Err(TabbyError::Validation(String::from(
                "Cannot create a tab without panes",
            )));
        }

        let mut state = self.lock_state()?;
        let tab_id = create_tab_id();
        let title = format!("Workspace {}", state.next_tab_index);

        let panes = pane_seeds
            .into_iter()
            .enumerate()
            .map(|(index, seed)| PaneSnapshot {
                id: seed.pane_id,
                session_id: seed.session_id,
                title: format!("Pane {}", index + 1),
                cwd: seed.cwd,
                profile_id: seed.profile_id,
                profile_label: seed.profile_label,
                startup_command: seed.startup_command,
            })
            .collect::<Vec<_>>();

        let active_pane_id = panes
            .first()
            .map(|pane| pane.id.clone())
            .ok_or_else(|| TabbyError::State(String::from("New tab has no active pane")))?;

        state.tabs.push(TabSnapshot {
            id: tab_id.clone(),
            title,
            preset,
            panes,
            active_pane_id,
        });
        state.active_tab_id = Some(tab_id);
        state.next_tab_index += 1;

        Ok(Self::workspace_snapshot(&state))
    }

    pub fn close_tab(&self, tab_id: &str) -> Result<(WorkspaceSnapshot, Vec<String>), TabbyError> {
        let mut state = self.lock_state()?;
        let index = state
            .tabs
            .iter()
            .position(|tab| tab.id == tab_id)
            .ok_or_else(|| TabbyError::NotFound(format!("Tab {tab_id}")))?;

        let removed = state.tabs.remove(index);
        let session_ids = removed
            .panes
            .iter()
            .map(|pane| pane.session_id.clone())
            .collect::<Vec<_>>();

        state.active_tab_id = if state.tabs.is_empty() {
            None
        } else if removed.id == state.active_tab_id.clone().unwrap_or_default() {
            Some(state.tabs[index.saturating_sub(1)].id.clone())
        } else {
            state.active_tab_id.clone()
        };

        Ok((Self::workspace_snapshot(&state), session_ids))
    }

    pub fn set_active_tab(&self, tab_id: &str) -> Result<WorkspaceSnapshot, TabbyError> {
        let mut state = self.lock_state()?;
        let exists = state.tabs.iter().any(|tab| tab.id == tab_id);
        if !exists {
            return Err(TabbyError::NotFound(format!("Tab {tab_id}")));
        }

        state.active_tab_id = Some(String::from(tab_id));
        Ok(Self::workspace_snapshot(&state))
    }

    pub fn focus_pane(&self, tab_id: &str, pane_id: &str) -> Result<WorkspaceSnapshot, TabbyError> {
        let mut state = self.lock_state()?;
        let tab = state
            .tabs
            .iter_mut()
            .find(|tab| tab.id == tab_id)
            .ok_or_else(|| TabbyError::NotFound(format!("Tab {tab_id}")))?;

        if !tab.panes.iter().any(|pane| pane.id == pane_id) {
            return Err(TabbyError::NotFound(format!("Pane {pane_id}")));
        }

        tab.active_pane_id = String::from(pane_id);
        state.active_tab_id = Some(String::from(tab_id));
        Ok(Self::workspace_snapshot(&state))
    }

    pub fn locate_pane(&self, pane_id: &str) -> Result<LocatedPane, TabbyError> {
        let state = self.lock_state()?;
        state
            .tabs
            .iter()
            .find_map(|tab| {
                tab.panes
                    .iter()
                    .find(|pane| pane.id == pane_id)
                    .map(|pane| LocatedPane { pane: pane.clone() })
            })
            .ok_or_else(|| TabbyError::NotFound(format!("Pane {pane_id}")))
    }

    pub fn replace_pane(
        &self,
        pane_id: &str,
        session_id: String,
        profile_id: String,
        profile_label: String,
        startup_command: Option<String>,
        cwd: String,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let mut state = self.lock_state()?;

        let pane = state
            .tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.id == pane_id)
            .ok_or_else(|| TabbyError::NotFound(format!("Pane {pane_id}")))?;

        pane.session_id = session_id;
        pane.profile_id = profile_id;
        pane.profile_label = profile_label;
        pane.startup_command = startup_command;
        pane.cwd = cwd;

        Ok(Self::workspace_snapshot(&state))
    }

    pub fn session_id_for_pane(&self, pane_id: &str) -> Result<String, TabbyError> {
        Ok(self.locate_pane(pane_id)?.pane.session_id)
    }

    fn workspace_snapshot(state: &WorkspaceState) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            active_tab_id: state.active_tab_id.clone().unwrap_or_default(),
            tabs: state.tabs.clone(),
        }
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, WorkspaceState>, TabbyError> {
        self.state
            .lock()
            .map_err(|_| TabbyError::State(String::from("Workspace state lock poisoned")))
    }
}

#[cfg(test)]
mod tests {
    use super::TabManager;
    use crate::domain::types::{LayoutPreset, PaneSeed};

    fn pane_seed(id: &str) -> PaneSeed {
        PaneSeed {
            pane_id: format!("pane-{id}"),
            session_id: format!("session-{id}"),
            cwd: String::from("/tmp"),
            profile_id: String::from("terminal"),
            profile_label: String::from("Terminal"),
            startup_command: None,
        }
    }

    #[test]
    fn creates_a_tab_and_marks_it_active() {
        let manager = TabManager::new();
        let snapshot = manager
            .create_tab(LayoutPreset::OneByTwo, vec![pane_seed("a"), pane_seed("b")])
            .expect("tab should be created");

        assert_eq!(snapshot.tabs.len(), 1);
        assert_eq!(snapshot.tabs[0].panes.len(), 2);
        assert_eq!(snapshot.active_tab_id, snapshot.tabs[0].id);
    }

    #[test]
    fn closing_active_tab_falls_back_to_previous_tab() {
        let manager = TabManager::new();
        let first = manager
            .create_tab(LayoutPreset::OneByOne, vec![pane_seed("a")])
            .expect("first tab");
        let second = manager
            .create_tab(LayoutPreset::OneByOne, vec![pane_seed("b")])
            .expect("second tab");

        let (snapshot, _) = manager
            .close_tab(&second.active_tab_id)
            .expect("close second tab");

        assert_eq!(snapshot.tabs.len(), 1);
        assert_eq!(snapshot.active_tab_id, first.active_tab_id);
    }
}
