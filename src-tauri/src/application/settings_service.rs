use std::sync::Mutex;

use tracing::warn;

use tabby_settings::{
    default_preferences, normalize_preferences, validate_preferences, SettingsError,
    UserPreferences,
};

use crate::application::commands::SettingsCommand;
use crate::application::ports::PreferencesRepository;
use crate::mapping::dto_mappers;
use crate::shell::error::ShellError;

#[derive(Debug, Clone)]
struct LoadedPreferences {
    preferences: UserPreferences,
    should_persist: bool,
}

#[derive(Debug)]
pub struct SettingsApplicationService {
    repository: Box<dyn PreferencesRepository>,
    preferences: Mutex<UserPreferences>,
}

impl SettingsApplicationService {
    pub fn new(repository: Box<dyn PreferencesRepository>) -> Result<Self, ShellError> {
        let loaded = load_preferences(&*repository)?;
        Ok(Self {
            repository,
            preferences: Mutex::new(loaded),
        })
    }

    pub fn preferences(&self) -> Result<UserPreferences, ShellError> {
        self.preferences
            .lock()
            .map_err(|_| ShellError::State(String::from("Preferences lock poisoned")))
            .map(|guard| guard.clone())
    }

    pub fn dispatch_settings_command(
        &self,
        command: SettingsCommand,
    ) -> Result<UserPreferences, ShellError> {
        let next_preferences = match command {
            SettingsCommand::Update(update) => {
                let next = normalize_preferences(update.preferences);
                validate_preferences(&next).map_err(settings_error_to_shell)?;
                next
            }
            SettingsCommand::Reset => default_preferences(),
        };

        self.persist_preferences(&next_preferences)?;
        Ok(next_preferences)
    }

    pub fn persist_preferences(
        &self,
        next_preferences: &UserPreferences,
    ) -> Result<(), ShellError> {
        self.repository.save(next_preferences)?;

        let mut preferences = self
            .preferences
            .lock()
            .map_err(|_| ShellError::State(String::from("Preferences lock poisoned")))?;
        *preferences = next_preferences.clone();
        Ok(())
    }
}

fn load_preferences(repository: &dyn PreferencesRepository) -> Result<UserPreferences, ShellError> {
    let raw_value = repository.load()?;
    let loaded = decode_preferences(raw_value)?;

    if loaded.should_persist {
        repository.save(&loaded.preferences)?;
    }

    Ok(loaded.preferences)
}

fn decode_preferences(value: Option<serde_json::Value>) -> Result<LoadedPreferences, ShellError> {
    let Some(value) = value else {
        return Ok(LoadedPreferences {
            preferences: default_preferences(),
            should_persist: true,
        });
    };

    match dto_mappers::deserialize_preferences(value) {
        Ok(raw) => {
            let preferences = normalize_preferences(raw);
            validate_preferences(&preferences).map_err(settings_error_to_shell)?;
            Ok(LoadedPreferences {
                preferences,
                should_persist: false,
            })
        }
        Err(error) => {
            warn!(
                ?error,
                "Discarding incompatible persisted settings and resetting to defaults"
            );
            Ok(LoadedPreferences {
                preferences: default_preferences(),
                should_persist: true,
            })
        }
    }
}

