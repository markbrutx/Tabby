use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::domain::error::TabbyError;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Type)]
pub enum ThemeMode {
    #[serde(rename = "system")]
    #[default]
    System,
    #[serde(rename = "dawn")]
    Dawn,
    #[serde(rename = "midnight")]
    Midnight,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Type)]
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
    pub fn dimensions(self) -> (u16, u16) {
        match self {
            Self::OneByOne => (1, 1),
            Self::OneByTwo => (1, 2),
            Self::TwoByTwo => (2, 2),
            Self::TwoByThree => (2, 3),
            Self::ThreeByThree => (3, 3),
        }
    }

    pub fn pane_count(self) -> u16 {
        let (rows, columns) = self.dimensions();
        rows * columns
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct GridDefinition {
    pub preset: LayoutPreset,
    pub rows: u16,
    pub columns: u16,
    pub pane_count: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct PaneProfile {
    pub id: String,
    pub label: String,
    pub description: String,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_layout: LayoutPreset,
    pub default_profile_id: String,
    pub default_working_directory: String,
    pub default_custom_command: String,
    pub font_size: u16,
    pub theme: ThemeMode,
    pub launch_fullscreen: bool,
    #[serde(default)]
    pub has_completed_onboarding: bool,
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
        has_completed_onboarding: false,
    }
}

pub fn create_pane_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn create_tab_id() -> String {
    Uuid::new_v4().to_string()
}
