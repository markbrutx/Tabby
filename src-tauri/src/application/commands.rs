use tabby_settings::UserPreferences;
use tabby_workspace::layout::{LayoutPreset, SplitDirection};
use tabby_workspace::PaneSpec;

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
    pub tab_id: String,
}

/// Internal command to split an existing pane in the given direction.
#[derive(Debug, Clone)]
pub struct SplitPaneCommand {
    pub pane_id: String,
    pub direction: SplitDirection,
    pub spec: PaneSpec,
}

/// Internal command to replace the spec of an existing pane (e.g. swap terminal for browser).
#[derive(Debug, Clone)]
pub struct ReplacePaneSpecCommand {
    pub pane_id: String,
    pub spec: PaneSpec,
}

/// All workspace commands accepted by the application layer.
#[derive(Debug, Clone)]
pub enum WorkspaceCommand {
    OpenTab(OpenTabCommand),
    CloseTab(CloseTabCommand),
    SetActiveTab {
        tab_id: String,
    },
    FocusPane {
        tab_id: String,
        pane_id: String,
    },
    SplitPane(SplitPaneCommand),
    ClosePane {
        pane_id: String,
    },
    SwapPaneSlots {
        pane_id_a: String,
        pane_id_b: String,
    },
    ReplacePaneSpec(ReplacePaneSpecCommand),
    RestartPaneRuntime {
        pane_id: String,
    },
    TrackTerminalWorkingDirectory {
        pane_id: String,
        working_directory: String,
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
        pane_id: String,
        input: String,
    },
    ResizeTerminal {
        pane_id: String,
        cols: u16,
        rows: u16,
    },
    NavigateBrowser {
        pane_id: String,
        url: String,
    },
}
