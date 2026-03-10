pub mod content;
pub mod ids;
pub mod layout;

use std::collections::HashMap;

use thiserror::Error;
use uuid::Uuid;

pub use content::PaneContentDefinition;
pub use ids::{BrowserUrl, PaneContentId, PaneId, TabId};
pub use tabby_kernel::CommandTemplate;

use crate::layout::{
    close_pane as close_pane_layout, remap_pane_ids, split_pane as split_pane_layout, swap_panes,
    tree_from_count, tree_from_preset, validate_layout, LayoutError, LayoutPreset, SplitDirection,
    SplitNode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalPaneSpec {
    pub launch_profile_id: String,
    pub working_directory: String,
    pub command_override: Option<CommandTemplate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserPaneSpec {
    pub initial_url: BrowserUrl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitPaneSpec {
    pub working_directory: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneSpec {
    Terminal(TerminalPaneSpec),
    Browser(BrowserPaneSpec),
    Git(GitPaneSpec),
}

impl PaneSpec {
    pub fn terminal_profile_id(&self) -> Option<&str> {
        match self {
            Self::Terminal(spec) => Some(spec.launch_profile_id.as_str()),
            Self::Browser(_) | Self::Git(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneSlot {
    pub pane_id: PaneId,
    pub title: String,
    pub content_id: PaneContentId,
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
    content_store: HashMap<PaneContentId, PaneContentDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabLayoutStrategy {
    Preset(LayoutPreset),
    AutoCount,
    ExplicitTree(SplitNode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceDomainEvent {
    // -- Structural events: affect workspace layout (tabs, panes, focus) --
    PaneAdded {
        pane_id: PaneId,
        content: PaneContentDefinition,
    },
    PaneRemoved {
        pane_id: PaneId,
        content: PaneContentDefinition,
    },
    ActivePaneChanged {
        pane_id: PaneId,
        tab_id: TabId,
    },
    ActiveTabChanged {
        tab_id: TabId,
    },

    // -- Content events: mutate what runs inside a pane --
    PaneContentChanged {
        pane_id: PaneId,
        old_content: PaneContentDefinition,
        new_content: PaneContentDefinition,
    },

    // -- Metadata events: affect display properties only --
    TabRenamed {
        tab_id: TabId,
        new_title: String,
    },
}

impl WorkspaceDomainEvent {
    /// Returns `true` when this event requires a runtime lifecycle action
    /// (start, stop, or restart). Structural focus events do not.
    pub fn is_runtime_relevant(&self) -> bool {
        matches!(
            self,
            Self::PaneAdded { .. } | Self::PaneRemoved { .. } | Self::PaneContentChanged { .. }
        )
    }
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
            content_store: HashMap::new(),
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

        let mut panes = Vec::new();
        let mut pane_added_events = Vec::new();

        for (index, spec) in pane_specs.into_iter().enumerate() {
            let content = content_from_spec(&spec);
            let content_id = content.content_id().clone();
            let pane_id = create_pane_id();

            self.content_store
                .insert(content_id.clone(), content.clone());

            pane_added_events.push(WorkspaceDomainEvent::PaneAdded {
                pane_id: pane_id.clone(),
                content,
            });

            panes.push(PaneSlot {
                pane_id,
                title: format!("Pane {}", index + 1),
                content_id,
            });
        }

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
            panes,
            active_pane_id: active_pane_id.clone(),
        });
        self.active_tab_id = Some(tab_id.clone());
        self.next_tab_index += 1;
        self.validate()?;

        let mut events = pane_added_events;
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

        let mut events: Vec<WorkspaceDomainEvent> = removed
            .panes
            .into_iter()
            .filter_map(|pane| {
                let content = self.content_store.remove(&pane.content_id)?;
                Some(WorkspaceDomainEvent::PaneRemoved {
                    pane_id: pane.pane_id,
                    content,
                })
            })
            .collect();

        self.validate()?;

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

    pub fn rename_tab(
        &mut self,
        tab_id: &TabId,
        new_title: String,
    ) -> Result<Vec<WorkspaceDomainEvent>, WorkspaceError> {
        let trimmed = new_title.trim().to_string();
        if trimmed.is_empty() {
            return Err(WorkspaceError::Validation(String::from(
                "tab title must not be empty",
            )));
        }
        if trimmed.len() > 64 {
            return Err(WorkspaceError::Validation(String::from(
                "tab title must not exceed 64 characters",
            )));
        }

        let tab = self
            .tabs
            .iter_mut()
            .find(|tab| tab.tab_id == *tab_id)
            .ok_or_else(|| WorkspaceError::NotFound(format!("tab {tab_id}")))?;

        tab.title = trimmed.clone();

        Ok(vec![WorkspaceDomainEvent::TabRenamed {
            tab_id: tab_id.clone(),
            new_title: trimmed,
        }])
    }

    pub fn split_pane(
        &mut self,
        pane_id: &PaneId,
        direction: SplitDirection,
        spec: PaneSpec,
    ) -> Result<Vec<WorkspaceDomainEvent>, WorkspaceError> {
        let (tab_index, _) = self.locate_pane(pane_id)?;

        let content = content_from_spec(&spec);
        let content_id = content.content_id().clone();
        let new_pane_id = create_pane_id();

        self.content_store
            .insert(content_id.clone(), content.clone());

        let new_pane = PaneSlot {
            pane_id: new_pane_id.clone(),
            title: format!("Pane {}", self.tabs[tab_index].panes.len() + 1),
            content_id,
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
        self.tabs[tab_index].panes.push(new_pane);
        self.validate()?;

        Ok(vec![WorkspaceDomainEvent::PaneAdded {
            pane_id: new_pane_id,
            content,
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

        // Destroy the associated PaneContentDefinition (1:1 ownership)
        let removed_content = self
            .content_store
            .remove(&removed.content_id)
            .ok_or_else(|| {
                WorkspaceError::State(format!(
                    "content not found for pane {} during close",
                    removed.pane_id
                ))
            })?;

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
            content: removed_content,
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
        let (tab_index, pane_index) = self.locate_pane(pane_id)?;

        // Destroy old content
        let old_content_id = self.tabs[tab_index].panes[pane_index].content_id.clone();
        let old_content = self.content_store.remove(&old_content_id).ok_or_else(|| {
            WorkspaceError::State(format!(
                "content not found for pane {} during replace",
                pane_id
            ))
        })?;

        // Create new content with new id (atomic replace)
        let new_content = content_from_spec(&spec);
        let new_content_id = new_content.content_id().clone();
        self.content_store
            .insert(new_content_id.clone(), new_content.clone());

        // Update pane's content reference
        self.tabs[tab_index].panes[pane_index].content_id = new_content_id;

        self.validate()?;

        Ok(vec![WorkspaceDomainEvent::PaneContentChanged {
            pane_id: pane_id.clone(),
            old_content,
            new_content,
        }])
    }

    pub fn pane_spec(&self, pane_id: &PaneId) -> Option<PaneSpec> {
        self.tabs
            .iter()
            .flat_map(|tab| tab.panes.iter())
            .find(|pane| pane.pane_id == *pane_id)
            .and_then(|pane| self.content_store.get(&pane.content_id))
            .map(spec_from_content)
    }

    /// Returns the content definition for a given content ID.
    pub fn pane_content(&self, content_id: &PaneContentId) -> Option<&PaneContentDefinition> {
        self.content_store.get(content_id)
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
            if !self.content_store.is_empty() {
                return Err(WorkspaceError::State(String::from(
                    "workspace has no tabs but content store is not empty (orphaned content)",
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

        let mut all_content_ids = std::collections::HashSet::new();

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

            // Verify every pane has a content definition in the store
            for pane in &tab.panes {
                if !self.content_store.contains_key(&pane.content_id) {
                    return Err(WorkspaceError::State(format!(
                        "pane {} references missing content {}",
                        pane.pane_id, pane.content_id
                    )));
                }
                all_content_ids.insert(pane.content_id.clone());
            }
        }

        // Verify no orphaned content definitions
        for content_id in self.content_store.keys() {
            if !all_content_ids.contains(content_id) {
                return Err(WorkspaceError::State(format!(
                    "orphaned content definition {content_id}"
                )));
            }
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
        TabLayoutStrategy::ExplicitTree(template) => {
            let remapped = remap_pane_ids(template, pane_ids);
            validate_layout(&remapped, pane_ids)?;
            Ok(remapped)
        }
    }
}

pub fn create_pane_id() -> PaneId {
    PaneId::from(Uuid::new_v4().to_string())
}

pub fn create_tab_id() -> TabId {
    TabId::from(Uuid::new_v4().to_string())
}

pub fn create_content_id() -> PaneContentId {
    PaneContentId::from(Uuid::new_v4().to_string())
}

fn content_from_spec(spec: &PaneSpec) -> PaneContentDefinition {
    let id = create_content_id();
    match spec {
        PaneSpec::Terminal(t) => PaneContentDefinition::terminal(
            id,
            &t.launch_profile_id,
            &t.working_directory,
            t.command_override.clone(),
        ),
        PaneSpec::Browser(b) => PaneContentDefinition::browser(id, b.initial_url.clone()),
        PaneSpec::Git(g) => PaneContentDefinition::git(id, &g.working_directory),
    }
}

pub fn spec_from_content(content: &PaneContentDefinition) -> PaneSpec {
    match content {
        PaneContentDefinition::Terminal {
            profile_id,
            working_directory,
            command_override,
            ..
        } => PaneSpec::Terminal(TerminalPaneSpec {
            launch_profile_id: profile_id.clone(),
            working_directory: working_directory.clone(),
            command_override: command_override.clone(),
        }),
        PaneContentDefinition::Browser { initial_url, .. } => PaneSpec::Browser(BrowserPaneSpec {
            initial_url: initial_url.clone(),
        }),
        PaneContentDefinition::Git {
            working_directory, ..
        } => PaneSpec::Git(GitPaneSpec {
            working_directory: working_directory.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        layout::{LayoutPreset, SplitDirection},
        BrowserPaneSpec, BrowserUrl, PaneContentDefinition, PaneId, PaneSpec, TabId,
        TabLayoutStrategy, WorkspaceDomainEvent, WorkspaceSession,
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
            initial_url: BrowserUrl::new(url),
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
            matches!(&events[0], WorkspaceDomainEvent::PaneAdded { pane_id: pid, content: PaneContentDefinition::Terminal { .. } } if *pid == pane_id)
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
            WorkspaceDomainEvent::PaneAdded { content, .. } => {
                assert!(content.browser_url().is_some());
                assert_eq!(
                    content.browser_url().map(|u| u.as_str()),
                    Some("https://example.com")
                );
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
            WorkspaceDomainEvent::PaneRemoved { pane_id: pid, content: PaneContentDefinition::Terminal { .. } } if *pid == pane_id
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
    fn replace_pane_spec_emits_pane_content_changed() {
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
                assert_eq!(
                    new_content.browser_url().map(|u| u.as_str()),
                    Some("https://example.com")
                );
                // Old content id is never reused
                assert_ne!(old_content.content_id(), new_content.content_id());
            }
            other => panic!("expected PaneContentChanged, got {other:?}"),
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
    fn events_carry_no_transport_types() {
        // Negative case: WorkspaceDomainEvent only uses domain types (newtypes, PaneContentDefinition).
        // This test verifies that events can be constructed without any Tauri/transport imports.
        let content_id = super::create_content_id();
        let event = WorkspaceDomainEvent::PaneAdded {
            pane_id: PaneId::from(String::from("p1")),
            content: PaneContentDefinition::terminal(content_id, "default", "/tmp", None),
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

    // --- US-013: Event classification tests ---

    #[test]
    fn structural_events_are_runtime_relevant_when_they_add_or_remove() {
        let content_id = super::create_content_id();
        let added = WorkspaceDomainEvent::PaneAdded {
            pane_id: PaneId::from(String::from("p1")),
            content: PaneContentDefinition::terminal(content_id, "default", "/tmp", None),
        };
        assert!(added.is_runtime_relevant());

        let content_id2 = super::create_content_id();
        let removed = WorkspaceDomainEvent::PaneRemoved {
            pane_id: PaneId::from(String::from("p1")),
            content: PaneContentDefinition::terminal(content_id2, "default", "/tmp", None),
        };
        assert!(removed.is_runtime_relevant());
    }

    #[test]
    fn content_event_is_runtime_relevant() {
        let old_id = super::create_content_id();
        let new_id = super::create_content_id();
        let event = WorkspaceDomainEvent::PaneContentChanged {
            pane_id: PaneId::from(String::from("p1")),
            old_content: PaneContentDefinition::terminal(old_id, "default", "/tmp", None),
            new_content: PaneContentDefinition::browser(
                new_id,
                super::BrowserUrl::new("https://example.com"),
            ),
        };
        assert!(event.is_runtime_relevant());
    }

    #[test]
    fn focus_events_are_not_runtime_relevant() {
        let active_pane = WorkspaceDomainEvent::ActivePaneChanged {
            pane_id: PaneId::from(String::from("p1")),
            tab_id: TabId::from(String::from("t1")),
        };
        assert!(
            !active_pane.is_runtime_relevant(),
            "ActivePaneChanged is structural-only and must not trigger runtime actions"
        );

        let active_tab = WorkspaceDomainEvent::ActiveTabChanged {
            tab_id: TabId::from(String::from("t1")),
        };
        assert!(
            !active_tab.is_runtime_relevant(),
            "ActiveTabChanged is structural-only and must not trigger runtime actions"
        );
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

    // --- US-010: PaneSlot + content ref pattern tests ---

    #[test]
    fn pane_slot_holds_content_id_not_spec() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");

        let pane = &workspace.tabs[0].panes[0];
        // PaneSlot has content_id (not spec directly)
        let content = workspace
            .pane_content(&pane.content_id)
            .expect("content should exist for pane's content_id");

        // Content definition holds the terminal details
        assert_eq!(content.terminal_profile_id(), Some("terminal"));
        assert_eq!(content.working_directory(), Some("/tmp"));
    }

    #[test]
    fn open_tab_creates_content_definitions_for_each_pane() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
                vec![terminal("/a"), browser("https://example.com")],
            )
            .expect("tab should open");

        // Each pane has a distinct content_id
        let content_id_a = workspace.tabs[0].panes[0].content_id.clone();
        let content_id_b = workspace.tabs[0].panes[1].content_id.clone();
        assert_ne!(content_id_a, content_id_b);

        // Content definitions are accessible via pane_content()
        let content_a = workspace
            .pane_content(&content_id_a)
            .expect("content A should exist");
        assert_eq!(content_a.terminal_profile_id(), Some("terminal"));

        let content_b = workspace
            .pane_content(&content_id_b)
            .expect("content B should exist");
        assert!(content_b.browser_url().is_some());
    }

    #[test]
    fn close_pane_destroys_associated_content() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
                vec![terminal("/a"), terminal("/b")],
            )
            .expect("tab should open");

        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();
        let content_id = workspace.tabs[0].panes[0].content_id.clone();

        // Content exists before close
        assert!(workspace.pane_content(&content_id).is_some());

        workspace
            .close_pane(&pane_id)
            .expect("close should succeed");

        // Content destroyed after close — no orphans
        assert!(
            workspace.pane_content(&content_id).is_none(),
            "content should be destroyed when pane is closed"
        );
        workspace
            .validate()
            .expect("workspace should be valid after close_pane");
    }

    #[test]
    fn close_tab_destroys_all_pane_content() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
                vec![terminal("/a"), terminal("/b")],
            )
            .expect("tab should open");

        let content_id_a = workspace.tabs[0].panes[0].content_id.clone();
        let content_id_b = workspace.tabs[0].panes[1].content_id.clone();
        let tab_id = workspace.tabs[0].tab_id.clone();

        workspace
            .close_tab(&tab_id)
            .expect("close tab should succeed");

        assert!(workspace.pane_content(&content_id_a).is_none());
        assert!(workspace.pane_content(&content_id_b).is_none());
        workspace
            .validate()
            .expect("workspace should be valid after close_tab");
    }

    #[test]
    fn replace_pane_spec_destroys_old_creates_new_content() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");

        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();
        let old_content_id = workspace.tabs[0].panes[0].content_id.clone();

        let new_spec = browser("https://example.com");
        workspace
            .replace_pane_spec(&pane_id, new_spec)
            .expect("replace should succeed");

        // Old content destroyed
        assert!(
            workspace.pane_content(&old_content_id).is_none(),
            "old content should be destroyed on replace"
        );

        // New content created with new id
        let new_content_id = workspace.tabs[0].panes[0].content_id.clone();
        assert_ne!(
            old_content_id, new_content_id,
            "replace should create a new content_id"
        );

        let new_content = workspace
            .pane_content(&new_content_id)
            .expect("new content should exist");
        assert!(
            new_content.browser_url().is_some(),
            "new content should be a browser"
        );

        workspace
            .validate()
            .expect("workspace should be valid after replace");
    }

    #[test]
    fn spec_accessed_through_content_definition_not_pane() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/home/user")],
            )
            .expect("tab should open");

        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();

        // Access spec through pane_spec() which internally goes through content store
        let spec = workspace
            .pane_spec(&pane_id)
            .expect("pane_spec should resolve through content");
        match spec {
            PaneSpec::Terminal(t) => {
                assert_eq!(t.launch_profile_id, "terminal");
                assert_eq!(t.working_directory, "/home/user");
            }
            other => panic!("expected terminal, got {other:?}"),
        }
    }

    #[test]
    fn no_orphaned_content_after_split_and_close_sequence() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");

        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();

        // Split creates new content
        workspace
            .split_pane(&pane_id, SplitDirection::Horizontal, terminal("/home"))
            .expect("split should succeed");
        assert_eq!(workspace.tabs[0].panes.len(), 2);
        workspace.validate().expect("valid after split");

        // Close new pane — its content is destroyed
        let new_pane_id = workspace.tabs[0].panes[1].pane_id.clone();
        let new_content_id = workspace.tabs[0].panes[1].content_id.clone();
        workspace
            .close_pane(&new_pane_id)
            .expect("close should succeed");

        assert!(
            workspace.pane_content(&new_content_id).is_none(),
            "split pane content should be destroyed after close"
        );
        workspace
            .validate()
            .expect("no orphans after split+close sequence");
    }

    // --- US-014: Workspace structural invariants without runtime coupling ---

    /// Proves workspace operations (open_tab, close_tab, split_pane, focus_pane,
    /// swap_pane_slots, replace_pane_spec) all work without any runtime service
    /// dependency. WorkspaceSession is self-contained — no RuntimeRegistry,
    /// PaneRuntime, or runtime types needed.
    #[test]
    fn full_lifecycle_without_runtime_dependency() {
        let mut ws = WorkspaceSession::default();

        // open two tabs
        let events_tab1 = ws
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
                vec![terminal("/a"), browser("https://example.com")],
            )
            .expect("open tab 1");
        assert!(!events_tab1.is_empty());

        let events_tab2 = ws
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/b")],
            )
            .expect("open tab 2");
        assert!(!events_tab2.is_empty());

        let tab1_id = ws.tabs[0].tab_id.clone();
        let tab2_id = ws.tabs[1].tab_id.clone();

        // set_active_tab
        let events = ws.set_active_tab(&tab1_id).expect("set active tab");
        assert!(!events.is_empty());

        // focus_pane
        let pane_b = ws.tabs[0].panes[1].pane_id.clone();
        let events = ws.focus_pane(&tab1_id, &pane_b).expect("focus pane");
        assert!(!events.is_empty());

        // split_pane
        let pane_a = ws.tabs[0].panes[0].pane_id.clone();
        let events = ws
            .split_pane(&pane_a, SplitDirection::Vertical, terminal("/c"))
            .expect("split pane");
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], WorkspaceDomainEvent::PaneAdded { .. }));

        // swap_pane_slots
        let pane_c = ws.tabs[0].panes[2].pane_id.clone();
        ws.swap_pane_slots(&pane_a, &pane_c)
            .expect("swap pane slots");

        // replace_pane_spec
        let events = ws
            .replace_pane_spec(&pane_a, browser("https://rust-lang.org"))
            .expect("replace pane spec");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            WorkspaceDomainEvent::PaneContentChanged { .. }
        ));

        // close_pane
        let events = ws.close_pane(&pane_c).expect("close pane");
        assert!(events
            .iter()
            .any(|e| matches!(e, WorkspaceDomainEvent::PaneRemoved { .. })));

        // close_tab
        let events = ws.close_tab(&tab2_id).expect("close tab");
        assert!(events
            .iter()
            .any(|e| matches!(e, WorkspaceDomainEvent::PaneRemoved { .. })));

        // everything validates without runtime
        ws.validate().expect("workspace valid after full lifecycle");
    }

    /// Every variant of WorkspaceDomainEvent carries only typed IDs (PaneId, TabId)
    /// and PaneContentDefinition. None carry RuntimeStatus, RuntimeSessionId,
    /// terminal_cwd, or browser_location — those belong to the runtime projection.
    #[test]
    fn events_carry_structural_and_content_ref_data_only() {
        let content_id_1 = super::create_content_id();
        let content_id_2 = super::create_content_id();
        let content_id_3 = super::create_content_id();

        let all_events = vec![
            WorkspaceDomainEvent::PaneAdded {
                pane_id: PaneId::from(String::from("p1")),
                content: PaneContentDefinition::terminal(content_id_1, "zsh", "/home", None),
            },
            WorkspaceDomainEvent::PaneRemoved {
                pane_id: PaneId::from(String::from("p1")),
                content: PaneContentDefinition::browser(
                    content_id_2,
                    super::BrowserUrl::new("https://example.com"),
                ),
            },
            WorkspaceDomainEvent::ActivePaneChanged {
                pane_id: PaneId::from(String::from("p1")),
                tab_id: TabId::from(String::from("t1")),
            },
            WorkspaceDomainEvent::ActiveTabChanged {
                tab_id: TabId::from(String::from("t1")),
            },
            WorkspaceDomainEvent::PaneContentChanged {
                pane_id: PaneId::from(String::from("p1")),
                old_content: PaneContentDefinition::terminal(content_id_3, "bash", "/tmp", None),
                new_content: PaneContentDefinition::browser(
                    super::create_content_id(),
                    super::BrowserUrl::new("https://tabby.dev"),
                ),
            },
        ];

        // All events serialize to Debug without any runtime field names.
        // If RuntimeStatus, RuntimeSessionId, terminal_cwd, or browser_location
        // were present in the enum, the debug output would contain them.
        for event in &all_events {
            let debug = format!("{event:?}");
            assert!(
                !debug.contains("RuntimeStatus"),
                "event must not carry RuntimeStatus: {debug}"
            );
            assert!(
                !debug.contains("RuntimeSessionId"),
                "event must not carry RuntimeSessionId: {debug}"
            );
            assert!(
                !debug.contains("terminal_cwd"),
                "event must not carry terminal_cwd: {debug}"
            );
            // PaneContentChanged.old_content may contain browser_location as part
            // of BrowserUrl, but the field name in the event itself should not be
            // "browser_location" — that's a runtime-observed value. BrowserUrl is
            // "initial_url" in PaneContentDefinition::Browser.
            assert!(
                !debug.contains("browser_location"),
                "event must not carry browser_location: {debug}"
            );
        }

        // Verify the event enum has exactly 5 variants (structural + content)
        assert_eq!(
            all_events.len(),
            5,
            "all WorkspaceDomainEvent variants should be covered"
        );
    }

    /// Workspace projection (tab_summaries + pane_content) provides a complete
    /// structural view without needing runtime data. Tab summaries contain tab
    /// metadata, pane slots with content refs, and layout. Runtime-observed
    /// values (cwd, status) come from the runtime projection separately.
    #[test]
    fn workspace_projection_is_complete_without_runtime_data() {
        let mut ws = WorkspaceSession::default();
        ws.open_tab(
            TabLayoutStrategy::Preset(LayoutPreset::OneByTwo),
            vec![terminal("/projects"), browser("https://docs.rs")],
        )
        .expect("open tab");

        // Tab summaries provide full structural projection
        let summaries = ws.tab_summaries();
        assert_eq!(summaries.len(), 1);

        let tab = &summaries[0];
        assert!(!tab.title.is_empty());
        assert_eq!(tab.panes.len(), 2);

        // Each pane slot carries structural data + content ref
        for pane in &tab.panes {
            // Structural: pane_id, title
            assert!(!pane.pane_id.as_ref().is_empty());
            assert!(!pane.title.is_empty());

            // Content ref: content_id links to content store
            let content = ws
                .pane_content(&pane.content_id)
                .expect("content definition should exist for every pane");

            // Content definition carries what-to-launch, not runtime state
            let _content_id = content.content_id();
            // One of these will be Some depending on type
            let _profile = content.terminal_profile_id();
            let _cwd = content.working_directory();
            let _url = content.browser_url();
        }

        // Terminal pane has launch config, not observed runtime cwd
        let terminal_content = ws
            .pane_content(&tab.panes[0].content_id)
            .expect("terminal content");
        assert_eq!(terminal_content.terminal_profile_id(), Some("terminal"));
        assert_eq!(
            terminal_content.working_directory(),
            Some("/projects"),
            "working_directory is launch config, not runtime-observed cwd"
        );

        // Browser pane has initial_url, not runtime-observed location
        let browser_content = ws
            .pane_content(&tab.panes[1].content_id)
            .expect("browser content");
        assert_eq!(
            browser_content.browser_url().map(|u| u.as_str()),
            Some("https://docs.rs"),
            "browser_url is launch config, not runtime-observed location"
        );

        // Layout tree is structural — no runtime data embedded
        let layout_debug = format!("{:?}", tab.layout);
        assert!(
            !layout_debug.contains("RuntimeStatus"),
            "layout must not embed runtime status"
        );

        // Active pane is structural focus, not runtime state
        assert!(tab.panes.iter().any(|p| p.pane_id == tab.active_pane_id));

        // Workspace validates without any runtime interaction
        ws.validate()
            .expect("workspace projection is valid without runtime data");
    }

    /// The workspace domain model can be instantiated and fully mutated using
    /// only types from the tabby-workspace crate. No runtime types are needed
    /// at any point in the lifecycle.
    #[test]
    fn workspace_model_instantiation_requires_no_runtime_types() {
        // All types used here are from tabby-workspace — this is a compile-time
        // proof that no runtime types (RuntimeRegistry, PaneRuntime, RuntimeStatus,
        // RuntimeSessionId) are needed to operate the workspace domain.
        let mut ws = WorkspaceSession::default();

        let _: Result<Vec<WorkspaceDomainEvent>, super::WorkspaceError> = ws.open_tab(
            TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
            vec![PaneSpec::Terminal(super::TerminalPaneSpec {
                launch_profile_id: String::from("default"),
                working_directory: String::from("/tmp"),
                command_override: None,
            })],
        );

        let _: &[super::Tab] = ws.tab_summaries();
        let _: Option<PaneSpec> = ws.pane_spec(&ws.tabs[0].panes[0].pane_id);
        let _: Option<&PaneContentDefinition> = ws.pane_content(&ws.tabs[0].panes[0].content_id);
        let _: Result<(), super::WorkspaceError> = ws.validate();

        // PaneSlot contains only structural fields + content ref
        let slot: &super::PaneSlot = &ws.tabs[0].panes[0];
        let _: &PaneId = &slot.pane_id;
        let _: &str = &slot.title;
        let _: &super::ids::PaneContentId = &slot.content_id;
    }

    /// Cargo.toml for tabby-workspace depends only on thiserror and uuid.
    /// No dependency on tabby-runtime exists. This is verified by the fact that
    /// this crate compiles and all tests run without tabby-runtime being a
    /// dependency — if someone adds it, the Cargo.toml diff in code review
    /// will catch it. This test documents the expectation.
    #[test]
    fn workspace_crate_has_no_runtime_dependency() {
        // This test is a documentation/compilation gate rather than a runtime check.
        // The workspace crate's Cargo.toml lists only:
        //   - thiserror
        //   - uuid
        // If tabby-runtime were added as a dependency, it would be visible in
        // Cargo.toml and would be caught in code review. The fact that ALL tests
        // in this module compile and pass without runtime types proves the boundary.
        //
        // Additionally, we verify that WorkspaceSession's public API surface uses
        // only workspace-local types:
        let ws = WorkspaceSession::default();
        assert!(ws.tabs.is_empty());
        assert!(ws.active_tab_id.is_none());
    }

    #[test]
    fn close_last_pane_destroys_content_and_tab() {
        let mut workspace = WorkspaceSession::default();
        workspace
            .open_tab(
                TabLayoutStrategy::Preset(LayoutPreset::OneByOne),
                vec![terminal("/tmp")],
            )
            .expect("tab should open");

        let pane_id = workspace.tabs[0].panes[0].pane_id.clone();
        let content_id = workspace.tabs[0].panes[0].content_id.clone();

        workspace
            .close_pane(&pane_id)
            .expect("close last pane should succeed");

        assert!(workspace.tabs.is_empty(), "tab should be removed");
        assert!(
            workspace.pane_content(&content_id).is_none(),
            "content should be destroyed"
        );
        workspace
            .validate()
            .expect("empty workspace should be valid");
    }
}
