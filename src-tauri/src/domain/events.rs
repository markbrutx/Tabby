use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::snapshot::{PaneRuntimeStatus, WorkspaceSnapshot};

pub const PTY_OUTPUT_EVENT_NAME: &str = "pty-output";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct PtyOutputEvent {
    pub pane_id: String,
    pub session_id: String,
    pub chunk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct PaneLifecycleEvent {
    pub pane_id: String,
    pub session_id: Option<String>,
    pub status: PaneRuntimeStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceChangedEvent {
    pub workspace: WorkspaceSnapshot,
}