fn settings_error_to_shell(error: SettingsError) -> ShellError {
    match error {
        SettingsError::Validation(message) => ShellError::Validation(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::commands::UpdateSettingsCommand;
    use serde_json::json;
    use std::sync::Mutex;
    use tabby_settings::FontSize;

    // ------------------------------------------------------------------
    // Mock PreferencesRepository for unit tests
    // ------------------------------------------------------------------

    #[derive(Debug)]
    struct MockPreferencesRepository {
        stored: Mutex<Option<serde_json::Value>>,
    }

    impl MockPreferencesRepository {
        fn empty() -> Self {
            Self {
                stored: Mutex::new(None),
            }
        }

        fn with_value(value: serde_json::Value) -> Self {
            Self {
                stored: Mutex::new(Some(value)),
            }
        }
    }

    impl PreferencesRepository for MockPreferencesRepository {
        fn load(&self) -> Result<Option<serde_json::Value>, ShellError> {
            let guard = self
                .stored
                .lock()
                .map_err(|_| ShellError::State(String::from("Mock lock poisoned")))?;
            Ok(guard.clone())
        }

        fn save(&self, preferences: &UserPreferences) -> Result<(), ShellError> {
            let value = dto_mappers::serialize_preferences(preferences)
                .map_err(|e| ShellError::Serialization(e.to_string()))?;
            let mut guard = self
                .stored
                .lock()
                .map_err(|_| ShellError::State(String::from("Mock lock poisoned")))?;
            *guard = Some(value);
            Ok(())
        }
    }

    // ------------------------------------------------------------------
    // Existing decode_preferences tests (kept intact)
    // ------------------------------------------------------------------

    #[test]
    fn loads_default_preferences_when_store_is_empty() {
        let loaded = decode_preferences(None).expect("should decode");
        assert!(loaded.should_persist);
        assert_eq!(loaded.preferences.default_terminal_profile_id, "terminal");
    }

    #[test]
    fn updates_preferences_via_dispatch() {
        let valid_json = json!({
            "defaultLayout": "2x2",
            "defaultTerminalProfileId": "claude",
            "defaultWorkingDirectory": "/tmp",
            "defaultCustomCommand": "",
            "fontSize": 14,
            "theme": "system",
            "launchFullscreen": false,
            "hasCompletedOnboarding": true,
            "lastWorkingDirectory": "/tmp"
        });

        let loaded = decode_preferences(Some(valid_json)).expect("should decode");
        assert!(!loaded.should_persist);
        assert_eq!(loaded.preferences.default_terminal_profile_id, "claude");
        assert_eq!(loaded.preferences.font_size.value(), 14);
    }

    #[test]
    fn resets_to_defaults_for_incompatible_settings() {
        let loaded = decode_preferences(Some(json!({
            "defaultLayout": "1x1",
            "defaultProfileId": "claude",
            "defaultWorkingDirectory": "/tmp",
            "defaultCustomCommand": "",
            "fontSize": 13,
            "theme": "system",
            "launchFullscreen": true,
            "hasCompletedOnboarding": true,
            "lastWorkingDirectory": null
        })))
        .expect("should fall back to defaults");

        assert!(loaded.should_persist);
        assert_eq!(loaded.preferences.default_terminal_profile_id, "terminal");
    }

    #[test]
    fn maps_settings_validation_error() {
        let error = settings_error_to_shell(tabby_settings::SettingsError::Validation(
            String::from("bad value"),
        ));
        assert!(error.to_string().contains("validation error"));
    }

    #[test]
    fn reset_returns_default_preferences() {
        let defaults = tabby_settings::default_preferences();
        assert_eq!(defaults.default_terminal_profile_id, "terminal");
        assert_eq!(defaults.font_size.value(), 13);
        assert!(
            defaults.launch_fullscreen,
            "default launch_fullscreen is true"
        );
        assert!(!defaults.has_completed_onboarding);
    }

    #[test]
    fn decode_preferences_with_malformed_json_falls_back_to_defaults() {
        let loaded =
            decode_preferences(Some(json!("just a string"))).expect("should fall back to defaults");
        assert!(
            loaded.should_persist,
            "should persist defaults after fallback"
        );
        assert_eq!(loaded.preferences.default_terminal_profile_id, "terminal");
    }

    #[test]
    fn decode_preferences_with_empty_object_falls_back_to_defaults() {
        let loaded = decode_preferences(Some(json!({}))).expect("should fall back to defaults");
        assert!(
            loaded.should_persist,
            "should persist defaults after fallback from empty object"
        );
        assert_eq!(loaded.preferences.default_terminal_profile_id, "terminal");
    }

    #[test]
    fn decode_preferences_with_null_value_returns_defaults() {
        let loaded = decode_preferences(Some(json!(null))).expect("should fall back to defaults");
        assert!(loaded.should_persist);
        assert_eq!(loaded.preferences.default_terminal_profile_id, "terminal");
    }

    #[test]
    fn decode_preferences_preserves_valid_font_size() {
        let valid_json = json!({
            "defaultLayout": "1x1",
            "defaultTerminalProfileId": "terminal",
            "defaultWorkingDirectory": "~",
            "defaultCustomCommand": "",
            "fontSize": 20,
            "theme": "system",
            "launchFullscreen": false,
            "hasCompletedOnboarding": false,
            "lastWorkingDirectory": null
        });

        let loaded = decode_preferences(Some(valid_json)).expect("should decode");
        assert!(!loaded.should_persist);
        assert_eq!(loaded.preferences.font_size.value(), 20);
    }

    #[test]
    fn decode_preferences_normalizes_out_of_range_font_size() {
        let json_with_bad_font = json!({
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

        let result = decode_preferences(Some(json_with_bad_font));
        assert!(
            result.is_ok(),
            "should handle out-of-range font size gracefully"
        );
    }

    // ------------------------------------------------------------------
    // New: SettingsApplicationService with mock PreferencesRepository
    // ------------------------------------------------------------------

    #[test]
    fn service_loads_defaults_from_empty_repository() {
        let repo = MockPreferencesRepository::empty();
        let service =
            SettingsApplicationService::new(Box::new(repo)).expect("should construct service");
        let prefs = service.preferences().expect("should read preferences");
        assert_eq!(prefs.default_terminal_profile_id, "terminal");
        assert_eq!(prefs.font_size.value(), 13);
    }

    #[test]
    fn service_loads_persisted_preferences_from_repository() {
        let stored = json!({
            "defaultLayout": "2x2",
            "defaultTerminalProfileId": "claude",
            "defaultWorkingDirectory": "/projects",
            "defaultCustomCommand": "",
            "fontSize": 16,
            "theme": "midnight",
            "launchFullscreen": false,
            "hasCompletedOnboarding": true,
            "lastWorkingDirectory": null
        });
        let repo = MockPreferencesRepository::with_value(stored);
        let service =
            SettingsApplicationService::new(Box::new(repo)).expect("should construct service");
        let prefs = service.preferences().expect("should read preferences");
        assert_eq!(prefs.default_terminal_profile_id, "claude");
        assert_eq!(prefs.font_size.value(), 16);
        assert!(!prefs.launch_fullscreen);
    }

    #[test]
    fn service_update_persists_via_repository() {
        let repo = MockPreferencesRepository::empty();
        let service =
            SettingsApplicationService::new(Box::new(repo)).expect("should construct service");

        let mut updated = default_preferences();
        updated.font_size = FontSize::new(20).expect("valid font size");

        let result = service
            .dispatch_settings_command(SettingsCommand::Update(UpdateSettingsCommand {
                preferences: updated,
            }))
            .expect("should dispatch update");

        assert_eq!(result.font_size.value(), 20);

        // Verify in-memory state is also updated
        let current = service.preferences().expect("should read preferences");
        assert_eq!(current.font_size.value(), 20);
    }

    #[test]
    fn service_reset_returns_defaults_and_persists() {
        let stored = json!({
            "defaultLayout": "2x2",
            "defaultTerminalProfileId": "claude",
            "defaultWorkingDirectory": "/projects",
            "defaultCustomCommand": "",
            "fontSize": 16,
            "theme": "midnight",
            "launchFullscreen": false,
            "hasCompletedOnboarding": true,
            "lastWorkingDirectory": null
        });
        let repo = MockPreferencesRepository::with_value(stored);
        let service =
            SettingsApplicationService::new(Box::new(repo)).expect("should construct service");

        let result = service
            .dispatch_settings_command(SettingsCommand::Reset)
            .expect("should dispatch reset");

        assert_eq!(result.default_terminal_profile_id, "terminal");
        assert_eq!(result.font_size.value(), 13);

        let current = service.preferences().expect("should read preferences");
        assert_eq!(current.default_terminal_profile_id, "terminal");
    }
}
