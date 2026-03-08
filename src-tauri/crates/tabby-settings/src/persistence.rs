//! Persistence schema for user preferences.
//!
//! This module owns the on-disk serialization format, independent of the IPC
//! DTOs used by the transport layer. The two schemas can evolve separately:
//! adding a field here does not affect the frontend contract and vice versa.

use serde::{Deserialize, Serialize};

use crate::{FontSize, ProfileId, SettingsError, ThemeMode, UserPreferences, WorkingDirectory};

// ---------------------------------------------------------------------------
// Persistence schema — what gets written to / read from disk
// ---------------------------------------------------------------------------

/// On-disk representation of user preferences.
///
/// Uses `camelCase` for backward compatibility with the original storage
/// format. New fields should include `#[serde(default)]` so that existing
/// files missing the field still deserialize correctly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedPreferences {
    pub default_layout: String,
    pub default_terminal_profile_id: String,
    pub default_working_directory: String,
    pub default_custom_command: String,
    pub font_size: u16,
    pub theme: PersistedThemeMode,
    pub launch_fullscreen: bool,
    pub has_completed_onboarding: bool,
    #[serde(default)]
    pub last_working_directory: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PersistedThemeMode {
    System,
    Dawn,
    Midnight,
}

// ---------------------------------------------------------------------------
// Domain ↔ Persistence conversions
// ---------------------------------------------------------------------------

impl PersistedPreferences {
    pub fn from_domain(preferences: &UserPreferences) -> Self {
        Self {
            default_layout: preferences.default_layout.clone(),
            default_terminal_profile_id: preferences
                .default_terminal_profile_id
                .as_str()
                .to_string(),
            default_working_directory: preferences.default_working_directory.as_str().to_string(),
            default_custom_command: preferences.default_custom_command.clone(),
            font_size: preferences.font_size.value(),
            theme: PersistedThemeMode::from_domain(preferences.theme),
            launch_fullscreen: preferences.launch_fullscreen,
            has_completed_onboarding: preferences.has_completed_onboarding,
            last_working_directory: preferences.last_working_directory.clone(),
        }
    }

    pub fn to_domain(&self) -> Result<UserPreferences, SettingsError> {
        Ok(UserPreferences {
            default_layout: self.default_layout.clone(),
            default_terminal_profile_id: ProfileId::new(self.default_terminal_profile_id.clone()),
            default_working_directory: WorkingDirectory::new(
                self.default_working_directory.clone(),
            )?,
            default_custom_command: self.default_custom_command.clone(),
            font_size: FontSize::new(self.font_size)?,
            theme: self.theme.to_domain(),
            launch_fullscreen: self.launch_fullscreen,
            has_completed_onboarding: self.has_completed_onboarding,
            last_working_directory: self.last_working_directory.clone(),
        })
    }
}

impl PersistedThemeMode {
    fn from_domain(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::System => Self::System,
            ThemeMode::Dawn => Self::Dawn,
            ThemeMode::Midnight => Self::Midnight,
        }
    }

    fn to_domain(self) -> ThemeMode {
        match self {
            Self::System => ThemeMode::System,
            Self::Dawn => ThemeMode::Dawn,
            Self::Midnight => ThemeMode::Midnight,
        }
    }
}

// ---------------------------------------------------------------------------
// Public helpers used by the repository and settings service
// ---------------------------------------------------------------------------

/// Serialize domain preferences into a JSON value for storage.
pub fn serialize_preferences(
    preferences: &UserPreferences,
) -> Result<serde_json::Value, serde_json::Error> {
    let persisted = PersistedPreferences::from_domain(preferences);
    serde_json::to_value(persisted)
}

