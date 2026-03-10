mod cli_git_adapter;
mod tauri_browser_surface_adapter;
mod tauri_projection_publisher;
mod tauri_store_preferences_repository;

// CliGitAdapter is not yet wired into the app shell; will be connected in a follow-up story.
#[allow(unused_imports)]
pub use cli_git_adapter::CliGitAdapter;
pub use tauri_browser_surface_adapter::TauriBrowserSurfaceAdapter;
pub use tauri_projection_publisher::TauriProjectionPublisher;
pub use tauri_store_preferences_repository::TauriStorePreferencesRepository;
