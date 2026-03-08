use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub enum PaneKind {
    #[default]
    Terminal,
    Browser,
}

#[derive(Debug, Clone)]
pub struct PaneSeed {
    pub pane_id: String,
    pub session_id: String,
    pub cwd: String,
    pub profile_id: String,
    pub profile_label: String,
    pub startup_command: Option<String>,
    pub pane_kind: PaneKind,
    pub url: Option<String>,
}

pub fn create_pane_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn create_tab_id() -> String {
    Uuid::new_v4().to_string()
}
