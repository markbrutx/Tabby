use std::path::PathBuf;

use tabby_git::value_objects::{BranchName, RemoteName, StashId};
use tabby_git::{
    BlameEntry, BranchInfo, CommitInfo, DiffContent, FileStatus, GitRepositoryState, StashEntry,
};
use tabby_settings::UserPreferences;
use tabby_workspace::layout::{LayoutPreset, SplitDirection};
use tabby_workspace::{PaneId, PaneSpec, TabId};

// ---------------------------------------------------------------------------
// Workspace commands
// ---------------------------------------------------------------------------

/// Internal command to open a new tab with the given layout and pane specifications.
#[derive(Debug, Clone)]
pub struct OpenTabCommand {
    pub layout: LayoutPreset,
    pub auto_layout: bool,
    pub pane_specs: Vec<PaneSpec>,
}

/// Internal command to close an existing tab.
#[derive(Debug, Clone)]
pub struct CloseTabCommand {
    pub tab_id: TabId,
}

/// Internal command to split an existing pane in the given direction.
#[derive(Debug, Clone)]
pub struct SplitPaneCommand {
    pub pane_id: PaneId,
    pub direction: SplitDirection,
    pub spec: PaneSpec,
}

/// Internal command to replace the spec of an existing pane (e.g. swap terminal for browser).
#[derive(Debug, Clone)]
pub struct ReplacePaneSpecCommand {
    pub pane_id: PaneId,
    pub spec: PaneSpec,
}

/// All workspace commands accepted by the application layer.
#[derive(Debug, Clone)]
pub enum WorkspaceCommand {
    OpenTab(OpenTabCommand),
    CloseTab(CloseTabCommand),
    SetActiveTab {
        tab_id: TabId,
    },
    FocusPane {
        tab_id: TabId,
        pane_id: PaneId,
    },
    SplitPane(SplitPaneCommand),
    ClosePane {
        pane_id: PaneId,
    },
    SwapPaneSlots {
        pane_id_a: PaneId,
        pane_id_b: PaneId,
    },
    ReplacePaneSpec(ReplacePaneSpecCommand),
    RestartPaneRuntime {
        pane_id: PaneId,
    },
}

// ---------------------------------------------------------------------------
// Settings commands
// ---------------------------------------------------------------------------

/// Internal command to update user preferences.
#[derive(Debug, Clone)]
pub struct UpdateSettingsCommand {
    pub preferences: UserPreferences,
}

/// All settings commands accepted by the application layer.
#[derive(Debug, Clone)]
pub enum SettingsCommand {
    Update(UpdateSettingsCommand),
    Reset,
}

// ---------------------------------------------------------------------------
// Runtime commands
// ---------------------------------------------------------------------------

/// All runtime commands accepted by the application layer.
#[derive(Debug, Clone)]
pub enum RuntimeCommand {
    WriteTerminalInput {
        pane_id: PaneId,
        input: String,
    },
    ResizeTerminal {
        pane_id: PaneId,
        cols: u16,
        rows: u16,
    },
    NavigateBrowser {
        pane_id: PaneId,
        url: String,
    },
    ObserveTerminalCwd {
        pane_id: PaneId,
        working_directory: String,
    },
    ObserveBrowserLocation {
        pane_id: PaneId,
        url: String,
    },
}

// ---------------------------------------------------------------------------
// Git commands
// ---------------------------------------------------------------------------

/// All Git commands accepted by the application layer.
///
/// Each variant carries the resolved `repo_path` (not the pane ID) so that
/// the application service is decoupled from pane-to-repo resolution logic.
#[derive(Debug, Clone)]
pub enum GitCommand {
    Status {
        repo_path: PathBuf,
    },
    Diff {
        repo_path: PathBuf,
        staged: bool,
    },
    Stage {
        repo_path: PathBuf,
        paths: Vec<String>,
    },
    Unstage {
        repo_path: PathBuf,
        paths: Vec<String>,
    },
    StageLines {
        repo_path: PathBuf,
        file_path: String,
        line_ranges: Vec<(u32, u32)>,
    },
    Commit {
        repo_path: PathBuf,
        message: String,
        amend: bool,
    },
    Push {
        repo_path: PathBuf,
        remote: RemoteName,
        branch: BranchName,
    },
    Pull {
        repo_path: PathBuf,
        remote: RemoteName,
        branch: BranchName,
    },
    Fetch {
        repo_path: PathBuf,
        remote: RemoteName,
    },
    Branches {
        repo_path: PathBuf,
    },
    CheckoutBranch {
        repo_path: PathBuf,
        branch: BranchName,
    },
    CreateBranch {
        repo_path: PathBuf,
        branch: BranchName,
        start_point: Option<BranchName>,
    },
    DeleteBranch {
        repo_path: PathBuf,
        branch: BranchName,
        force: bool,
    },
    MergeBranch {
        repo_path: PathBuf,
        branch: BranchName,
    },
    Log {
        repo_path: PathBuf,
        max_count: u32,
    },
    Blame {
        repo_path: PathBuf,
        file_path: String,
    },
    StashPush {
        repo_path: PathBuf,
        message: Option<String>,
    },
    StashPop {
        repo_path: PathBuf,
    },
    StashList {
        repo_path: PathBuf,
    },
    StashDrop {
        repo_path: PathBuf,
        stash_id: StashId,
    },
    DiscardChanges {
        repo_path: PathBuf,
        paths: Vec<String>,
    },
    RepoState {
        repo_path: PathBuf,
    },
}

/// Result variants returned by `GitApplicationService::dispatch_command`.
///
/// Each variant wraps the domain types returned by `GitOperationsPort` methods.
#[derive(Debug)]
pub enum GitResult {
    Status(Vec<FileStatus>),
    Diff(Vec<DiffContent>),
    Stage,
    Unstage,
    StageLines,
    Commit(CommitInfo),
    Push,
    Pull,
    Fetch,
    Branches(Vec<BranchInfo>),
    CheckoutBranch,
    CreateBranch,
    DeleteBranch,
    MergeBranch,
    Log(Vec<CommitInfo>),
    Blame(Vec<BlameEntry>),
    StashPush,
    StashPop,
    StashList(Vec<StashEntry>),
    StashDrop,
    DiscardChanges,
    RepoState(GitRepositoryState),
}
