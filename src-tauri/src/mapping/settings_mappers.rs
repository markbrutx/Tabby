use tabby_contracts::{ProfileCatalogView, ProfileView, SettingsView};
use tabby_settings::{
    FontSize, ProfileCatalog, ProfileId, SettingsError, UserPreferences, WorkingDirectory,
};

use super::workspace_mappers::{layout_preset_from_dto, layout_preset_to_dto};

// ---------------------------------------------------------------------------
// Domain → DTO (outbound / projections)
// ---------------------------------------------------------------------------

pub(crate) fn settings_view_from_preferences(preferences: &UserPreferences) -> SettingsView {
    SettingsView {
        default_layout: layout_preset_to_dto(preferences.default_layout),
        default_terminal_profile_id: preferences.default_terminal_profile_id.as_str().to_string(),
        default_working_directory: preferences.default_working_directory.as_str().to_string(),
        default_custom_command: preferences.default_custom_command.clone(),
        font_size: preferences.font_size.value(),
        theme: preferences.theme.clone(),
        launch_fullscreen: preferences.launch_fullscreen,
        has_completed_onboarding: preferences.has_completed_onboarding,
        last_working_directory: preferences.last_working_directory.clone(),
    }
}

pub(crate) fn profile_catalog_view_from_catalog(catalog: &ProfileCatalog) -> ProfileCatalogView {
    ProfileCatalogView {
        terminal_profiles: catalog
            .terminal_profiles
            .iter()
            .map(|profile| ProfileView {
                id: profile.id.as_str().to_string(),
                label: profile.label.clone(),
                description: profile.description.clone(),
                startup_command_template: profile
                    .startup_command_template
                    .as_ref()
                    .map(|c| c.as_str().to_string()),
            })
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// DTO → Domain (inbound)
// ---------------------------------------------------------------------------

pub(crate) fn preferences_from_settings_view(
    view: &SettingsView,
) -> Result<UserPreferences, SettingsError> {
    Ok(UserPreferences {
        default_layout: layout_preset_from_dto(view.default_layout),
        default_terminal_profile_id: ProfileId::new(view.default_terminal_profile_id.clone()),
        default_working_directory: WorkingDirectory::new(view.default_working_directory.clone())?,
        default_custom_command: view.default_custom_command.clone(),
        font_size: FontSize::new(view.font_size)?,
        theme: view.theme.clone(),
        launch_fullscreen: view.launch_fullscreen,
        has_completed_onboarding: view.has_completed_onboarding,
        last_working_directory: view.last_working_directory.clone(),
    })
}
