use std::path::Path;
use std::sync::Arc;

use tabby_git::value_objects::{BranchName, RemoteName, StashId};
use tabby_git::{
    BlameEntry, BranchInfo, CommitInfo, DiffContent, FileStatus, GitRepositoryState, StashEntry,
};
use tabby_runtime::PaneRuntime;
use tabby_settings::UserPreferences;
use tabby_workspace::WorkspaceSession;

use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
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

/// Port for publishing projections (workspace, settings, runtime status) to the frontend.
///
/// Infrastructure adapters implement this trait to decouple application services
/// from any specific event transport (e.g., Tauri `app.emit`).
pub trait ProjectionPublisherPort: Send + Sync + std::fmt::Debug {
    /// Publish a workspace projection update to the frontend.
    fn publish_workspace_projection(&self, workspace: &WorkspaceSession);

    /// Publish a settings projection update to the frontend.
    fn publish_settings_projection(&self, preferences: &UserPreferences);

    /// Publish a runtime status change for a single pane runtime.
    fn publish_runtime_status(&self, runtime: &PaneRuntime);
}

/// Port for managing terminal process (PTY) lifecycle.
///
/// Infrastructure adapters implement this trait to decouple
/// `RuntimeApplicationService` from any specific PTY backend.
pub trait TerminalProcessPort: Send + Sync + std::fmt::Debug {
    /// Spawn a new terminal process and return the runtime session ID.
    fn spawn(
        &self,
        pane_id: &str,
        working_directory: &str,
        startup_command: Option<&str>,
        observation_receiver: Arc<dyn RuntimeObservationReceiver>,
    ) -> Result<String, ShellError>;

    /// Terminate a terminal process by its runtime session ID.
    fn kill(&self, runtime_session_id: &str) -> Result<(), ShellError>;

    /// Resize a terminal process by its runtime session ID.
    fn resize(&self, runtime_session_id: &str, cols: u16, rows: u16) -> Result<(), ShellError>;

    /// Write user input to a terminal process by its runtime session ID.
    fn write_input(&self, runtime_session_id: &str, data: &str) -> Result<(), ShellError>;
}

/// Port for managing browser surface (webview) lifecycle.
///
/// Infrastructure adapters implement this trait to decouple
/// `RuntimeApplicationService` from any specific webview backend.
///
pub trait BrowserSurfacePort: Send + Sync + std::fmt::Debug {
    /// Ensure a browser surface exists for the given pane, creating it if needed.
    fn ensure_surface(
        &self,
        pane_id: &str,
        url: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<(), ShellError>;

    /// Update the position and size of an existing browser surface.
    fn set_bounds(
        &self,
        pane_id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<(), ShellError>;

    /// Show or hide a browser surface.
    fn set_visible(&self, pane_id: &str, visible: bool) -> Result<(), ShellError>;

    /// Close and destroy a browser surface.
    fn close_surface(&self, pane_id: &str) -> Result<(), ShellError>;

    /// Navigate an existing browser surface to a new URL.
    fn navigate(&self, pane_id: &str, url: &str) -> Result<(), ShellError>;
}

/// Port for executing Git operations against a repository on disk.
///
/// Infrastructure adapters implement this trait to decouple application services
/// from any specific Git backend (CLI, libgit2, etc.).
pub trait GitOperationsPort: Send + Sync + std::fmt::Debug {
    /// Return the status of all files in the repository (staged and unstaged).
    fn status(&self, repo_path: &Path) -> Result<Vec<FileStatus>, ShellError>;

    /// Return the diff for the repository (unstaged changes by default).
    fn diff(&self, repo_path: &Path, staged: bool) -> Result<Vec<DiffContent>, ShellError>;

    /// Stage one or more files by path.
    fn stage(&self, repo_path: &Path, paths: &[&str]) -> Result<(), ShellError>;

    /// Unstage one or more files by path.
    fn unstage(&self, repo_path: &Path, paths: &[&str]) -> Result<(), ShellError>;

    /// Stage specific line ranges within a file (partial/hunk staging).
    fn stage_lines(
        &self,
        repo_path: &Path,
        file_path: &str,
        line_ranges: &[(u32, u32)],
    ) -> Result<(), ShellError>;

    /// Create a commit with the given message. If `amend` is true, amend the previous commit.
    fn commit(
        &self,
        repo_path: &Path,
        message: &str,
        amend: bool,
    ) -> Result<CommitInfo, ShellError>;

    /// Push the current branch to the remote.
    fn push(
        &self,
        repo_path: &Path,
        remote: &RemoteName,
        branch: &BranchName,
    ) -> Result<(), ShellError>;

    /// Pull changes from the remote into the current branch.
    fn pull(
        &self,
        repo_path: &Path,
        remote: &RemoteName,
        branch: &BranchName,
    ) -> Result<(), ShellError>;

    /// Fetch refs from a remote without merging.
    fn fetch(&self, repo_path: &Path, remote: &RemoteName) -> Result<(), ShellError>;

    /// List all local branches.
    fn branches(&self, repo_path: &Path) -> Result<Vec<BranchInfo>, ShellError>;

    /// Check out an existing branch by name.
    fn checkout_branch(&self, repo_path: &Path, branch: &BranchName) -> Result<(), ShellError>;

    /// Create a new branch, optionally from a given start point instead of HEAD.
    fn create_branch(
        &self,
        repo_path: &Path,
        branch: &BranchName,
        start_point: Option<&BranchName>,
    ) -> Result<(), ShellError>;

    /// Delete a local branch. If `force` is true, uses `-D` instead of `-d`.
    fn delete_branch(
        &self,
        repo_path: &Path,
        branch: &BranchName,
        force: bool,
    ) -> Result<(), ShellError>;

    /// Merge another branch into the current branch.
    fn merge_branch(&self, repo_path: &Path, branch: &BranchName) -> Result<(), ShellError>;

    /// Return the commit log, limited to `max_count` entries, optionally skipping `skip` entries.
    fn log(
        &self,
        repo_path: &Path,
        max_count: u32,
        skip: u32,
    ) -> Result<Vec<CommitInfo>, ShellError>;

    /// Return the diff for a specific commit by hash.
    fn show_commit(&self, repo_path: &Path, hash: &str) -> Result<Vec<DiffContent>, ShellError>;

    /// Return blame information for a file.
    fn blame(&self, repo_path: &Path, file_path: &str) -> Result<Vec<BlameEntry>, ShellError>;

    /// Push the current worktree state onto the stash stack.
    fn stash_push(&self, repo_path: &Path, message: Option<&str>) -> Result<(), ShellError>;

    /// Pop the top stash entry and apply it to the worktree.
    fn stash_pop(&self, repo_path: &Path) -> Result<(), ShellError>;

    /// List all stash entries.
    fn stash_list(&self, repo_path: &Path) -> Result<Vec<StashEntry>, ShellError>;

    /// Drop a specific stash entry.
    fn stash_drop(&self, repo_path: &Path, stash_id: StashId) -> Result<(), ShellError>;

    /// Discard unstaged changes for the given file paths.
    fn discard_changes(&self, repo_path: &Path, paths: &[&str]) -> Result<(), ShellError>;

    /// Return the high-level repository state (HEAD branch, detached status, etc.).
    fn repo_state(&self, repo_path: &Path) -> Result<GitRepositoryState, ShellError>;
}
