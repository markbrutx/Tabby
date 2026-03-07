use std::sync::Mutex;

use crate::domain::error::TabbyError;
use crate::domain::snapshot::{PaneRuntimeStatus, PaneSnapshot, TabSnapshot, WorkspaceSnapshot};
use crate::domain::split_tree;
use crate::domain::layout::{SplitDirection, SplitNode};
use crate::domain::pane::{create_tab_id, PaneKind, PaneSeed};

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

    pub fn snapshot(&self) -> Result<WorkspaceSnapshot, TabbyError> {
        let state = self.lock_state()?;
        Ok(Self::workspace_snapshot(&state))
    }

    pub fn create_tab(
        &self,
        layout: SplitNode,
        pane_seeds: Vec<PaneSeed>,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let mut state = self.lock_state()?;
        let tab_id = create_tab_id();
        let title = format!("Workspace {}", state.next_tab_index);
        let snapshot = TabSnapshot::from_seeds(
            tab_id.clone(),
            title,
            layout,
            pane_seeds,
            PaneRuntimeStatus::Running,
        )?;

        state.tabs.push(snapshot);
        state.active_tab_id = Some(tab_id);
        state.next_tab_index += 1;

        Ok(Self::workspace_snapshot(&state))
    }

    pub fn split_pane(
        &self,
        target_pane_id: &str,
        direction: SplitDirection,
        new_seed: PaneSeed,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let mut state = self.lock_state()?;

        let tab = state
            .tabs
            .iter_mut()
            .find(|tab| tab.panes.iter().any(|p| p.id == target_pane_id))
            .ok_or_else(|| TabbyError::NotFound(format!("Pane {target_pane_id}")))?;

        let new_layout =
            split_tree::split_pane(&tab.layout, target_pane_id, direction, &new_seed.pane_id)
                .ok_or_else(|| {
                    TabbyError::State(format!(
                        "Failed to split pane {target_pane_id} in layout tree"
                    ))
                })?;

        let new_pane = PaneSnapshot {
            id: new_seed.pane_id,
            session_id: new_seed.session_id,
            title: format!("Pane {}", tab.panes.len() + 1),
            cwd: new_seed.cwd,
            profile_id: new_seed.profile_id,
            profile_label: new_seed.profile_label,
            startup_command: new_seed.startup_command,
            status: PaneRuntimeStatus::Running,
            pane_kind: new_seed.pane_kind,
            url: new_seed.url,
        };

        tab.layout = new_layout;
        tab.panes.push(new_pane);

        Ok(Self::workspace_snapshot(&state))
    }

    /// Closes a pane. Returns the workspace snapshot and the session_id of the
    /// killed pane. If the pane was the last in the tab, also returns the tab_id
    /// that was removed.
    pub fn close_pane(
        &self,
        target_pane_id: &str,
    ) -> Result<(WorkspaceSnapshot, String, Option<String>), TabbyError> {
        let mut state = self.lock_state()?;

        let (tab_index, session_id) = state
            .tabs
            .iter()
            .enumerate()
            .find_map(|(ti, tab)| {
                tab.panes
                    .iter()
                    .find(|p| p.id == target_pane_id)
                    .map(|p| (ti, p.session_id.clone()))
            })
            .ok_or_else(|| TabbyError::NotFound(format!("Pane {target_pane_id}")))?;

        let close_result = split_tree::close_pane(&state.tabs[tab_index].layout, target_pane_id)
            .ok_or_else(|| {
                TabbyError::State(format!(
                    "Failed to close pane {target_pane_id} in layout tree"
                ))
            })?;

        match close_result {
            Some(new_layout) => {
                let tab = &mut state.tabs[tab_index];
                tab.layout = new_layout;
                tab.panes.retain(|p| p.id != target_pane_id);

                if tab.active_pane_id == target_pane_id {
                    tab.active_pane_id =
                        tab.panes.first().map(|p| p.id.clone()).unwrap_or_default();
                }

                Ok((Self::workspace_snapshot(&state), session_id, None))
            }
            None => {
                let removed = state.tabs.remove(tab_index);
                let removed_tab_id = removed.id.clone();

                state.active_tab_id = if state.tabs.is_empty() {
                    None
                } else if state.active_tab_id.as_deref() == Some(&removed_tab_id) {
                    Some(state.tabs[tab_index.saturating_sub(1)].id.clone())
                } else {
                    state.active_tab_id.clone()
                };

                Ok((
                    Self::workspace_snapshot(&state),
                    session_id,
                    Some(removed_tab_id),
                ))
            }
        }
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
        resolved: crate::domain::profiles::ResolvedProfile,
        cwd: String,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        self.replace_pane_full(pane_id, session_id, resolved, cwd, PaneKind::Terminal, None)
    }

    pub fn replace_pane_full(
        &self,
        pane_id: &str,
        session_id: String,
        resolved: crate::domain::profiles::ResolvedProfile,
        cwd: String,
        pane_kind: PaneKind,
        url: Option<String>,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let mut state = self.lock_state()?;

        let pane = state
            .tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.id == pane_id)
            .ok_or_else(|| TabbyError::NotFound(format!("Pane {pane_id}")))?;

        pane.session_id = session_id;
        pane.profile_id = resolved.id;
        pane.profile_label = resolved.label;
        pane.startup_command = resolved.startup_command;
        pane.cwd = cwd;
        pane.pane_kind = pane_kind;
        pane.url = url;

        Ok(Self::workspace_snapshot(&state))
    }

    pub fn terminal_session_ids_for_tab(&self, tab_id: &str) -> Result<Vec<String>, TabbyError> {
        let state = self.lock_state()?;
        let tab = state
            .tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .ok_or_else(|| TabbyError::NotFound(format!("Tab {tab_id}")))?;

        Ok(tab
            .panes
            .iter()
            .filter(|p| p.pane_kind == PaneKind::Terminal)
            .map(|p| p.session_id.clone())
            .collect())
    }

    pub fn browser_pane_ids_for_tab(&self, tab_id: &str) -> Vec<String> {
        let state = match self.lock_state() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        state
            .tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .map(|tab| {
                tab.panes
                    .iter()
                    .filter(|p| p.pane_kind == PaneKind::Browser)
                    .map(|p| p.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn swap_panes(
        &self,
        pane_id_a: &str,
        pane_id_b: &str,
    ) -> Result<WorkspaceSnapshot, TabbyError> {
        let mut state = self.lock_state()?;

        let tab = state
            .tabs
            .iter_mut()
            .find(|tab| {
                tab.panes.iter().any(|p| p.id == pane_id_a)
                    && tab.panes.iter().any(|p| p.id == pane_id_b)
            })
            .ok_or_else(|| {
                TabbyError::NotFound(format!(
                    "Panes {pane_id_a} and {pane_id_b} must be in the same tab"
                ))
            })?;

        let new_layout =
            split_tree::swap_panes(&tab.layout, pane_id_a, pane_id_b).ok_or_else(|| {
                TabbyError::State(format!(
                    "Failed to swap panes {pane_id_a} and {pane_id_b} in layout tree"
                ))
            })?;

        tab.layout = new_layout;

        Ok(Self::workspace_snapshot(&state))
    }

    /// Silently updates the tracked CWD for a pane without restarting the PTY.
    /// Called when the frontend detects an OSC 7 escape sequence.
    pub fn update_tracked_cwd(&self, pane_id: &str, cwd: &str) -> Result<(), TabbyError> {
        let mut state = self.lock_state()?;

        let pane = state
            .tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.id == pane_id)
            .ok_or_else(|| TabbyError::NotFound(format!("Pane {pane_id}")))?;

        pane.cwd = String::from(cwd);
        Ok(())
    }

    fn workspace_snapshot(state: &WorkspaceState) -> WorkspaceSnapshot {
        WorkspaceSnapshot::new(state.active_tab_id.clone(), state.tabs.clone())
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
    use crate::domain::split_tree::tree_from_preset;
    use crate::domain::layout::{LayoutPreset, SplitDirection};
    use crate::domain::pane::PaneSeed;

    fn pane_seed(id: &str) -> PaneSeed {
        PaneSeed {
            pane_id: format!("pane-{id}"),
            session_id: format!("session-{id}"),
            cwd: String::from("/tmp"),
            profile_id: String::from("terminal"),
            profile_label: String::from("Terminal"),
            startup_command: None,
            pane_kind: crate::domain::pane::PaneKind::Terminal,
            url: None,
        }
    }

    #[test]
    fn creates_a_tab_and_marks_it_active() {
        let manager = TabManager::new();
        let seeds = vec![pane_seed("a"), pane_seed("b")];
        let pane_ids: Vec<String> = seeds.iter().map(|s| s.pane_id.clone()).collect();
        let layout = tree_from_preset(LayoutPreset::OneByTwo, &pane_ids);

        let snapshot = manager
            .create_tab(layout, seeds)
            .expect("tab should be created");

        assert_eq!(snapshot.tabs.len(), 1);
        assert_eq!(snapshot.tabs[0].panes.len(), 2);
        assert_eq!(snapshot.active_tab_id, snapshot.tabs[0].id);
    }

    #[test]
    fn closing_active_tab_falls_back_to_previous_tab() {
        let manager = TabManager::new();
        let seeds_a = vec![pane_seed("a")];
        let ids_a: Vec<String> = seeds_a.iter().map(|s| s.pane_id.clone()).collect();
        let first = manager
            .create_tab(tree_from_preset(LayoutPreset::OneByOne, &ids_a), seeds_a)
            .expect("first tab");

        let seeds_b = vec![pane_seed("b")];
        let ids_b: Vec<String> = seeds_b.iter().map(|s| s.pane_id.clone()).collect();
        let second = manager
            .create_tab(tree_from_preset(LayoutPreset::OneByOne, &ids_b), seeds_b)
            .expect("second tab");

        let (snapshot, _) = manager
            .close_tab(&second.active_tab_id)
            .expect("close second tab");

        assert_eq!(snapshot.tabs.len(), 1);
        assert_eq!(snapshot.active_tab_id, first.active_tab_id);
    }

    #[test]
    fn split_pane_adds_new_pane_to_tab() {
        let manager = TabManager::new();
        let seeds = vec![pane_seed("a")];
        let ids: Vec<String> = seeds.iter().map(|s| s.pane_id.clone()).collect();
        manager
            .create_tab(tree_from_preset(LayoutPreset::OneByOne, &ids), seeds)
            .expect("tab created");

        let snapshot = manager
            .split_pane("pane-a", SplitDirection::Horizontal, pane_seed("b"))
            .expect("split should succeed");

        assert_eq!(snapshot.tabs[0].panes.len(), 2);
    }

    #[test]
    fn close_pane_removes_pane_from_tab() {
        let manager = TabManager::new();
        let seeds = vec![pane_seed("a"), pane_seed("b")];
        let ids: Vec<String> = seeds.iter().map(|s| s.pane_id.clone()).collect();
        manager
            .create_tab(tree_from_preset(LayoutPreset::OneByTwo, &ids), seeds)
            .expect("tab created");

        let (snapshot, session_id, removed_tab) =
            manager.close_pane("pane-a").expect("close should succeed");

        assert_eq!(session_id, "session-a");
        assert!(removed_tab.is_none());
        assert_eq!(snapshot.tabs[0].panes.len(), 1);
        assert_eq!(snapshot.tabs[0].panes[0].id, "pane-b");
    }

    #[test]
    fn update_tracked_cwd_changes_snapshot() {
        let manager = TabManager::new();
        let seeds = vec![pane_seed("a")];
        let ids: Vec<String> = seeds.iter().map(|s| s.pane_id.clone()).collect();
        manager
            .create_tab(tree_from_preset(LayoutPreset::OneByOne, &ids), seeds)
            .expect("tab created");

        manager
            .update_tracked_cwd("pane-a", "/new/cwd")
            .expect("cwd update should succeed");

        let located = manager.locate_pane("pane-a").expect("pane found");
        assert_eq!(located.pane.cwd, "/new/cwd");
        assert_eq!(located.pane.session_id, "session-a");
    }

    #[test]
    fn swap_panes_swaps_layout_positions() {
        let manager = TabManager::new();
        let seeds = vec![pane_seed("a"), pane_seed("b")];
        let ids: Vec<String> = seeds.iter().map(|s| s.pane_id.clone()).collect();
        manager
            .create_tab(tree_from_preset(LayoutPreset::OneByTwo, &ids), seeds)
            .expect("tab created");

        let snapshot = manager
            .swap_panes("pane-a", "pane-b")
            .expect("swap should succeed");

        let pane_ids = crate::domain::split_tree::collect_pane_ids(&snapshot.tabs[0].layout);
        assert_eq!(pane_ids, vec!["pane-b", "pane-a"]);
    }

    #[test]
    fn swap_panes_different_tabs_errors() {
        let manager = TabManager::new();
        let seeds_a = vec![pane_seed("a")];
        let ids_a: Vec<String> = seeds_a.iter().map(|s| s.pane_id.clone()).collect();
        manager
            .create_tab(tree_from_preset(LayoutPreset::OneByOne, &ids_a), seeds_a)
            .expect("tab created");

        let seeds_b = vec![pane_seed("b")];
        let ids_b: Vec<String> = seeds_b.iter().map(|s| s.pane_id.clone()).collect();
        manager
            .create_tab(tree_from_preset(LayoutPreset::OneByOne, &ids_b), seeds_b)
            .expect("tab created");

        let result = manager.swap_panes("pane-a", "pane-b");
        assert!(result.is_err());
    }

    #[test]
    fn update_tracked_cwd_unknown_pane_errors() {
        let manager = TabManager::new();
        let result = manager.update_tracked_cwd("nonexistent", "/tmp");
        assert!(result.is_err());
    }

    #[test]
    fn close_last_pane_removes_tab() {
        let manager = TabManager::new();
        let seeds = vec![pane_seed("a")];
        let ids: Vec<String> = seeds.iter().map(|s| s.pane_id.clone()).collect();
        manager
            .create_tab(tree_from_preset(LayoutPreset::OneByOne, &ids), seeds)
            .expect("tab created");

        let (snapshot, _, removed_tab) =
            manager.close_pane("pane-a").expect("close should succeed");

        assert!(removed_tab.is_some());
        assert!(snapshot.tabs.is_empty());
    }
}