/// Deserialize a JSON value from storage into domain preferences.
pub fn deserialize_preferences(value: serde_json::Value) -> Result<UserPreferences, SettingsError> {
    let persisted: PersistedPreferences =
        serde_json::from_value(value).map_err(|e| SettingsError::Validation(e.to_string()))?;
    persisted.to_domain()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_preferences;
    use serde_json::json;

    #[test]
    fn round_trip_preserves_all_fields() {
        let preferences = UserPreferences {
            default_layout: String::from("2x2"),
            default_terminal_profile_id: ProfileId::new("claude"),
            default_working_directory: WorkingDirectory::new("/tmp").expect("valid path"),
            default_custom_command: String::from("fish"),
            font_size: FontSize::new(16).expect("valid size"),
            theme: ThemeMode::Dawn,
            launch_fullscreen: true,
            has_completed_onboarding: true,
            last_working_directory: Some(String::from("/home")),
        };

        let value = serialize_preferences(&preferences).expect("should serialize");
        let restored = deserialize_preferences(value).expect("should deserialize");

        assert_eq!(restored.default_layout, "2x2");
        assert_eq!(restored.default_terminal_profile_id, "claude");
        assert_eq!(restored.default_working_directory.as_str(), "/tmp");
        assert_eq!(restored.default_custom_command, "fish");
        assert_eq!(restored.font_size.value(), 16);
        assert!(matches!(restored.theme, ThemeMode::Dawn));
        assert!(restored.launch_fullscreen);
        assert!(restored.has_completed_onboarding);
        assert_eq!(restored.last_working_directory.as_deref(), Some("/home"));
    }

    #[test]
    fn round_trip_with_defaults() {
        let defaults = default_preferences();
        let value = serialize_preferences(&defaults).expect("should serialize");
        let restored = deserialize_preferences(value).expect("should deserialize");

        assert_eq!(restored.default_layout, defaults.default_layout);
        assert_eq!(
            restored.default_terminal_profile_id,
            defaults.default_terminal_profile_id
        );
        assert_eq!(restored.font_size, defaults.font_size);
    }

    #[test]
    fn deserialize_rejects_invalid_font_size() {
        let value = json!({
            "defaultLayout": "1x1",
            "defaultTerminalProfileId": "terminal",
            "defaultWorkingDirectory": "~",
            "defaultCustomCommand": "",
            "fontSize": 200,
            "theme": "system",
            "launchFullscreen": false,
            "hasCompletedOnboarding": false,
            "lastWorkingDirectory": null
        });

        let err = deserialize_preferences(value).expect_err("should reject invalid font size");
        assert!(err.to_string().contains("Font size"));
    }

    #[test]
    fn backward_compatible_with_existing_stored_format() {
        // This JSON matches the format previously produced by SettingsView serialization
        let legacy_json = json!({
            "defaultLayout": "1x2",
            "defaultTerminalProfileId": "claude",
            "defaultWorkingDirectory": "/home",
            "defaultCustomCommand": "",
            "fontSize": 18,
            "theme": "midnight",
            "launchFullscreen": false,
            "hasCompletedOnboarding": true,
            "lastWorkingDirectory": "/var"
        });

        let restored = deserialize_preferences(legacy_json).expect("should load legacy format");

        assert_eq!(restored.default_layout, "1x2");
        assert_eq!(restored.default_terminal_profile_id, "claude");
        assert_eq!(restored.font_size.value(), 18);
        assert!(matches!(restored.theme, ThemeMode::Midnight));
        assert_eq!(restored.last_working_directory.as_deref(), Some("/var"));
    }

    #[test]
    fn backward_compatible_without_last_working_directory() {
        // Old stored data may not have lastWorkingDirectory at all
        let legacy_json = json!({
            "defaultLayout": "1x1",
            "defaultTerminalProfileId": "terminal",
            "defaultWorkingDirectory": "~",
            "defaultCustomCommand": "",
            "fontSize": 13,
            "theme": "system",
            "launchFullscreen": true,
            "hasCompletedOnboarding": false
        });

        let restored =
            deserialize_preferences(legacy_json).expect("should load without lastWorkingDirectory");

        assert!(restored.last_working_directory.is_none());
    }

    #[test]
    fn deserialize_rejects_malformed_json() {
        let result = deserialize_preferences(json!("just a string"));
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_rejects_empty_object() {
        let result = deserialize_preferences(json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn serialized_format_uses_camel_case() {
        let preferences = default_preferences();
        let value = serialize_preferences(&preferences).expect("should serialize");
        let obj = value.as_object().expect("should be object");

        assert!(obj.contains_key("defaultLayout"));
        assert!(obj.contains_key("defaultTerminalProfileId"));
        assert!(obj.contains_key("fontSize"));
        assert!(obj.contains_key("launchFullscreen"));
        assert!(obj.contains_key("hasCompletedOnboarding"));
        // Ensure snake_case is NOT used
        assert!(!obj.contains_key("default_layout"));
        assert!(!obj.contains_key("font_size"));
    }
}
