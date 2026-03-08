use tabby_settings::UserPreferences;

use crate::shell::error::ShellError;

/// Port for persisting and loading user preferences.
///
/// Infrastructure adapters implement this trait to decouple
/// `SettingsApplicationService` from any specific storage backend.
pub trait PreferencesRepository: Send + Sync + std::fmt::Debug {
    /// Load persisted preferences, or `None` if no preferences have been saved yet.
    fn load(&self) -> Result<Option<serde_json::Value>, ShellError>;

    /// Persist the given preferences.
    fn save(&self, preferences: &UserPreferences) -> Result<(), ShellError>;
}
