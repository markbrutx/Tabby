use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::settings::domain::app_settings::{default_settings, normalize_settings, AppSettings};
use crate::settings::domain::profiles::{is_known_profile_id, CUSTOM_PROFILE_ID};
use crate::shared::error::TabbyError;

const STORE_PATH: &str = "tabby-settings.json";
const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone)]
pub struct SettingsManager {
    app: AppHandle,
}

impl SettingsManager {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn get_settings(&self) -> Result<AppSettings, TabbyError> {
        let store = self
            .app
            .store(STORE_PATH)
            .map_err(|error| TabbyError::Store(error.to_string()))?;

        if let Some(value) = store.get(SETTINGS_KEY) {
            let settings = serde_json::from_value::<AppSettings>(value.clone())
                .map_err(|error| TabbyError::Serialization(error.to_string()))?;
            let normalized = normalize_settings(settings);
            let current = serde_json::from_value::<AppSettings>(value)
                .map_err(|error| TabbyError::Serialization(error.to_string()))?;
            if normalized != current {
                let _ = self.write_settings(&normalized)?;
            }
            return Ok(normalized);
        }

        let settings = self.default_settings();
        let value = serde_json::to_value(&settings)
            .map_err(|error| TabbyError::Serialization(error.to_string()))?;
        store.set(SETTINGS_KEY, value);
        Ok(settings)
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<AppSettings, TabbyError> {
        let normalized = normalize_settings(settings);
        self.validate_settings(&normalized)?;
        self.write_settings(&normalized)
    }

    pub fn reset_settings(&self) -> Result<AppSettings, TabbyError> {
        let defaults = self.default_settings();
        self.write_settings(&defaults)
    }

    pub fn update_last_working_directory(&self, cwd: &str) -> Result<(), TabbyError> {
        let mut settings = self.get_settings()?;
        settings.last_working_directory = Some(String::from(cwd));
        self.write_settings(&settings)?;
        Ok(())
    }

    fn write_settings(&self, settings: &AppSettings) -> Result<AppSettings, TabbyError> {
        let store = self
            .app
            .store(STORE_PATH)
            .map_err(|error| TabbyError::Store(error.to_string()))?;
        let value = serde_json::to_value(settings)
            .map_err(|error| TabbyError::Serialization(error.to_string()))?;

        store.set(SETTINGS_KEY, value);
        Ok(settings.clone())
    }

    fn default_settings(&self) -> AppSettings {
        default_settings()
    }

    fn validate_settings(&self, settings: &AppSettings) -> Result<(), TabbyError> {
        if settings.font_size < 10 || settings.font_size > 24 {
            return Err(TabbyError::Validation(String::from(
                "Font size must be between 10 and 24",
            )));
        }

        if settings.default_profile_id.trim().is_empty() {
            return Err(TabbyError::Validation(String::from(
                "Default profile must not be empty",
            )));
        }

        if !is_known_profile_id(&settings.default_profile_id) {
            return Err(TabbyError::Validation(format!(
                "Unknown default profile: {}",
                settings.default_profile_id
            )));
        }

        if settings.default_profile_id == CUSTOM_PROFILE_ID
            && settings.default_custom_command.trim().is_empty()
        {
            return Err(TabbyError::Validation(String::from(
                "Default custom profile requires a default custom command",
            )));
        }

        Ok(())
    }
}
