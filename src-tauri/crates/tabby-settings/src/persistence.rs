//! Persistence schema for user preferences.
//!
//! This module owns the on-disk serialization format, independent of the IPC
//! DTOs used by the transport layer. The two schemas can evolve separately:
//! adding a field here does not affect the frontend contract and vice versa.

use serde::{Deserialize, Serialize};

use tabby_kernel::LayoutPreset;

use crate::{FontSize, ProfileId, SettingsError, UserPreferences, WorkingDirectory};

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
    pub theme: String,
    pub launch_fullscreen: bool,
    pub has_completed_onboarding: bool,
    #[serde(default)]
    pub last_working_directory: Option<String>,
}

// ---------------------------------------------------------------------------
// Domain ↔ Persistence conversions
// ---------------------------------------------------------------------------

impl PersistedPreferences {
    pub fn from_domain(preferences: &UserPreferences) -> Self {
        Self {
            default_layout: String::from(preferences.default_layout.as_str()),
            default_terminal_profile_id: preferences
                .default_terminal_profile_id
                .as_str()
                .to_string(),
            default_working_directory: preferences.default_working_directory.as_str().to_string(),
            default_custom_command: preferences.default_custom_command.clone(),
            font_size: preferences.font_size.value(),
            theme: preferences.theme.clone(),
            launch_fullscreen: preferences.launch_fullscreen,
            has_completed_onboarding: preferences.has_completed_onboarding,
            last_working_directory: preferences.last_working_directory.clone(),
        }
    }

    pub fn to_domain(&self) -> Result<UserPreferences, SettingsError> {
        let default_layout = LayoutPreset::parse(&self.default_layout).unwrap_or_default();
        Ok(UserPreferences {
            default_layout,
            default_terminal_profile_id: ProfileId::new(self.default_terminal_profile_id.clone()),
            default_working_directory: WorkingDirectory::new(
                self.default_working_directory.clone(),
            )?,
            default_custom_command: self.default_custom_command.clone(),
            font_size: FontSize::new(self.font_size)?,
            theme: self.theme.clone(),
            launch_fullscreen: self.launch_fullscreen,
            has_completed_onboarding: self.has_completed_onboarding,
            last_working_directory: self.last_working_directory.clone(),
        })
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
            default_layout: LayoutPreset::TwoByTwo,
            default_terminal_profile_id: ProfileId::new("claude"),
            default_working_directory: WorkingDirectory::new("/tmp").expect("valid path"),
            default_custom_command: String::from("fish"),
            font_size: FontSize::new(16).expect("valid size"),
            theme: String::from("dawn"),
            launch_fullscreen: true,
            has_completed_onboarding: true,
            last_working_directory: Some(String::from("/home")),
        };

        let value = serialize_preferences(&preferences).expect("should serialize");
        let restored = deserialize_preferences(value).expect("should deserialize");

        assert_eq!(restored.default_layout, LayoutPreset::TwoByTwo);
        assert_eq!(restored.default_terminal_profile_id, "claude");
        assert_eq!(restored.default_working_directory.as_str(), "/tmp");
        assert_eq!(restored.default_custom_command, "fish");
        assert_eq!(restored.font_size.value(), 16);
        assert_eq!(restored.theme, "dawn");
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

