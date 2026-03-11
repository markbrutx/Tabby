pub mod persistence;
mod value_objects;

pub use value_objects::{FontSize, ProfileId, WorkingDirectory};

use tabby_kernel::{CommandTemplate, LayoutPreset};
use thiserror::Error;

pub const CUSTOM_PROFILE_ID: &str = "custom";
pub const TERMINAL_PROFILE_ID: &str = "terminal";
pub const CLAUDE_PROFILE_ID: &str = "claude";
pub const CODEX_PROFILE_ID: &str = "codex";
pub const GEMINI_PROFILE_ID: &str = "gemini";
pub const OPENCODE_PROFILE_ID: &str = "opencode";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalProfile {
    pub id: ProfileId,
    pub label: String,
    pub description: String,
    pub startup_command_template: Option<CommandTemplate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileCatalog {
    pub terminal_profiles: Vec<TerminalProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserPreferences {
    pub default_layout: LayoutPreset,
    pub default_terminal_profile_id: ProfileId,
    pub default_working_directory: WorkingDirectory,
    pub default_custom_command: String,
    pub font_size: FontSize,
    pub theme: String,
    pub launch_fullscreen: bool,
    pub has_completed_onboarding: bool,
    pub last_working_directory: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTerminalProfile {
    pub id: ProfileId,
    pub label: String,
    pub command: Option<CommandTemplate>,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("validation error: {0}")]
    Validation(String),
}

impl From<tabby_kernel::ValueObjectError> for SettingsError {
    fn from(err: tabby_kernel::ValueObjectError) -> Self {
        Self::Validation(err.to_string())
    }
}

pub fn default_preferences() -> UserPreferences {
    UserPreferences {
        default_layout: LayoutPreset::default(),
        default_terminal_profile_id: ProfileId::new(TERMINAL_PROFILE_ID),
        default_working_directory: WorkingDirectory::empty(),
        default_custom_command: String::new(),
        font_size: FontSize::default(),
        theme: String::from("system"),
        launch_fullscreen: true,
        has_completed_onboarding: false,
        last_working_directory: None,
    }
}

pub fn built_in_profile_catalog() -> ProfileCatalog {
    ProfileCatalog {
        terminal_profiles: vec![
            TerminalProfile {
                id: ProfileId::new(TERMINAL_PROFILE_ID),
                label: String::from("Terminal"),
                description: String::from("Standard shell session"),
                startup_command_template: None,
            },
            TerminalProfile {
                id: ProfileId::new(CLAUDE_PROFILE_ID),
                label: String::from("Claude Code"),
                description: String::from("Anthropic coding assistant"),
                startup_command_template: Some(CommandTemplate::new("claude")),
            },
            TerminalProfile {
                id: ProfileId::new(CODEX_PROFILE_ID),
                label: String::from("Codex"),
                description: String::from("OpenAI coding agent"),
                startup_command_template: Some(CommandTemplate::new("codex")),
            },
            TerminalProfile {
                id: ProfileId::new(GEMINI_PROFILE_ID),
                label: String::from("Gemini CLI"),
                description: String::from("Google Gemini coding agent"),
                startup_command_template: Some(CommandTemplate::new("gemini")),
            },
            TerminalProfile {
                id: ProfileId::new(OPENCODE_PROFILE_ID),
                label: String::from("OpenCode CLI"),
                description: String::from("OpenCode coding agent"),
                startup_command_template: Some(CommandTemplate::new("opencode")),
            },
            TerminalProfile {
                id: ProfileId::new(CUSTOM_PROFILE_ID),
                label: String::from("Custom"),
                description: String::from("Run any command"),
                startup_command_template: None,
            },
        ],
    }
}

pub fn normalize_preferences(mut preferences: UserPreferences) -> UserPreferences {
    let catalog = built_in_profile_catalog();
    let profile_id = preferences.default_terminal_profile_id.as_str().trim();

    if profile_id.is_empty()
        || !catalog
            .terminal_profiles
            .iter()
            .any(|profile| profile.id == profile_id)
        || (profile_id == CUSTOM_PROFILE_ID && preferences.default_custom_command.trim().is_empty())
    {
        preferences.default_terminal_profile_id = ProfileId::new(TERMINAL_PROFILE_ID);
    } else if profile_id != preferences.default_terminal_profile_id.as_str() {
        preferences.default_terminal_profile_id = ProfileId::new(profile_id);
    }

    preferences
}

pub fn validate_preferences(preferences: &UserPreferences) -> Result<(), SettingsError> {
    // FontSize is validated at construction time via FontSize::new().
    // LayoutPreset is validated at construction time via the enum type.

    resolve_terminal_profile(
        preferences.default_terminal_profile_id.as_str(),
        None,
        &preferences.default_custom_command,
    )?;

    Ok(())
}

pub fn resolve_terminal_profile(
    profile_id: &str,
    command_override: Option<CommandTemplate>,
    default_custom_command: &str,
) -> Result<ResolvedTerminalProfile, SettingsError> {
    let catalog = built_in_profile_catalog();
    let profile = catalog
        .terminal_profiles
        .into_iter()
        .find(|candidate| candidate.id == profile_id)
        .ok_or_else(|| SettingsError::Validation(format!("Unknown profile: {profile_id}")))?;

    if profile.id == CUSTOM_PROFILE_ID {
        let command = command_override
            .filter(|value| !value.as_str().trim().is_empty())
            .or_else(|| {
                let trimmed = default_custom_command.trim();
                (!trimmed.is_empty()).then(|| CommandTemplate::new(trimmed))
            })
            .ok_or_else(|| {
                SettingsError::Validation(String::from("Custom profile requires a startup command"))
            })?;

        return Ok(ResolvedTerminalProfile {
            id: profile.id,
            label: profile.label,
            command: Some(command),
        });
    }

    Ok(ResolvedTerminalProfile {
        id: profile.id,
        label: profile.label,
        command: profile.startup_command_template,
    })
}

pub fn resolve_default_working_directory(
    explicit: Option<&str>,
    preferences: &UserPreferences,
) -> String {
    explicit
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
        .or_else(|| {
            let value = preferences.default_working_directory.as_str().trim();
            (!value.is_empty()).then(|| String::from(value))
        })
        .or_else(|| {
            preferences
                .last_working_directory
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(String::from)
        })
        .unwrap_or_else(|| String::from("~"))
}

#[cfg(test)]
mod tests {
    use super::{
        built_in_profile_catalog, default_preferences, normalize_preferences,
        resolve_default_working_directory, resolve_terminal_profile, validate_preferences,
        FontSize, LayoutPreset, ProfileId, SettingsError,
        TerminalProfile, UserPreferences, WorkingDirectory, CLAUDE_PROFILE_ID,
        CODEX_PROFILE_ID, CUSTOM_PROFILE_ID, GEMINI_PROFILE_ID, OPENCODE_PROFILE_ID,
        TERMINAL_PROFILE_ID,
    };
    use tabby_kernel::CommandTemplate;

    // -----------------------------------------------------------------------
    // default_preferences
    // -----------------------------------------------------------------------

    #[test]
    fn default_preferences_use_terminal_profile() {
        assert_eq!(
            default_preferences().default_terminal_profile_id,
            TERMINAL_PROFILE_ID
        );
    }

    #[test]
    fn default_preferences_use_one_by_one_layout() {
        assert_eq!(default_preferences().default_layout, LayoutPreset::OneByOne);
    }

    #[test]
    fn default_preferences_font_size_is_valid() {
        let prefs = default_preferences();
        assert_eq!(prefs.font_size.value(), 13);
    }

    #[test]
    fn default_preferences_theme_is_system() {
        assert_eq!(default_preferences().theme, "system");
    }

    #[test]
    fn default_preferences_launch_fullscreen_is_true() {
        assert!(default_preferences().launch_fullscreen);
    }

    #[test]
    fn default_preferences_onboarding_not_completed() {
        assert!(!default_preferences().has_completed_onboarding);
    }

    #[test]
    fn default_preferences_last_working_directory_is_none() {
        assert!(default_preferences().last_working_directory.is_none());
    }

    #[test]
    fn default_preferences_working_directory_is_empty() {
        assert!(default_preferences().default_working_directory.is_empty());
    }

    #[test]
    fn default_preferences_custom_command_is_empty() {
        assert!(default_preferences().default_custom_command.is_empty());
    }

    #[test]
    fn default_preferences_clone_is_equal() {
        let prefs = default_preferences();
        let cloned = prefs.clone();
        assert_eq!(prefs, cloned);
    }

    #[test]
    fn user_preferences_immutable_field_update() {
        let original = default_preferences();
        let updated = UserPreferences {
            theme: String::from("midnight"),
            ..original.clone()
        };
        // Original is unchanged
        assert_eq!(original.theme, "system");
        // Updated reflects change
        assert_eq!(updated.theme, "midnight");
        // Other fields preserved
        assert_eq!(updated.font_size, original.font_size);
        assert_eq!(updated.default_layout, original.default_layout);
    }

    #[test]
    fn user_preferences_debug_format_contains_theme() {
        let prefs = default_preferences();
        let debug = format!("{prefs:?}");
        assert!(debug.contains("system"));
    }

    // -----------------------------------------------------------------------
    // font_size validation (via lib.rs test block, duplicating edge cases)
    // -----------------------------------------------------------------------

    #[test]
    fn font_size_validation_is_enforced_at_construction() {
        assert!(FontSize::new(6).is_err());
        assert!(FontSize::new(7).is_err());
        assert!(FontSize::new(8).is_ok());
        assert!(FontSize::new(14).is_ok());
        assert!(FontSize::new(72).is_ok());
        assert!(FontSize::new(73).is_err());
    }

    // -----------------------------------------------------------------------
    // built_in_profile_catalog
    // -----------------------------------------------------------------------

    #[test]
    fn built_in_catalog_contains_six_profiles() {
        let catalog = built_in_profile_catalog();
        assert_eq!(catalog.terminal_profiles.len(), 6);
    }

    #[test]
    fn built_in_catalog_contains_terminal_profile() {
        let catalog = built_in_profile_catalog();
        assert!(catalog
            .terminal_profiles
            .iter()
            .any(|p| p.id == TERMINAL_PROFILE_ID));
    }

    #[test]
    fn built_in_catalog_contains_claude_profile() {
        let catalog = built_in_profile_catalog();
        let profile = catalog
            .terminal_profiles
            .iter()
            .find(|p| p.id == CLAUDE_PROFILE_ID)
            .expect("claude profile should exist");
        assert_eq!(profile.label, "Claude Code");
        assert!(profile.startup_command_template.is_some());
        assert_eq!(
            profile.startup_command_template.as_ref().unwrap().as_str(),
            "claude"
        );
    }

    #[test]
    fn built_in_catalog_contains_codex_profile() {
        let catalog = built_in_profile_catalog();
        let profile = catalog
            .terminal_profiles
            .iter()
            .find(|p| p.id == CODEX_PROFILE_ID)
            .expect("codex profile should exist");
        assert_eq!(profile.label, "Codex");
        assert_eq!(
            profile.startup_command_template.as_ref().unwrap().as_str(),
            "codex"
        );
    }

    #[test]
    fn built_in_catalog_contains_gemini_profile() {
        let catalog = built_in_profile_catalog();
        let profile = catalog
            .terminal_profiles
            .iter()
            .find(|p| p.id == GEMINI_PROFILE_ID)
            .expect("gemini profile should exist");
        assert_eq!(profile.label, "Gemini CLI");
        assert_eq!(
            profile.startup_command_template.as_ref().unwrap().as_str(),
            "gemini"
        );
    }

    #[test]
    fn built_in_catalog_contains_opencode_profile() {
        let catalog = built_in_profile_catalog();
        let profile = catalog
            .terminal_profiles
            .iter()
            .find(|p| p.id == OPENCODE_PROFILE_ID)
            .expect("opencode profile should exist");
        assert_eq!(profile.label, "OpenCode CLI");
        assert_eq!(
            profile.startup_command_template.as_ref().unwrap().as_str(),
            "opencode"
        );
    }

    #[test]
    fn built_in_catalog_terminal_profile_has_no_startup_command() {
        let catalog = built_in_profile_catalog();
        let profile = catalog
            .terminal_profiles
            .iter()
            .find(|p| p.id == TERMINAL_PROFILE_ID)
            .expect("terminal profile should exist");
        assert!(profile.startup_command_template.is_none());
    }

    #[test]
    fn built_in_catalog_custom_profile_has_no_startup_command() {
        let catalog = built_in_profile_catalog();
        let profile = catalog
            .terminal_profiles
            .iter()
            .find(|p| p.id == CUSTOM_PROFILE_ID)
            .expect("custom profile should exist");
        assert!(profile.startup_command_template.is_none());
    }

    #[test]
    fn built_in_catalog_custom_profile_description() {
        let catalog = built_in_profile_catalog();
        let profile = catalog
            .terminal_profiles
            .iter()
            .find(|p| p.id == CUSTOM_PROFILE_ID)
            .unwrap();
        assert_eq!(profile.description, "Run any command");
    }

    #[test]
    fn terminal_profile_clone_is_equal() {
        let catalog = built_in_profile_catalog();
        let profile = catalog.terminal_profiles[0].clone();
        let cloned = profile.clone();
        assert_eq!(profile, cloned);
    }

    #[test]
    fn profile_catalog_clone_is_equal() {
        let catalog = built_in_profile_catalog();
        let cloned = catalog.clone();
        assert_eq!(catalog, cloned);
    }

    // -----------------------------------------------------------------------
    // resolve_terminal_profile
    // -----------------------------------------------------------------------

    #[test]
    fn custom_profile_requires_command() {
        let error = resolve_terminal_profile(CUSTOM_PROFILE_ID, None, "")
            .expect_err("custom profile should require a command");
        assert!(error.to_string().contains("startup command"));
    }

    #[test]
    fn resolve_terminal_profile_returns_terminal_with_no_command() {
        let resolved = resolve_terminal_profile(TERMINAL_PROFILE_ID, None, "").unwrap();
        assert_eq!(resolved.id, TERMINAL_PROFILE_ID);
        assert_eq!(resolved.label, "Terminal");
        assert!(resolved.command.is_none());
    }

    #[test]
    fn resolve_terminal_profile_returns_claude_with_command() {
        let resolved = resolve_terminal_profile(CLAUDE_PROFILE_ID, None, "").unwrap();
        assert_eq!(resolved.id, CLAUDE_PROFILE_ID);
        assert_eq!(resolved.label, "Claude Code");
        assert_eq!(resolved.command.unwrap().as_str(), "claude");
    }

    #[test]
    fn resolve_terminal_profile_returns_codex_with_command() {
        let resolved = resolve_terminal_profile(CODEX_PROFILE_ID, None, "").unwrap();
        assert_eq!(resolved.command.unwrap().as_str(), "codex");
    }

    #[test]
    fn resolve_terminal_profile_returns_gemini_with_command() {
        let resolved = resolve_terminal_profile(GEMINI_PROFILE_ID, None, "").unwrap();
        assert_eq!(resolved.command.unwrap().as_str(), "gemini");
    }

    #[test]
    fn resolve_terminal_profile_returns_opencode_with_command() {
        let resolved = resolve_terminal_profile(OPENCODE_PROFILE_ID, None, "").unwrap();
        assert_eq!(resolved.command.unwrap().as_str(), "opencode");
    }

    #[test]
    fn resolve_terminal_profile_unknown_id_returns_error() {
        let err = resolve_terminal_profile("nonexistent", None, "").unwrap_err();
        assert!(err.to_string().contains("Unknown profile"));
        assert!(err.to_string().contains("nonexistent"));
    }

    #[test]
    fn resolve_custom_profile_with_default_command() {
        let resolved =
            resolve_terminal_profile(CUSTOM_PROFILE_ID, None, "vim").unwrap();
        assert_eq!(resolved.id, CUSTOM_PROFILE_ID);
        assert_eq!(resolved.command.unwrap().as_str(), "vim");
    }

    #[test]
    fn resolve_custom_profile_command_override_takes_priority() {
        let override_cmd = CommandTemplate::new("zsh");
        let resolved =
            resolve_terminal_profile(CUSTOM_PROFILE_ID, Some(override_cmd), "vim").unwrap();
        // Override should win over default_custom_command
        assert_eq!(resolved.command.unwrap().as_str(), "zsh");
    }

    #[test]
    fn resolve_custom_profile_whitespace_only_override_falls_back_to_default() {
        let override_cmd = CommandTemplate::new("   ");
        let resolved =
            resolve_terminal_profile(CUSTOM_PROFILE_ID, Some(override_cmd), "fish").unwrap();
        assert_eq!(resolved.command.unwrap().as_str(), "fish");
    }

    #[test]
    fn resolve_custom_profile_whitespace_only_default_command_errors() {
        let err = resolve_terminal_profile(CUSTOM_PROFILE_ID, None, "   ").unwrap_err();
        assert!(err.to_string().contains("startup command"));
    }

    #[test]
    fn resolve_custom_profile_trims_default_command() {
        let resolved =
            resolve_terminal_profile(CUSTOM_PROFILE_ID, None, "  fish  ").unwrap();
        assert_eq!(resolved.command.unwrap().as_str(), "fish");
    }

    #[test]
    fn resolved_terminal_profile_debug_format() {
        let resolved = resolve_terminal_profile(TERMINAL_PROFILE_ID, None, "").unwrap();
        let debug = format!("{resolved:?}");
        assert!(debug.contains("Terminal"));
    }

    // -----------------------------------------------------------------------
    // normalize_preferences
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_preferences_fixes_invalid_defaults() {
        let normalized = normalize_preferences(UserPreferences {
            default_terminal_profile_id: ProfileId::new("browser"),
            ..default_preferences()
        });
        assert_eq!(normalized.default_terminal_profile_id, TERMINAL_PROFILE_ID);
    }

    #[test]
    fn normalize_preferences_keeps_valid_claude_profile() {
        let normalized = normalize_preferences(UserPreferences {
            default_terminal_profile_id: ProfileId::new(CLAUDE_PROFILE_ID),
            ..default_preferences()
        });
        assert_eq!(normalized.default_terminal_profile_id, CLAUDE_PROFILE_ID);
    }

    #[test]
    fn normalize_preferences_keeps_valid_codex_profile() {
        let normalized = normalize_preferences(UserPreferences {
            default_terminal_profile_id: ProfileId::new(CODEX_PROFILE_ID),
            ..default_preferences()
        });
        assert_eq!(normalized.default_terminal_profile_id, CODEX_PROFILE_ID);
    }

    #[test]
    fn normalize_preferences_fixes_empty_profile_id_to_terminal() {
        let normalized = normalize_preferences(UserPreferences {
            default_terminal_profile_id: ProfileId::new(""),
            ..default_preferences()
        });
        assert_eq!(normalized.default_terminal_profile_id, TERMINAL_PROFILE_ID);
    }

    #[test]
    fn normalize_preferences_fixes_custom_profile_without_command() {
        let normalized = normalize_preferences(UserPreferences {
            default_terminal_profile_id: ProfileId::new(CUSTOM_PROFILE_ID),
            default_custom_command: String::new(),
            ..default_preferences()
        });
        assert_eq!(normalized.default_terminal_profile_id, TERMINAL_PROFILE_ID);
    }

    #[test]
    fn normalize_preferences_keeps_custom_profile_with_command() {
        let normalized = normalize_preferences(UserPreferences {
            default_terminal_profile_id: ProfileId::new(CUSTOM_PROFILE_ID),
            default_custom_command: String::from("fish"),
            ..default_preferences()
        });
        assert_eq!(normalized.default_terminal_profile_id, CUSTOM_PROFILE_ID);
    }

    #[test]
    fn normalize_preferences_fixes_whitespace_only_custom_command() {
        let normalized = normalize_preferences(UserPreferences {
            default_terminal_profile_id: ProfileId::new(CUSTOM_PROFILE_ID),
            default_custom_command: String::from("   "),
            ..default_preferences()
        });
        assert_eq!(normalized.default_terminal_profile_id, TERMINAL_PROFILE_ID);
    }

    #[test]
    fn normalize_preferences_does_not_change_other_fields() {
        let original = UserPreferences {
            default_terminal_profile_id: ProfileId::new(CLAUDE_PROFILE_ID),
            theme: String::from("midnight"),
            font_size: FontSize::new(16).unwrap(),
            ..default_preferences()
        };
        let normalized = normalize_preferences(original.clone());
        assert_eq!(normalized.theme, "midnight");
        assert_eq!(normalized.font_size.value(), 16);
        assert_eq!(normalized.default_layout, original.default_layout);
    }

    // -----------------------------------------------------------------------
    // validate_preferences
    // -----------------------------------------------------------------------

    #[test]
    fn validate_preferences_succeeds_for_default() {
        let prefs = default_preferences();
        assert!(validate_preferences(&prefs).is_ok());
    }

    #[test]
    fn validate_preferences_succeeds_for_claude_profile() {
        let prefs = UserPreferences {
            default_terminal_profile_id: ProfileId::new(CLAUDE_PROFILE_ID),
            ..default_preferences()
        };
        assert!(validate_preferences(&prefs).is_ok());
    }

    #[test]
    fn validate_preferences_succeeds_for_custom_profile_with_command() {
        let prefs = UserPreferences {
            default_terminal_profile_id: ProfileId::new(CUSTOM_PROFILE_ID),
            default_custom_command: String::from("vim"),
            ..default_preferences()
        };
        assert!(validate_preferences(&prefs).is_ok());
    }

    #[test]
    fn validate_preferences_fails_for_unknown_profile() {
        let prefs = UserPreferences {
            default_terminal_profile_id: ProfileId::new("not-a-real-profile"),
            ..default_preferences()
        };
        let err = validate_preferences(&prefs).unwrap_err();
        assert!(err.to_string().contains("Unknown profile"));
    }

    #[test]
    fn validate_preferences_fails_for_custom_profile_without_command() {
        let prefs = UserPreferences {
            default_terminal_profile_id: ProfileId::new(CUSTOM_PROFILE_ID),
            default_custom_command: String::new(),
            ..default_preferences()
        };
        let err = validate_preferences(&prefs).unwrap_err();
        assert!(err.to_string().contains("startup command"));
    }

    // -----------------------------------------------------------------------
    // resolve_default_working_directory
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_default_working_directory_prefers_explicit_then_defaults() {
        let mut preferences = default_preferences();
        preferences.default_working_directory =
            WorkingDirectory::new("/defaults").expect("valid path");
        preferences.last_working_directory = Some(String::from("/last"));

        assert_eq!(
            resolve_default_working_directory(Some(" /tmp/project "), &preferences),
            "/tmp/project"
        );
        assert_eq!(
            resolve_default_working_directory(None, &preferences),
            "/defaults"
        );
    }

    #[test]
    fn resolve_default_working_directory_falls_back_to_last_when_default_empty() {
        let preferences = UserPreferences {
            last_working_directory: Some(String::from("/last")),
            ..default_preferences()
        };
        assert_eq!(
            resolve_default_working_directory(None, &preferences),
            "/last"
        );
    }

    #[test]
    fn resolve_default_working_directory_falls_back_to_tilde_when_all_empty() {
        let preferences = default_preferences();
        assert_eq!(
            resolve_default_working_directory(None, &preferences),
            "~"
        );
    }

    #[test]
    fn resolve_default_working_directory_ignores_whitespace_only_explicit() {
        let preferences = UserPreferences {
            default_working_directory: WorkingDirectory::new("/fallback").unwrap(),
            ..default_preferences()
        };
        assert_eq!(
            resolve_default_working_directory(Some("   "), &preferences),
            "/fallback"
        );
    }

    #[test]
    fn resolve_default_working_directory_ignores_whitespace_only_last() {
        let preferences = UserPreferences {
            last_working_directory: Some(String::from("   ")),
            ..default_preferences()
        };
        assert_eq!(
            resolve_default_working_directory(None, &preferences),
            "~"
        );
    }

    #[test]
    fn resolve_default_working_directory_trims_last_working_directory() {
        let preferences = UserPreferences {
            last_working_directory: Some(String::from("  /trimmed  ")),
            ..default_preferences()
        };
        assert_eq!(
            resolve_default_working_directory(None, &preferences),
            "/trimmed"
        );
    }

    // -----------------------------------------------------------------------
    // SettingsError
    // -----------------------------------------------------------------------

    #[test]
    fn settings_error_validation_display() {
        let err = SettingsError::Validation(String::from("test error message"));
        assert!(err.to_string().contains("test error message"));
        assert!(err.to_string().contains("validation error"));
    }

    #[test]
    fn settings_error_from_value_object_error() {
        let vo_err = tabby_kernel::ValueObjectError::new("some constraint violated");
        let settings_err = SettingsError::from(vo_err);
        assert!(settings_err.to_string().contains("some constraint violated"));
    }

    // -----------------------------------------------------------------------
    // TerminalProfile struct
    // -----------------------------------------------------------------------

    #[test]
    fn terminal_profile_fields_are_accessible() {
        let profile = TerminalProfile {
            id: ProfileId::new("my-profile"),
            label: String::from("My Profile"),
            description: String::from("A test profile"),
            startup_command_template: Some(CommandTemplate::new("zsh")),
        };
        assert_eq!(profile.id.as_str(), "my-profile");
        assert_eq!(profile.label, "My Profile");
        assert_eq!(profile.description, "A test profile");
        assert_eq!(
            profile.startup_command_template.unwrap().as_str(),
            "zsh"
        );
    }

    #[test]
    fn terminal_profile_without_command_is_valid() {
        let profile = TerminalProfile {
            id: ProfileId::new("bare"),
            label: String::from("Bare"),
            description: String::from("No command"),
            startup_command_template: None,
        };
        assert!(profile.startup_command_template.is_none());
    }

    #[test]
    fn terminal_profile_with_special_characters_in_label() {
        let profile = TerminalProfile {
            id: ProfileId::new("special"),
            label: String::from("Ünïcödé & Emojis 🚀"),
            description: String::from("Special chars"),
            startup_command_template: None,
        };
        assert_eq!(profile.label, "Ünïcödé & Emojis 🚀");
    }

    // -----------------------------------------------------------------------
    // Profile ID constants
    // -----------------------------------------------------------------------

    #[test]
    fn profile_id_constants_have_expected_values() {
        assert_eq!(CUSTOM_PROFILE_ID, "custom");
        assert_eq!(TERMINAL_PROFILE_ID, "terminal");
        assert_eq!(CLAUDE_PROFILE_ID, "claude");
        assert_eq!(CODEX_PROFILE_ID, "codex");
        assert_eq!(GEMINI_PROFILE_ID, "gemini");
        assert_eq!(OPENCODE_PROFILE_ID, "opencode");
    }
}
