use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::TabbyError;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeMode {
    #[serde(rename = "system")]
    #[default]
    System,
    #[serde(rename = "dawn")]
    Dawn,
    #[serde(rename = "midnight")]
    Midnight,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayoutPreset {
    #[serde(rename = "1x1")]
    OneByOne,
    #[serde(rename = "1x2")]
    OneByTwo,
    #[serde(rename = "2x2")]
    #[default]
    TwoByTwo,
    #[serde(rename = "2x3")]
    TwoByThree,
    #[serde(rename = "3x3")]
    ThreeByThree,
}

impl LayoutPreset {
    pub fn dimensions(self) -> (usize, usize) {
        match self {
            Self::OneByOne => (1, 1),
            Self::OneByTwo => (1, 2),
            Self::TwoByTwo => (2, 2),
            Self::TwoByThree => (2, 3),
            Self::ThreeByThree => (3, 3),
        }
    }

    pub fn pane_count(self) -> usize {
        let (rows, columns) = self.dimensions();
        rows * columns
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GridDefinition {
    pub preset: LayoutPreset,
    pub rows: usize,
    pub columns: usize,
    pub pane_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaneProfile {
    pub id: String,
    pub label: String,
    pub description: String,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_layout: LayoutPreset,
    pub default_profile_id: String,
    pub default_working_directory: String,
    pub default_custom_command: String,
    pub font_size: u16,
    pub theme: ThemeMode,
    pub launch_fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaneSnapshot {
    pub id: String,
    pub session_id: String,
    pub title: String,
    pub cwd: String,
    pub profile_id: String,
    pub profile_label: String,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TabSnapshot {
    pub id: String,
    pub title: String,
    pub preset: LayoutPreset,
    pub panes: Vec<PaneSnapshot>,
    pub active_pane_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub active_tab_id: String,
    pub tabs: Vec<TabSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapSnapshot {
    pub workspace: WorkspaceSnapshot,
    pub settings: AppSettings,
    pub profiles: Vec<PaneProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NewTabRequest {
    pub preset: LayoutPreset,
    pub cwd: Option<String>,
    pub profile_id: Option<String>,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePaneProfileRequest {
    pub pane_id: String,
    pub profile_id: String,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePaneCwdRequest {
    pub pane_id: String,
    pub cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PtyResizeRequest {
    pub pane_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PtyOutputEvent {
    pub pane_id: String,
    pub session_id: String,
    pub chunk: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    pub id: String,
    pub label: String,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PaneSeed {
    pub pane_id: String,
    pub session_id: String,
    pub cwd: String,
    pub profile_id: String,
    pub profile_label: String,
    pub startup_command: Option<String>,
}

pub fn built_in_profiles() -> Vec<PaneProfile> {
    vec![
        PaneProfile {
            id: String::from("terminal"),
            label: String::from("Terminal"),
            description: String::from("Pure login shell"),
            startup_command: None,
        },
        PaneProfile {
            id: String::from("claude"),
            label: String::from("Claude Code"),
            description: String::from("Open Claude Code in a fresh shell"),
            startup_command: Some(String::from("claude")),
        },
        PaneProfile {
            id: String::from("codex"),
            label: String::from("Codex"),
            description: String::from("Open Codex in a fresh shell"),
            startup_command: Some(String::from("codex")),
        },
        PaneProfile {
            id: String::from("custom"),
            label: String::from("Custom"),
            description: String::from("Run an arbitrary shell command"),
            startup_command: None,
        },
    ]
}

pub fn resolve_profile(
    profile_id: &str,
    startup_command: Option<String>,
) -> Result<ResolvedProfile, TabbyError> {
    let profile = built_in_profiles()
        .into_iter()
        .find(|candidate| candidate.id == profile_id)
        .ok_or_else(|| TabbyError::Validation(format!("Unknown profile: {profile_id}")))?;

    if profile.id == "custom" {
        let command = startup_command
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                TabbyError::Validation(String::from("Custom profile requires a startup command"))
            })?;

        return Ok(ResolvedProfile {
            id: profile.id,
            label: profile.label,
            startup_command: Some(command),
        });
    }

    Ok(ResolvedProfile {
        id: profile.id,
        label: profile.label,
        startup_command: profile.startup_command,
    })
}

pub fn default_settings(default_working_directory: String) -> AppSettings {
    AppSettings {
        default_layout: LayoutPreset::TwoByTwo,
        default_profile_id: String::from("terminal"),
        default_working_directory,
        default_custom_command: String::new(),
        font_size: 13,
        theme: ThemeMode::System,
        launch_fullscreen: true,
    }
}

pub fn create_pane_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn create_tab_id() -> String {
    Uuid::new_v4().to_string()
}
