use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use tabby_settings::UserPreferences;

use crate::application::ports::PreferencesRepository;
use crate::shell::error::ShellError;

const STORE_PATH: &str = "tabby-settings.json";
const SETTINGS_KEY: &str = "settings";

/// Infrastructure adapter that persists user preferences via `tauri-plugin-store`.
///
/// Uses the persistence schema from `tabby_settings::persistence`, which is
/// independent of the IPC DTO (`SettingsView`) used by the transport layer.
#[derive(Debug, Clone)]
pub struct TauriStorePreferencesRepository {
    app: AppHandle,
}

impl TauriStorePreferencesRepository {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl PreferencesRepository for TauriStorePreferencesRepository {
    fn load(&self) -> Result<Option<serde_json::Value>, ShellError> {
        let store = self
            .app
            .store(STORE_PATH)
            .map_err(|error| ShellError::Store(error.to_string()))?;
        Ok(store.get(SETTINGS_KEY))
    }

    fn save(&self, preferences: &UserPreferences) -> Result<(), ShellError> {
        let store = self
            .app
            .store(STORE_PATH)
            .map_err(|error| ShellError::Store(error.to_string()))?;
        let value = tabby_settings::persistence::serialize_preferences(preferences)
            .map_err(|error| ShellError::Serialization(error.to_string()))?;
        store.set(SETTINGS_KEY, value);
        Ok(())
    }
}
