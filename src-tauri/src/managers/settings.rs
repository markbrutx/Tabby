use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::domain::error::TabbyError;
use crate::domain::settings::{default_settings, AppSettings};

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
            return serde_json::from_value::<AppSettings>(value)
                .map_err(|error| TabbyError::Serialization(error.to_string()));
        }

        let settings = self.default_settings();
        let value = serde_json::to_value(&settings)
            .map_err(|error| TabbyError::Serialization(error.to_string()))?;
        store.set(SETTINGS_KEY, value);
        Ok(settings)
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<AppSettings, TabbyError> {
        self.validate_settings(&settings)?;
        self.write_settings(&settings)
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

        if settings.default_profile_id == "custom"
            && settings.default_custom_command.trim().is_empty()
        {
            return Err(TabbyError::Validation(String::from(
                "Default custom profile requires a default custom command",
            )));
        }

        Ok(())
    }
}