        assert_eq!(restored.default_layout, LayoutPreset::OneByTwo);
        assert_eq!(restored.default_terminal_profile_id, "claude");
        assert_eq!(restored.font_size.value(), 18);
        assert_eq!(restored.theme, "midnight");
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
    fn backward_compatible_unknown_layout_falls_back_to_default() {
        let json_with_unknown_layout = json!({
            "defaultLayout": "4x4",
            "defaultTerminalProfileId": "terminal",
            "defaultWorkingDirectory": "~",
            "defaultCustomCommand": "",
            "fontSize": 13,
            "theme": "system",
            "launchFullscreen": true,
            "hasCompletedOnboarding": false
        });

        let restored = deserialize_preferences(json_with_unknown_layout)
            .expect("unknown layout should fall back to default");

        assert_eq!(restored.default_layout, LayoutPreset::default());
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

    // -----------------------------------------------------------------------
    // PersistedPreferences::from_domain field mapping
    // -----------------------------------------------------------------------

    #[test]
    fn from_domain_maps_all_fields_correctly() {
        let prefs = UserPreferences {
            default_layout: LayoutPreset::TwoByThree,
            default_terminal_profile_id: ProfileId::new("codex"),
            default_working_directory: WorkingDirectory::new("/work").unwrap(),
            default_custom_command: String::from("fish"),
            font_size: FontSize::new(18).unwrap(),
            theme: String::from("dawn"),
            launch_fullscreen: false,
            has_completed_onboarding: true,
            last_working_directory: Some(String::from("/last")),
        };

        let persisted = PersistedPreferences::from_domain(&prefs);

        assert_eq!(persisted.default_layout, "2x3");
        assert_eq!(persisted.default_terminal_profile_id, "codex");
        assert_eq!(persisted.default_working_directory, "/work");
        assert_eq!(persisted.default_custom_command, "fish");
        assert_eq!(persisted.font_size, 18);
        assert_eq!(persisted.theme, "dawn");
        assert!(!persisted.launch_fullscreen);
        assert!(persisted.has_completed_onboarding);
        assert_eq!(persisted.last_working_directory.as_deref(), Some("/last"));
    }

    #[test]
    fn from_domain_maps_empty_working_directory() {
        let prefs = default_preferences();
        let persisted = PersistedPreferences::from_domain(&prefs);
        assert_eq!(persisted.default_working_directory, "");
    }

    #[test]
    fn from_domain_maps_none_last_working_directory() {
        let prefs = default_preferences();
        let persisted = PersistedPreferences::from_domain(&prefs);
        assert!(persisted.last_working_directory.is_none());
    }

    #[test]
    fn from_domain_maps_all_layout_variants() {
        let layouts = [
            (LayoutPreset::OneByOne, "1x1"),
            (LayoutPreset::OneByTwo, "1x2"),
            (LayoutPreset::TwoByTwo, "2x2"),
            (LayoutPreset::TwoByThree, "2x3"),
            (LayoutPreset::ThreeByThree, "3x3"),
        ];
        for (layout, expected_str) in layouts {
            let prefs = UserPreferences {
                default_layout: layout,
                ..default_preferences()
            };
            let persisted = PersistedPreferences::from_domain(&prefs);
            assert_eq!(persisted.default_layout, expected_str);
        }
    }

    // -----------------------------------------------------------------------
    // to_domain validation errors
    // -----------------------------------------------------------------------

    #[test]
    fn to_domain_rejects_invalid_font_size_zero() {
        let persisted = PersistedPreferences {
            default_layout: String::from("1x1"),
            default_terminal_profile_id: String::from("terminal"),
            default_working_directory: String::from("~"),
            default_custom_command: String::new(),
            font_size: 0,
            theme: String::from("system"),
            launch_fullscreen: false,
            has_completed_onboarding: false,
            last_working_directory: None,
        };
        let err = persisted.to_domain().unwrap_err();
        assert!(err.to_string().contains("Font size"));
    }

    #[test]
    fn to_domain_rejects_invalid_working_directory_with_null_byte() {
        let persisted = PersistedPreferences {
            default_layout: String::from("1x1"),
            default_terminal_profile_id: String::from("terminal"),
            default_working_directory: String::from("/bad/\0path"),
            default_custom_command: String::new(),
            font_size: 13,
            theme: String::from("system"),
            launch_fullscreen: false,
            has_completed_onboarding: false,
            last_working_directory: None,
        };
        let err = persisted.to_domain().unwrap_err();
        assert!(err.to_string().contains("null bytes"));
    }

    #[test]
    fn to_domain_accepts_unknown_layout_falling_back_to_default() {
        let persisted = PersistedPreferences {
            default_layout: String::from("99x99"),
            default_terminal_profile_id: String::from("terminal"),
            default_working_directory: String::new(),
            default_custom_command: String::new(),
            font_size: 13,
            theme: String::from("system"),
            launch_fullscreen: false,
            has_completed_onboarding: false,
            last_working_directory: None,
        };
        let domain = persisted.to_domain().unwrap();
        assert_eq!(domain.default_layout, LayoutPreset::default());
    }

    // -----------------------------------------------------------------------
    // round-trip with all layout variants
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_one_by_two_layout() {
        let prefs = UserPreferences {
            default_layout: LayoutPreset::OneByTwo,
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert_eq!(restored.default_layout, LayoutPreset::OneByTwo);
    }

    #[test]
    fn round_trip_two_by_three_layout() {
        let prefs = UserPreferences {
            default_layout: LayoutPreset::TwoByThree,
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert_eq!(restored.default_layout, LayoutPreset::TwoByThree);
    }

    #[test]
    fn round_trip_three_by_three_layout() {
        let prefs = UserPreferences {
            default_layout: LayoutPreset::ThreeByThree,
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert_eq!(restored.default_layout, LayoutPreset::ThreeByThree);
    }

    #[test]
    fn round_trip_min_font_size() {
        let prefs = UserPreferences {
            font_size: FontSize::new(FontSize::MIN).unwrap(),
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert_eq!(restored.font_size.value(), FontSize::MIN);
    }

    #[test]
    fn round_trip_max_font_size() {
        let prefs = UserPreferences {
            font_size: FontSize::new(FontSize::MAX).unwrap(),
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert_eq!(restored.font_size.value(), FontSize::MAX);
    }

    #[test]
    fn round_trip_no_last_working_directory() {
        let prefs = UserPreferences {
            last_working_directory: None,
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert!(restored.last_working_directory.is_none());
    }

    #[test]
    fn round_trip_with_last_working_directory() {
        let prefs = UserPreferences {
            last_working_directory: Some(String::from("/projects/tabby")),
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert_eq!(
            restored.last_working_directory.as_deref(),
            Some("/projects/tabby")
        );
    }

    #[test]
    fn round_trip_empty_custom_command() {
        let prefs = UserPreferences {
            default_custom_command: String::new(),
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert_eq!(restored.default_custom_command, "");
    }

    #[test]
    fn round_trip_non_empty_custom_command() {
        let prefs = UserPreferences {
            default_terminal_profile_id: ProfileId::new("custom"),
            default_custom_command: String::from("vim --noplugin"),
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert_eq!(restored.default_custom_command, "vim --noplugin");
    }

    #[test]
    fn round_trip_special_characters_in_theme() {
        let prefs = UserPreferences {
            theme: String::from("my theme/variant (dark)"),
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert_eq!(restored.theme, "my theme/variant (dark)");
    }

    #[test]
    fn round_trip_launch_fullscreen_false() {
        let prefs = UserPreferences {
            launch_fullscreen: false,
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert!(!restored.launch_fullscreen);
    }

    #[test]
    fn round_trip_has_completed_onboarding_true() {
        let prefs = UserPreferences {
            has_completed_onboarding: true,
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        let restored = deserialize_preferences(value).unwrap();
        assert!(restored.has_completed_onboarding);
    }

    // -----------------------------------------------------------------------
    // serialized field values
    // -----------------------------------------------------------------------

    #[test]
    fn serialize_produces_correct_font_size_value() {
        let prefs = UserPreferences {
            font_size: FontSize::new(20).unwrap(),
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        assert_eq!(value["fontSize"], 20);
    }

    #[test]
    fn serialize_produces_correct_layout_string() {
        let prefs = UserPreferences {
            default_layout: LayoutPreset::TwoByTwo,
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        assert_eq!(value["defaultLayout"], "2x2");
    }

    #[test]
    fn serialize_produces_correct_profile_id_string() {
        let prefs = UserPreferences {
            default_terminal_profile_id: ProfileId::new("claude"),
            ..default_preferences()
        };
        let value = serialize_preferences(&prefs).unwrap();
        assert_eq!(value["defaultTerminalProfileId"], "claude");
    }

    #[test]
    fn deserialize_rejects_number_instead_of_object() {
        let result = deserialize_preferences(json!(42));
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_rejects_array() {
        let result = deserialize_preferences(json!([]));
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_rejects_boolean() {
        let result = deserialize_preferences(json!(true));
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_rejects_null() {
        let result = deserialize_preferences(serde_json::Value::Null);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // PersistedPreferences Clone and Debug
    // -----------------------------------------------------------------------

    #[test]
    fn persisted_preferences_clone_is_equal() {
        let prefs = default_preferences();
        let persisted = PersistedPreferences::from_domain(&prefs);
        let cloned = persisted.clone();
        assert_eq!(persisted.font_size, cloned.font_size);
        assert_eq!(persisted.theme, cloned.theme);
        assert_eq!(persisted.default_layout, cloned.default_layout);
    }

    #[test]
    fn persisted_preferences_debug_format() {
        let prefs = default_preferences();
        let persisted = PersistedPreferences::from_domain(&prefs);
        let debug = format!("{persisted:?}");
        assert!(debug.contains("system")); // theme
    }
}
