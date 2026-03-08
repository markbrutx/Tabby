use std::sync::Mutex;

use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tracing::warn;

use tabby_contracts::SettingsView;
use tabby_settings::{
    default_preferences, normalize_preferences, validate_preferences, SettingsError,
    UserPreferences,
};

use crate::application::commands::SettingsCommand;

use crate::shell::error::ShellError;
use crate::shell::mapping::{preferences_from_settings_view, settings_view_from_preferences};

const STORE_PATH: &str = "tabby-settings.json";
const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone)]
struct LoadedPreferences {
    preferences: UserPreferences,
    should_persist: bool,
}

#[derive(Debug)]
pub struct SettingsApplicationService {
    app: AppHandle,
    preferences: Mutex<UserPreferences>,
}

impl SettingsApplicationService {
    pub fn new(app: AppHandle) -> Result<Self, ShellError> {
        let preferences = load_preferences(&app)?;
        Ok(Self {
            app,
            preferences: Mutex::new(preferences),
        })
    }

    pub fn preferences(&self) -> Result<UserPreferences, ShellError> {
        self.preferences
            .lock()
            .map_err(|_| ShellError::State(String::from("Preferences lock poisoned")))
            .map(|guard| guard.clone())
    }

    pub fn settings_view(&self) -> Result<SettingsView, ShellError> {
        let preferences = self.preferences()?;
        Ok(settings_view_from_preferences(&preferences))
    }

    pub fn dispatch_settings_command(
        &self,
        command: SettingsCommand,
    ) -> Result<(UserPreferences, SettingsView), ShellError> {
        let next_preferences = match command {
            SettingsCommand::Update(update) => {
                let next = normalize_preferences(update.preferences);
                validate_preferences(&next).map_err(settings_error_to_shell)?;
                next
            }
            SettingsCommand::Reset => default_preferences(),
        };

        self.persist_preferences(&next_preferences)?;
        let view = settings_view_from_preferences(&next_preferences);
        Ok((next_preferences, view))
    }

    pub fn persist_preferences(
        &self,
        next_preferences: &UserPreferences,
    ) -> Result<(), ShellError> {
        let settings_view = settings_view_from_preferences(next_preferences);
        let store = self
            .app
            .store(STORE_PATH)
            .map_err(|error| ShellError::Store(error.to_string()))?;
        let value = serde_json::to_value(settings_view)
            .map_err(|error| ShellError::Serialization(error.to_string()))?;
        store.set(SETTINGS_KEY, value);

        let mut preferences = self
            .preferences
            .lock()
            .map_err(|_| ShellError::State(String::from("Preferences lock poisoned")))?;
        *preferences = next_preferences.clone();
        Ok(())
    }
}

fn load_preferences(app: &AppHandle) -> Result<UserPreferences, ShellError> {
    let store = app
        .store(STORE_PATH)
        .map_err(|error| ShellError::Store(error.to_string()))?;

    let loaded = decode_preferences(store.get(SETTINGS_KEY))?;
    if loaded.should_persist {
        let value = serde_json::to_value(settings_view_from_preferences(&loaded.preferences))
            .map_err(|error| ShellError::Serialization(error.to_string()))?;
        store.set(SETTINGS_KEY, value);
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

    match serde_json::from_value::<SettingsView>(value) {
        Ok(saved) => {
            let preferences = normalize_preferences(preferences_from_settings_view(&saved));
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
    use super::{decode_preferences, settings_error_to_shell};
    use serde_json::json;

    #[test]
    fn loads_default_preferences_when_store_is_empty() {
        let loaded = decode_preferences(None).expect("should decode");
        assert!(loaded.should_persist);
        assert_eq!(loaded.preferences.default_terminal_profile_id, "terminal");
    }

    #[test]
    fn updates_preferences_via_dispatch() {
        // Validates decode + normalize round-trip for the Update path.
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
        assert_eq!(loaded.preferences.font_size, 14);
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
}
