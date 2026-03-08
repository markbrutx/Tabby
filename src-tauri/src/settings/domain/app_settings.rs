use serde::{Deserialize, Serialize};
use specta::Type;

use crate::settings::domain::profiles::{
    is_known_profile_id, CUSTOM_PROFILE_ID, TERMINAL_PROFILE_ID,
};
use crate::workspace::domain::layout::LayoutPreset;

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
        default_profile_id: String::from(TERMINAL_PROFILE_ID),
        default_working_directory: String::new(),
        default_custom_command: String::new(),
        font_size: 13,
        theme: ThemeMode::System,
        launch_fullscreen: true,
        has_completed_onboarding: false,
        last_working_directory: None,
    }
}

pub fn normalize_settings(mut settings: AppSettings) -> AppSettings {
    let normalized_profile = settings.default_profile_id.trim();
    let invalid_custom = normalized_profile == CUSTOM_PROFILE_ID
        && settings.default_custom_command.trim().is_empty();

    if normalized_profile.is_empty() || !is_known_profile_id(normalized_profile) || invalid_custom {
        settings.default_profile_id = String::from(TERMINAL_PROFILE_ID);
    } else if normalized_profile != settings.default_profile_id {
        settings.default_profile_id = normalized_profile.to_string();
    }

    settings
}

#[cfg(test)]
mod tests {
    use super::{default_settings, normalize_settings};
    use crate::settings::domain::profiles::{CUSTOM_PROFILE_ID, TERMINAL_PROFILE_ID};

    #[test]
    fn defaults_use_terminal_profile() {
        assert_eq!(default_settings().default_profile_id, TERMINAL_PROFILE_ID);
    }

    #[test]
    fn normalize_settings_replaces_empty_or_unknown_default_profile() {
        let empty = normalize_settings(super::AppSettings {
            default_profile_id: String::new(),
            ..default_settings()
        });
        assert_eq!(empty.default_profile_id, TERMINAL_PROFILE_ID);

        let unknown = normalize_settings(super::AppSettings {
            default_profile_id: String::from("unknown"),
            ..default_settings()
        });
        assert_eq!(unknown.default_profile_id, TERMINAL_PROFILE_ID);
    }

    #[test]
    fn normalize_settings_replaces_invalid_custom_default_profile() {
        let normalized = normalize_settings(super::AppSettings {
            default_profile_id: String::from(CUSTOM_PROFILE_ID),
            default_custom_command: String::from("   "),
            ..default_settings()
        });

        assert_eq!(normalized.default_profile_id, TERMINAL_PROFILE_ID);
    }

    #[test]
    fn normalize_settings_trims_valid_default_profile() {
        let normalized = normalize_settings(super::AppSettings {
            default_profile_id: String::from(" terminal "),
            ..default_settings()
        });

        assert_eq!(normalized.default_profile_id, TERMINAL_PROFILE_ID);
    }
}
