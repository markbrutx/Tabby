use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::error::TabbyError;
use crate::domain::types::{AppSettings, PaneKind, PaneProfile, PaneSeed, SplitNode};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub enum PaneRuntimeStatus {
    #[default]
    Starting,
    Running,
    Restarting,
    Exited,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct PaneSnapshot {
    pub id: String,
    pub session_id: String,
    pub title: String,
    pub cwd: String,
    pub profile_id: String,
    pub profile_label: String,
    pub startup_command: Option<String>,
    pub status: PaneRuntimeStatus,
    #[serde(default)]
    pub pane_kind: PaneKind,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct TabSnapshot {
    pub id: String,
    pub title: String,
    pub layout: SplitNode,
    pub panes: Vec<PaneSnapshot>,
    pub active_pane_id: String,
}

impl TabSnapshot {
    pub fn from_seeds(
        id: String,
        title: String,
        layout: SplitNode,
        pane_seeds: Vec<PaneSeed>,
        status: PaneRuntimeStatus,
    ) -> Result<Self, TabbyError> {
        if pane_seeds.is_empty() {
            return Err(TabbyError::Validation(String::from(
                "Cannot create a tab without panes",
            )));
        }

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
                status,
                pane_kind: seed.pane_kind,
                url: seed.url,
            })
            .collect::<Vec<_>>();

        let active_pane_id = panes
            .first()
            .map(|pane| pane.id.clone())
            .ok_or_else(|| TabbyError::State(String::from("New tab has no active pane")))?;

        Ok(Self {
            id,
            title,
            layout,
            panes,
            active_pane_id,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub active_tab_id: String,
    pub tabs: Vec<TabSnapshot>,
}

impl WorkspaceSnapshot {
    pub fn new(active_tab_id: Option<String>, tabs: Vec<TabSnapshot>) -> Self {
        Self {
            active_tab_id: active_tab_id.unwrap_or_default(),
            tabs,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapSnapshot {
    pub workspace: WorkspaceSnapshot,
    pub settings: AppSettings,
    pub profiles: Vec<PaneProfile>,
}
