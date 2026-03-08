mod value_objects;

pub use value_objects::{FontSize, ProfileId, WorkingDirectory};

use thiserror::Error;

pub const CUSTOM_PROFILE_ID: &str = "custom";
pub const TERMINAL_PROFILE_ID: &str = "terminal";
pub const CLAUDE_PROFILE_ID: &str = "claude";
pub const CODEX_PROFILE_ID: &str = "codex";
pub const DEFAULT_LAYOUT_PRESET: &str = "1x1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Dawn,
    Midnight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalProfile {
    pub id: ProfileId,
    pub label: String,
    pub description: String,
    pub startup_command_template: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileCatalog {
    pub terminal_profiles: Vec<TerminalProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserPreferences {
    pub default_layout: String,
    pub default_terminal_profile_id: ProfileId,
    pub default_working_directory: WorkingDirectory,
    pub default_custom_command: String,
    pub font_size: FontSize,
    pub theme: ThemeMode,
    pub launch_fullscreen: bool,
    pub has_completed_onboarding: bool,
    pub last_working_directory: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTerminalProfile {
    pub id: ProfileId,
    pub label: String,
    pub command: Option<String>,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("validation error: {0}")]
    Validation(String),
}

pub fn default_preferences() -> UserPreferences {
    UserPreferences {
        default_layout: String::from(DEFAULT_LAYOUT_PRESET),
        default_terminal_profile_id: ProfileId::new(TERMINAL_PROFILE_ID),
        default_working_directory: WorkingDirectory::empty(),
        default_custom_command: String::new(),
        font_size: FontSize::default(),
        theme: ThemeMode::System,
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
                startup_command_template: Some(String::from("claude")),
            },
            TerminalProfile {
                id: ProfileId::new(CODEX_PROFILE_ID),
                label: String::from("Codex"),
                description: String::from("OpenAI coding agent"),
                startup_command_template: Some(String::from("codex")),
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

    if !is_known_layout_preset(&preferences.default_layout) {
        preferences.default_layout = String::from(DEFAULT_LAYOUT_PRESET);
    }

    preferences
}

pub fn validate_preferences(preferences: &UserPreferences) -> Result<(), SettingsError> {
    // FontSize is validated at construction time via FontSize::new().

    if !is_known_layout_preset(&preferences.default_layout) {
        return Err(SettingsError::Validation(format!(
            "Unknown layout preset: {}",
            preferences.default_layout
        )));
    }

    resolve_terminal_profile(
        preferences.default_terminal_profile_id.as_str(),
        None,
        &preferences.default_custom_command,
    )?;

    Ok(())
}

pub fn resolve_terminal_profile(
    profile_id: &str,
    command_override: Option<String>,
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
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                (!default_custom_command.trim().is_empty())
                    .then(|| String::from(default_custom_command.trim()))
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

pub fn is_known_layout_preset(value: &str) -> bool {
    matches!(value, "1x1" | "1x2" | "2x2" | "2x3" | "3x3")
}

#[cfg(test)]
mod tests {
    use super::{
        default_preferences, normalize_preferences, resolve_default_working_directory,
        resolve_terminal_profile, validate_preferences, FontSize, ProfileId, WorkingDirectory,
        CUSTOM_PROFILE_ID, TERMINAL_PROFILE_ID,
    };

    #[test]
    fn default_preferences_use_terminal_profile() {
        assert_eq!(
            default_preferences().default_terminal_profile_id,
            TERMINAL_PROFILE_ID
        );
    }

    #[test]
    fn custom_profile_requires_command() {
        let error = resolve_terminal_profile(CUSTOM_PROFILE_ID, None, "")
            .expect_err("custom profile should require a command");
        assert!(error.to_string().contains("startup command"));
    }

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
    fn normalize_preferences_fixes_invalid_defaults() {
        let normalized = normalize_preferences(super::UserPreferences {
            default_terminal_profile_id: ProfileId::new("browser"),
            ..default_preferences()
        });
        assert_eq!(normalized.default_terminal_profile_id, TERMINAL_PROFILE_ID);
    }

    #[test]
    fn validate_preferences_rejects_unknown_layout() {
        let error = validate_preferences(&super::UserPreferences {
            default_layout: String::from("4x4"),
            ..default_preferences()
        })
        .expect_err("unknown layout should fail");
        assert!(error.to_string().contains("Unknown layout"));
    }

    #[test]
    fn font_size_validation_is_enforced_at_construction() {
        assert!(FontSize::new(6).is_err());
        assert!(FontSize::new(7).is_err());
        assert!(FontSize::new(8).is_ok());
        assert!(FontSize::new(14).is_ok());
        assert!(FontSize::new(72).is_ok());
        assert!(FontSize::new(73).is_err());
    }

    #[test]
    fn default_preferences_font_size_is_valid() {
        let prefs = default_preferences();
        assert_eq!(prefs.font_size.value(), 13);
    }
}
