mod value_objects;

pub use value_objects::{
    BrowserUrl, CommandTemplate, LayoutPreset, PaneId, TabId, ValueObjectError, WorkingDirectory,
};

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ThemeModeDto {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "dawn")]
    Dawn,
    #[serde(rename = "midnight")]
    Midnight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum LayoutPresetDto {
    #[serde(rename = "1x1")]
    OneByOne,
    #[serde(rename = "1x2")]
    OneByTwo,
    #[serde(rename = "2x2")]
    TwoByTwo,
    #[serde(rename = "2x3")]
    TwoByThree,
    #[serde(rename = "3x3")]
    ThreeByThree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SplitDirectionDto {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SplitNodeDto {
    Pane {
        #[serde(rename = "paneId")]
        pane_id: String,
    },
    Split {
        direction: SplitDirectionDto,
        ratio: u16,
        first: Box<SplitNodeDto>,
        second: Box<SplitNodeDto>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PaneSpecDto {
    Terminal {
        launch_profile_id: String,
        working_directory: String,
        command_override: Option<String>,
    },
    Browser {
        initial_url: String,
    },
    Git {
        working_directory: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PaneView {
    pub pane_id: String,
    pub title: String,
    pub spec: PaneSpecDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TabView {
    pub tab_id: String,
    pub title: String,
    pub layout: SplitNodeDto,
    pub panes: Vec<PaneView>,
    pub active_pane_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceView {
    pub active_tab_id: String,
    pub tabs: Vec<TabView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView {
    pub id: String,
    pub label: String,
    pub description: String,
    pub startup_command_template: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCatalogView {
    pub terminal_profiles: Vec<ProfileView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub default_layout: LayoutPresetDto,
    pub default_terminal_profile_id: String,
    pub default_working_directory: String,
    pub default_custom_command: String,
    pub font_size: u16,
    pub theme: ThemeModeDto,
    pub launch_fullscreen: bool,
    pub has_completed_onboarding: bool,
    pub last_working_directory: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeKindDto {
    Terminal,
    Browser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeStatusDto {
    Starting,
    Running,
    Exited,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PaneRuntimeView {
    pub pane_id: String,
    pub runtime_session_id: Option<String>,
    pub kind: RuntimeKindDto,
    pub status: RuntimeStatusDto,
    pub last_error: Option<String>,
    pub browser_location: Option<String>,
    pub terminal_cwd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBootstrapView {
    pub workspace: WorkspaceView,
    pub settings: SettingsView,
    pub profile_catalog: ProfileCatalogView,
    pub runtime_projections: Vec<PaneRuntimeView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WorkspaceCommandDto {
    OpenTab {
        layout: Option<LayoutPresetDto>,
        auto_layout: bool,
        pane_specs: Vec<PaneSpecDto>,
    },
    CloseTab {
        tab_id: String,
    },
    SetActiveTab {
        tab_id: String,
    },
    FocusPane {
        tab_id: String,
        pane_id: String,
    },
    SplitPane {
        pane_id: String,
        direction: SplitDirectionDto,
        pane_spec: PaneSpecDto,
    },
    ClosePane {
        pane_id: String,
    },
    SwapPaneSlots {
        pane_id_a: String,
        pane_id_b: String,
    },
    ReplacePaneSpec {
        pane_id: String,
        pane_spec: PaneSpecDto,
    },
    RestartPaneRuntime {
        pane_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SettingsCommandDto {
    Update { settings: SettingsView },
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RuntimeCommandDto {
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
    ObserveTerminalCwd {
        pane_id: String,
        working_directory: String,
    },
    ObserveBrowserLocation {
        pane_id: String,
        url: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSurfaceBoundsDto {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BrowserSurfaceCommandDto {
    Ensure {
        pane_id: String,
        url: String,
        bounds: BrowserSurfaceBoundsDto,
    },
    SetBounds {
        pane_id: String,
        bounds: BrowserSurfaceBoundsDto,
    },
    SetVisible {
        pane_id: String,
        visible: bool,
    },
    Close {
        pane_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutputEvent {
    pub pane_id: String,
    pub runtime_session_id: String,
    pub chunk: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatusChangedEvent {
    pub runtime: PaneRuntimeView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceProjectionUpdatedEvent {
    pub workspace: WorkspaceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsProjectionUpdatedEvent {
    pub settings: SettingsView,
    pub profile_catalog: ProfileCatalogView,
}
