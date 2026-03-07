use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::layout::LayoutPreset;

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
    #[serde(default)]
    pub last_working_directory: Option<String>,
}

pub fn default_settings() -> AppSettings {
    AppSettings {
        default_layout: LayoutPreset::OneByOne,
        default_profile_id: String::new(),
        default_working_directory: String::new(),
        default_custom_command: String::new(),
        font_size: 13,
        theme: ThemeMode::System,
        launch_fullscreen: true,
        has_completed_onboarding: false,
        last_working_directory: None,
    }
}
