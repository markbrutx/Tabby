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
