pub mod layout;

use thiserror::Error;
use uuid::Uuid;

use crate::layout::{
    close_pane as close_pane_layout, split_pane as split_pane_layout, swap_panes, tree_from_count,
    tree_from_preset, validate_layout, LayoutError, LayoutPreset, SplitDirection, SplitNode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalPaneSpec {
    pub launch_profile_id: String,
    pub working_directory: String,
    pub command_override: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserPaneSpec {
    pub initial_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneSpec {
    Terminal(TerminalPaneSpec),
    Browser(BrowserPaneSpec),
}

impl PaneSpec {
    pub fn terminal_profile_id(&self) -> Option<&str> {
        match self {
            Self::Terminal(spec) => Some(spec.launch_profile_id.as_str()),
            Self::Browser(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneSlot {
    pub pane_id: String,
    pub title: String,
    pub spec: PaneSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub tab_id: String,
    pub title: String,
    pub layout: SplitNode,
    pub panes: Vec<PaneSlot>,
    pub active_pane_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSession {
    pub tabs: Vec<Tab>,
    pub active_tab_id: Option<String>,
    next_tab_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabLayoutStrategy {
    Preset(LayoutPreset),
    AutoCount,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceEvent {
    PaneAdded { pane_id: String, spec: PaneSpec },
    PaneRemoved { pane_id: String, spec: PaneSpec },
    PaneSpecReplaced { pane_id: String, spec: PaneSpec },
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("item not found: {0}")]
    NotFound(String),
    #[error("state error: {0}")]
    State(String),
}

impl From<LayoutError> for WorkspaceError {
    fn from(value: LayoutError) -> Self {
        Self::Validation(value.to_string())
    }
}

impl Default for WorkspaceSession {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            next_tab_index: 1,
        }
    }
}

impl WorkspaceSession {
    pub fn open_tab(
        &mut self,
        layout_strategy: TabLayoutStrategy,
        pane_specs: Vec<PaneSpec>,
    ) -> Result<Vec<WorkspaceEvent>, WorkspaceError> {
        if pane_specs.is_empty() || pane_specs.len() > 9 {
            return Err(WorkspaceError::Validation(format!(
                "tab pane count must be between 1 and 9, got {}",
                pane_specs.len()
            )));
        }

        let tab_id = create_tab_id();
        let title = format!("Workspace {}", self.next_tab_index);
        let panes = pane_specs
            .into_iter()
            .enumerate()
            .map(|(index, spec)| PaneSlot {
                pane_id: create_pane_id(),
                title: format!("Pane {}", index + 1),
                spec,
            })
            .collect::<Vec<_>>();

        let pane_ids = panes
            .iter()
            .map(|pane| pane.pane_id.clone())
            .collect::<Vec<_>>();
        let layout = resolve_layout(&layout_strategy, &pane_ids)?;
        let active_pane_id = pane_ids
            .first()
            .cloned()
            .ok_or_else(|| WorkspaceError::State(String::from("new tab has no active pane")))?;

        self.tabs.push(Tab {
            tab_id: tab_id.clone(),
            title,
            layout,
            panes: panes.clone(),
            active_pane_id,
        });
        self.active_tab_id = Some(tab_id);
        self.next_tab_index += 1;
        self.validate()?;

        Ok(panes
            .into_iter()
            .map(|pane| WorkspaceEvent::PaneAdded {
                pane_id: pane.pane_id,
                spec: pane.spec,
            })
            .collect())
    }

    pub fn close_tab(&mut self, tab_id: &str) -> Result<Vec<WorkspaceEvent>, WorkspaceError> {
        let index = self
            .tabs
            .iter()
            .position(|tab| tab.tab_id == tab_id)
            .ok_or_else(|| WorkspaceError::NotFound(format!("tab {tab_id}")))?;

        let removed = self.tabs.remove(index);
        self.active_tab_id = if self.tabs.is_empty() {
            None
        } else if self.active_tab_id.as_deref() == Some(tab_id) {
            Some(self.tabs[index.saturating_sub(1)].tab_id.clone())
        } else {
            self.active_tab_id.clone()
        };
        self.validate()?;

        Ok(removed
            .panes
            .into_iter()
            .map(|pane| WorkspaceEvent::PaneRemoved {
                pane_id: pane.pane_id,
                spec: pane.spec,
            })
            .collect())
    }

    pub fn focus_pane(&mut self, tab_id: &str, pane_id: &str) -> Result<(), WorkspaceError> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|tab| tab.tab_id == tab_id)
            .ok_or_else(|| WorkspaceError::NotFound(format!("tab {tab_id}")))?;
        if !tab.panes.iter().any(|pane| pane.pane_id == pane_id) {
            return Err(WorkspaceError::NotFound(format!("pane {pane_id}")));
        }

        tab.active_pane_id = String::from(pane_id);
        self.active_tab_id = Some(String::from(tab_id));
        self.validate()
    }

    pub fn set_active_tab(&mut self, tab_id: &str) -> Result<(), WorkspaceError> {
        if !self.tabs.iter().any(|tab| tab.tab_id == tab_id) {
            return Err(WorkspaceError::NotFound(format!("tab {tab_id}")));
        }
        self.active_tab_id = Some(String::from(tab_id));
        self.validate()
    }

    pub fn split_pane(
        &mut self,
        pane_id: &str,
        direction: SplitDirection,
        spec: PaneSpec,
    ) -> Result<Vec<WorkspaceEvent>, WorkspaceError> {
        let (tab_index, _) = self.locate_pane(pane_id)?;
        let new_pane = PaneSlot {
            pane_id: create_pane_id(),
            title: format!("Pane {}", self.tabs[tab_index].panes.len() + 1),
            spec,
        };

        let next_layout = split_pane_layout(
            &self.tabs[tab_index].layout,
            pane_id,
            direction,
            &new_pane.pane_id,
        )
        .ok_or_else(|| {
            WorkspaceError::State(format!("failed to split pane {pane_id} in layout"))
        })?;

        self.tabs[tab_index].layout = next_layout;
        self.tabs[tab_index].panes.push(new_pane.clone());
        self.validate()?;

        Ok(vec![WorkspaceEvent::PaneAdded {
            pane_id: new_pane.pane_id,
            spec: new_pane.spec,
        }])
    }

    pub fn close_pane(&mut self, pane_id: &str) -> Result<Vec<WorkspaceEvent>, WorkspaceError> {
        let (tab_index, pane_index) = self.locate_pane(pane_id)?;
        let close_result =
            close_pane_layout(&self.tabs[tab_index].layout, pane_id).ok_or_else(|| {
                WorkspaceError::State(format!("failed to close pane {pane_id} in layout"))
            })?;
        let removed = self.tabs[tab_index].panes.remove(pane_index);

        match close_result {
            Some(next_layout) => {
                self.tabs[tab_index].layout = next_layout;
                if self.tabs[tab_index].active_pane_id == pane_id {
                    self.tabs[tab_index].active_pane_id = self.tabs[tab_index]
                        .panes
                        .first()
                        .map(|pane| pane.pane_id.clone())
                        .unwrap_or_default();
                }
            }
            None => {
                let removed_tab_id = self.tabs[tab_index].tab_id.clone();
                self.tabs.remove(tab_index);
                self.active_tab_id = if self.tabs.is_empty() {
                    None
                } else if self.active_tab_id.as_deref() == Some(&removed_tab_id) {
                    Some(self.tabs[tab_index.saturating_sub(1)].tab_id.clone())
                } else {
                    self.active_tab_id.clone()
                };
            }
        }
        self.validate()?;

        Ok(vec![WorkspaceEvent::PaneRemoved {
            pane_id: removed.pane_id,
            spec: removed.spec,
        }])
    }

    pub fn swap_pane_slots(
        &mut self,
        pane_id_a: &str,
        pane_id_b: &str,
    ) -> Result<(), WorkspaceError> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|tab| {
                tab.panes.iter().any(|pane| pane.pane_id == pane_id_a)
                    && tab.panes.iter().any(|pane| pane.pane_id == pane_id_b)
            })
            .ok_or_else(|| {
                WorkspaceError::NotFound(format!(
                    "panes {pane_id_a} and {pane_id_b} must belong to the same tab"
                ))
            })?;

        let next_layout = swap_panes(&tab.layout, pane_id_a, pane_id_b).ok_or_else(|| {
            WorkspaceError::State(format!("failed to swap panes {pane_id_a} and {pane_id_b}"))
        })?;
        tab.layout = next_layout;
        self.validate()
    }

    pub fn replace_pane_spec(
        &mut self,
        pane_id: &str,
        spec: PaneSpec,
    ) -> Result<Vec<WorkspaceEvent>, WorkspaceError> {
        let (_, pane_index) = self.locate_pane(pane_id)?;
        let pane = self
            .tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.pane_id == pane_id)
            .ok_or_else(|| WorkspaceError::NotFound(format!("pane {pane_id}")))?;
        pane.spec = spec.clone();
        let _ = pane_index;
        self.validate()?;

        Ok(vec![WorkspaceEvent::PaneSpecReplaced {
            pane_id: String::from(pane_id),
            spec,
        }])
    }

    pub fn track_terminal_working_directory(
        &mut self,
        pane_id: &str,
        working_directory: &str,
    ) -> Result<(), WorkspaceError> {
        let pane = self
            .tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.pane_id == pane_id)
            .ok_or_else(|| WorkspaceError::NotFound(format!("pane {pane_id}")))?;

        match &mut pane.spec {
            PaneSpec::Terminal(spec) => {
                spec.working_directory = String::from(working_directory);
            }
            PaneSpec::Browser(_) => {
                return Err(WorkspaceError::Validation(String::from(
                    "browser panes do not track working directories",
                )))
            }
        }
        self.validate()
    }

    pub fn pane_spec(&self, pane_id: &str) -> Option<PaneSpec> {
        self.tabs
            .iter()
            .flat_map(|tab| tab.panes.iter())
            .find(|pane| pane.pane_id == pane_id)
            .map(|pane| pane.spec.clone())
    }

    pub fn tab_summaries(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn validate(&self) -> Result<(), WorkspaceError> {
        if self.tabs.is_empty() {
            if self.active_tab_id.is_some() {
                return Err(WorkspaceError::State(String::from(
                    "workspace has no tabs but still points to an active tab",
                )));
            }
            return Ok(());
        }

        let active_tab_id = self.active_tab_id.as_ref().ok_or_else(|| {
            WorkspaceError::State(String::from("workspace is missing an active tab"))
        })?;

        if !self.tabs.iter().any(|tab| &tab.tab_id == active_tab_id) {
            return Err(WorkspaceError::State(String::from(
                "active tab does not exist in workspace",
            )));
        }

        for tab in &self.tabs {
            if tab.panes.is_empty() {
                return Err(WorkspaceError::State(format!(
                    "tab {} does not contain any panes",
                    tab.tab_id
                )));
            }

            if !tab
                .panes
                .iter()
                .any(|pane| pane.pane_id == tab.active_pane_id)
            {
                return Err(WorkspaceError::State(format!(
                    "active pane {} is missing in tab {}",
                    tab.active_pane_id, tab.tab_id
                )));
            }

            let pane_ids = tab
                .panes
                .iter()
                .map(|pane| pane.pane_id.clone())
                .collect::<Vec<_>>();
            validate_layout(&tab.layout, &pane_ids)?;
        }

        Ok(())
    }

    fn locate_pane(&self, pane_id: &str) -> Result<(usize, usize), WorkspaceError> {
        for (tab_index, tab) in self.tabs.iter().enumerate() {
            if let Some(pane_index) = tab.panes.iter().position(|pane| pane.pane_id == pane_id) {
                return Ok((tab_index, pane_index));
            }
        }

        Err(WorkspaceError::NotFound(format!("pane {pane_id}")))
    }
}

fn resolve_layout(
    layout_strategy: &TabLayoutStrategy,
    pane_ids: &[String],
) -> Result<SplitNode, WorkspaceError> {
    match layout_strategy {
        TabLayoutStrategy::Preset(preset) => {
            if preset.pane_count() != pane_ids.len() {
                return Err(WorkspaceError::Validation(format!(
                    "layout preset {} expects {} panes, got {}",
                    preset.as_str(),
                    preset.pane_count(),
                    pane_ids.len()
                )));
            }
            Ok(tree_from_preset(*preset, pane_ids))
        }
        TabLayoutStrategy::AutoCount => Ok(tree_from_count(pane_ids)?),
    }
}

pub fn create_pane_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn create_tab_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        layout::{LayoutPreset, SplitDirection},
        BrowserPaneSpec, PaneSpec, TabLayoutStrategy, TerminalPaneSpec, WorkspaceSession,
    };

    fn terminal(cwd: &str) -> PaneSpec {
        PaneSpec::Terminal(TerminalPaneSpec {
            launch_profile_id: String::from("terminal"),
            working_directory: String::from(cwd),
            command_override: None,
        })
    }

    #[test]
    fn opens_tab_and_tracks_active_tab() {
        let mut workspace = WorkspaceSession::default();
        let events = workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
                vec![terminal("/a"), terminal("/b")],
            )
            .expect("tab should open");

        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(events.len(), 2);
        assert!(workspace.active_tab_id.is_some());
    }

    #[test]
    fn splitting_and_closing_pane_keeps_invariants() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");
        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();

        workspace
            .split_pane(
                &pane_id,
                SplitDirection::Horizontal,
                PaneSpec::Browser(BrowserPaneSpec {
                    initial_url: String::from("https://example.com"),
                }),
            )
            .expect("split should succeed");
        workspace
            .close_pane(&pane_id)
            .expect("close should succeed");
        workspace.validate().expect("workspace should remain valid");
    }

    #[test]
    fn track_terminal_working_directory_updates_terminal_spec() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");
        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();

        workspace
            .track_terminal_working_directory(&pane_id, "/projects/tabby")
            .expect("cwd should update");

        match workspace.pane_spec(&pane_id).expect("pane should exist") {
            PaneSpec::Terminal(spec) => assert_eq!(spec.working_directory, "/projects/tabby"),
            PaneSpec::Browser(_) => panic!("expected terminal pane"),
        }
    }
}
