pub mod content;
pub mod ids;
pub mod layout;

use thiserror::Error;
use uuid::Uuid;

pub use content::{BrowserUrl, PaneContentDefinition};
pub use ids::{PaneContentId, PaneId, TabId};

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
    pub pane_id: PaneId,
    pub title: String,
    pub spec: PaneSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub tab_id: TabId,
    pub title: String,
    pub layout: SplitNode,
    pub panes: Vec<PaneSlot>,
    pub active_pane_id: PaneId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSession {
    pub tabs: Vec<Tab>,
    pub active_tab_id: Option<TabId>,
    next_tab_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabLayoutStrategy {
    Preset(LayoutPreset),
    AutoCount,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceDomainEvent {
    PaneAdded { pane_id: PaneId, spec: PaneSpec },
    PaneRemoved { pane_id: PaneId, spec: PaneSpec },
    PaneSpecReplaced { pane_id: PaneId, spec: PaneSpec },
    ActivePaneChanged { pane_id: PaneId, tab_id: TabId },
    ActiveTabChanged { tab_id: TabId },
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
    ) -> Result<Vec<WorkspaceDomainEvent>, WorkspaceError> {
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
            active_pane_id: active_pane_id.clone(),
        });
        self.active_tab_id = Some(tab_id.clone());
        self.next_tab_index += 1;
        self.validate()?;

        let mut events: Vec<WorkspaceDomainEvent> = panes
            .into_iter()
            .map(|pane| WorkspaceDomainEvent::PaneAdded {
                pane_id: pane.pane_id,
                spec: pane.spec,
            })
            .collect();
        events.push(WorkspaceDomainEvent::ActiveTabChanged {
            tab_id: tab_id.clone(),
        });
        events.push(WorkspaceDomainEvent::ActivePaneChanged {
            pane_id: active_pane_id,
            tab_id,
        });
        Ok(events)
    }

    pub fn close_tab(
        &mut self,
        tab_id: &TabId,
    ) -> Result<Vec<WorkspaceDomainEvent>, WorkspaceError> {
        let index = self
            .tabs
            .iter()
            .position(|tab| tab.tab_id == *tab_id)
            .ok_or_else(|| WorkspaceError::NotFound(format!("tab {tab_id}")))?;

        let was_active = self.active_tab_id.as_ref() == Some(tab_id);
        let removed = self.tabs.remove(index);
        self.active_tab_id = if self.tabs.is_empty() {
            None
        } else if was_active {
            Some(self.tabs[index.saturating_sub(1)].tab_id.clone())
        } else {
            self.active_tab_id.clone()
        };
        self.validate()?;

        let mut events: Vec<WorkspaceDomainEvent> = removed
            .panes
            .into_iter()
            .map(|pane| WorkspaceDomainEvent::PaneRemoved {
                pane_id: pane.pane_id,
                spec: pane.spec,
            })
            .collect();

        if was_active {
            if let Some(new_active_tab_id) = &self.active_tab_id {
                events.push(WorkspaceDomainEvent::ActiveTabChanged {
                    tab_id: new_active_tab_id.clone(),
                });
            }
        }

        Ok(events)
    }

    pub fn focus_pane(
        &mut self,
        tab_id: &TabId,
        pane_id: &PaneId,
    ) -> Result<Vec<WorkspaceDomainEvent>, WorkspaceError> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|tab| tab.tab_id == *tab_id)
            .ok_or_else(|| WorkspaceError::NotFound(format!("tab {tab_id}")))?;
        if !tab.panes.iter().any(|pane| pane.pane_id == *pane_id) {
            return Err(WorkspaceError::NotFound(format!("pane {pane_id}")));
        }

        let mut events = Vec::new();
        let tab_changed = self.active_tab_id.as_ref() != Some(tab_id);
        let pane_changed = tab.active_pane_id != *pane_id;

        tab.active_pane_id = pane_id.clone();
        self.active_tab_id = Some(tab_id.clone());
        self.validate()?;

        if tab_changed {
            events.push(WorkspaceDomainEvent::ActiveTabChanged {
                tab_id: tab_id.clone(),
            });
        }
        if pane_changed || tab_changed {
            events.push(WorkspaceDomainEvent::ActivePaneChanged {
                pane_id: pane_id.clone(),
                tab_id: tab_id.clone(),
            });
        }
        Ok(events)
    }

    pub fn set_active_tab(
        &mut self,
        tab_id: &TabId,
    ) -> Result<Vec<WorkspaceDomainEvent>, WorkspaceError> {
        if !self.tabs.iter().any(|tab| tab.tab_id == *tab_id) {
            return Err(WorkspaceError::NotFound(format!("tab {tab_id}")));
        }
        let changed = self.active_tab_id.as_ref() != Some(tab_id);
        self.active_tab_id = Some(tab_id.clone());
        self.validate()?;

        if changed {
            Ok(vec![WorkspaceDomainEvent::ActiveTabChanged {
                tab_id: tab_id.clone(),
            }])
        } else {
            Ok(vec![])
        }
    }

    pub fn split_pane(
        &mut self,
        pane_id: &PaneId,
        direction: SplitDirection,
        spec: PaneSpec,
    ) -> Result<Vec<WorkspaceDomainEvent>, WorkspaceError> {
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

        Ok(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: new_pane.pane_id,
            spec: new_pane.spec,
        }])
    }

    pub fn close_pane(
        &mut self,
        pane_id: &PaneId,
    ) -> Result<Vec<WorkspaceDomainEvent>, WorkspaceError> {
        let (tab_index, pane_index) = self.locate_pane(pane_id)?;
        let close_result =
            close_pane_layout(&self.tabs[tab_index].layout, pane_id).ok_or_else(|| {
                WorkspaceError::State(format!("failed to close pane {pane_id} in layout"))
            })?;
        let was_active_pane = self.tabs[tab_index].active_pane_id == *pane_id;
        let removed = self.tabs[tab_index].panes.remove(pane_index);

        let mut extra_events = Vec::new();

        match close_result {
            Some(next_layout) => {
                self.tabs[tab_index].layout = next_layout;
                if was_active_pane {
                    let new_active = self.tabs[tab_index]
                        .panes
                        .first()
                        .map(|pane| pane.pane_id.clone())
                        .unwrap_or_else(|| PaneId::from(String::new()));
                    self.tabs[tab_index].active_pane_id = new_active.clone();
                    let tab_id = self.tabs[tab_index].tab_id.clone();
                    extra_events.push(WorkspaceDomainEvent::ActivePaneChanged {
                        pane_id: new_active,
                        tab_id,
                    });
                }
            }
            None => {
                let removed_tab_id = self.tabs[tab_index].tab_id.clone();
                let was_active_tab = self.active_tab_id.as_ref() == Some(&removed_tab_id);
                self.tabs.remove(tab_index);
                self.active_tab_id = if self.tabs.is_empty() {
                    None
                } else if was_active_tab {
                    Some(self.tabs[tab_index.saturating_sub(1)].tab_id.clone())
                } else {
                    self.active_tab_id.clone()
                };
                if was_active_tab {
                    if let Some(new_tab_id) = &self.active_tab_id {
                        extra_events.push(WorkspaceDomainEvent::ActiveTabChanged {
                            tab_id: new_tab_id.clone(),
                        });
                    }
                }
            }
        }
        self.validate()?;

        let mut events = vec![WorkspaceDomainEvent::PaneRemoved {
            pane_id: removed.pane_id,
            spec: removed.spec,
        }];
        events.extend(extra_events);
        Ok(events)
    }

    pub fn swap_pane_slots(
        &mut self,
        pane_id_a: &PaneId,
        pane_id_b: &PaneId,
    ) -> Result<(), WorkspaceError> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|tab| {
                tab.panes.iter().any(|pane| pane.pane_id == *pane_id_a)
                    && tab.panes.iter().any(|pane| pane.pane_id == *pane_id_b)
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
        pane_id: &PaneId,
        spec: PaneSpec,
    ) -> Result<Vec<WorkspaceDomainEvent>, WorkspaceError> {
        let (_, pane_index) = self.locate_pane(pane_id)?;
        let pane = self
            .tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.pane_id == *pane_id)
            .ok_or_else(|| WorkspaceError::NotFound(format!("pane {pane_id}")))?;
        pane.spec = spec.clone();
        let _ = pane_index;
        self.validate()?;

        Ok(vec![WorkspaceDomainEvent::PaneSpecReplaced {
            pane_id: pane_id.clone(),
            spec,
        }])
    }

    pub fn track_terminal_working_directory(
        &mut self,
        pane_id: &PaneId,
        working_directory: &str,
    ) -> Result<(), WorkspaceError> {
        let pane = self
            .tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.pane_id == *pane_id)
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

    pub fn pane_spec(&self, pane_id: &PaneId) -> Option<PaneSpec> {
        self.tabs
            .iter()
            .flat_map(|tab| tab.panes.iter())
            .find(|pane| pane.pane_id == *pane_id)
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

        if !self.tabs.iter().any(|tab| tab.tab_id == *active_tab_id) {
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

    fn locate_pane(&self, pane_id: &PaneId) -> Result<(usize, usize), WorkspaceError> {
        for (tab_index, tab) in self.tabs.iter().enumerate() {
            if let Some(pane_index) = tab.panes.iter().position(|pane| pane.pane_id == *pane_id) {
                return Ok((tab_index, pane_index));
            }
        }

        Err(WorkspaceError::NotFound(format!("pane {pane_id}")))
    }
}

fn resolve_layout(
    layout_strategy: &TabLayoutStrategy,
    pane_ids: &[PaneId],
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

pub fn create_pane_id() -> PaneId {
    PaneId::from(Uuid::new_v4().to_string())
}

pub fn create_tab_id() -> TabId {
    TabId::from(Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        layout::{LayoutPreset, SplitDirection},
        BrowserPaneSpec, PaneId, PaneSpec, TabId, TabLayoutStrategy, WorkspaceDomainEvent,
        WorkspaceSession,
    };

    fn terminal(cwd: &str) -> PaneSpec {
        PaneSpec::Terminal(super::TerminalPaneSpec {
            launch_profile_id: String::from("terminal"),
            working_directory: String::from(cwd),
            command_override: None,
        })
    }

    fn browser(url: &str) -> PaneSpec {
        PaneSpec::Browser(BrowserPaneSpec {
            initial_url: String::from(url),
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
        // 2 PaneAdded + 1 ActiveTabChanged + 1 ActivePaneChanged = 4
        assert_eq!(events.len(), 4);
        assert!(workspace.active_tab_id.is_some());
    }

    #[test]
    fn open_tab_emits_pane_added_and_active_events() {
        let mut workspace = WorkspaceSession::default();
        let events = workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");

        let tab_id = workspace.tabs[0].tab_id.clone();
        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();

        // PaneAdded
        assert!(
            matches!(&events[0], WorkspaceDomainEvent::PaneAdded { pane_id: pid, spec: PaneSpec::Terminal(_) } if *pid == pane_id)
        );
        // ActiveTabChanged
        assert!(
            matches!(&events[1], WorkspaceDomainEvent::ActiveTabChanged { tab_id: tid } if *tid == tab_id)
        );
        // ActivePaneChanged
        assert!(
            matches!(&events[2], WorkspaceDomainEvent::ActivePaneChanged { pane_id: pid, tab_id: tid } if *pid == pane_id && *tid == tab_id)
        );
    }

    #[test]
    fn split_pane_emits_pane_added_with_correct_spec() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");
        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();

        let browser_spec = browser("https://example.com");
        let events = workspace
            .split_pane(&pane_id, SplitDirection::Horizontal, browser_spec.clone())
            .expect("split should succeed");

        assert_eq!(events.len(), 1);
        match &events[0] {
            WorkspaceDomainEvent::PaneAdded { spec, .. } => {
                assert_eq!(*spec, browser_spec);
            }
            other => panic!("expected PaneAdded, got {other:?}"),
        }
    }

    #[test]
    fn close_pane_emits_pane_removed() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
                vec![terminal("/a"), terminal("/b")],
            )
            .expect("tab should open");

        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();
        let events = workspace
            .close_pane(&pane_id)
            .expect("close should succeed");

        assert!(events.iter().any(|e| matches!(
            e,
            WorkspaceDomainEvent::PaneRemoved { pane_id: pid, .. } if *pid == pane_id
        )));
    }

    #[test]
    fn close_active_pane_emits_active_pane_changed() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
                vec![terminal("/a"), terminal("/b")],
            )
            .expect("tab should open");

        // Close the active pane (first pane)
        let active_pane_id = workspace.tabs[0].active_pane_id.clone();
        let events = workspace
            .close_pane(&active_pane_id)
            .expect("close should succeed");

        assert!(events
            .iter()
            .any(|e| matches!(e, WorkspaceDomainEvent::ActivePaneChanged { .. })));
    }

    #[test]
    fn replace_pane_spec_emits_pane_spec_replaced() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");
        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();

        let new_spec = browser("https://example.com");
        let events = workspace
            .replace_pane_spec(&pane_id, new_spec.clone())
            .expect("replace should succeed");

        assert_eq!(events.len(), 1);
        match &events[0] {
            WorkspaceDomainEvent::PaneSpecReplaced { pane_id: pid, spec } => {
                assert_eq!(*pid, pane_id);
                assert_eq!(*spec, new_spec);
            }
            other => panic!("expected PaneSpecReplaced, got {other:?}"),
        }
    }

    #[test]
    fn set_active_tab_emits_active_tab_changed() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/a")],
            )
            .expect("first tab");
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/b")],
            )
            .expect("second tab");

        let first_tab_id = workspace.tabs[0].tab_id.clone();
        let events = workspace
            .set_active_tab(&first_tab_id)
            .expect("set active tab");

        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], WorkspaceDomainEvent::ActiveTabChanged { tab_id } if *tab_id == first_tab_id)
        );
    }

    #[test]
    fn set_active_tab_same_tab_emits_nothing() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/a")],
            )
            .expect("tab");

        let tab_id = workspace.tabs[0].tab_id.clone();
        let events = workspace.set_active_tab(&tab_id).expect("set active tab");

        assert!(events.is_empty(), "no event when tab is already active");
    }

    #[test]
    fn focus_pane_emits_active_pane_changed() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
                vec![terminal("/a"), terminal("/b")],
            )
            .expect("tab");

        let tab_id = workspace.tabs[0].tab_id.clone();
        let second_pane_id = workspace.tabs[0].panes[1].pane_id.clone();

        let events = workspace
            .focus_pane(&tab_id, &second_pane_id)
            .expect("focus pane");

        assert!(events
            .iter()
            .any(|e| matches!(e, WorkspaceDomainEvent::ActivePaneChanged { pane_id, .. } if *pane_id == second_pane_id)));
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
                browser("https://example.com"),
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

    #[test]
    fn events_carry_no_transport_types() {
        // Negative case: WorkspaceDomainEvent only uses domain types (newtypes, PaneSpec).
        // This test verifies that events can be constructed without any Tauri/transport imports.
        let event = WorkspaceDomainEvent::PaneAdded {
            pane_id: PaneId::from(String::from("p1")),
            spec: terminal("/tmp"),
        };
        let event2 = WorkspaceDomainEvent::ActivePaneChanged {
            pane_id: PaneId::from(String::from("p1")),
            tab_id: TabId::from(String::from("t1")),
        };
        let event3 = WorkspaceDomainEvent::ActiveTabChanged {
            tab_id: TabId::from(String::from("t1")),
        };

        // If these compile and are Debug-printable, they are transport-free
        assert!(!format!("{event:?}").is_empty());
        assert!(!format!("{event2:?}").is_empty());
        assert!(!format!("{event3:?}").is_empty());
    }

    /// Negative case: domain functions no longer accept raw String for tab/pane ids.
    /// This is a compile-time guarantee — these tests verify the type system enforces it.
    #[test]
    fn domain_functions_require_typed_ids() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");

        let tab_id: TabId = workspace.tabs[0].tab_id.clone();
        let _pane_id: PaneId = workspace.tabs[0].panes[0].pane_id.clone();

        // These calls prove that TabId/PaneId are required (not String)
        workspace.close_tab(&tab_id).ok();

        let mut workspace2 = WorkspaceSession::default();
        workspace2
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
                vec![terminal("/a"), terminal("/b")],
            )
            .expect("tab");
        let tab_id2 = workspace2.tabs[0].tab_id.clone();
        let pane_id2 = workspace2.tabs[0].panes[0].pane_id.clone();

        workspace2.focus_pane(&tab_id2, &pane_id2).ok();
        workspace2
            .split_pane(&pane_id2, SplitDirection::Horizontal, terminal("/c"))
            .ok();
        workspace2.set_active_tab(&tab_id2).ok();
        workspace2.pane_spec(&pane_id2);
    }
}
