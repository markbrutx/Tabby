use serde::{Deserialize, Serialize};
use specta::Type;

use crate::cli::CliArgs;
use crate::domain::error::TabbyError;
use crate::domain::types::{AppSettings, LayoutPreset, SplitDirection};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    pub new_tab: bool,
    pub preset: LayoutPreset,
    pub cwd: Option<String>,
    pub profile_id: Option<String>,
    pub startup_command: Option<String>,
}

impl LaunchRequest {
    pub fn from_cli_args(cli_args: CliArgs, settings: &AppSettings) -> Result<Self, TabbyError> {
        let preset = match cli_args.layout.as_deref() {
            Some("1x1") => LayoutPreset::OneByOne,
            Some("1x2") => LayoutPreset::OneByTwo,
            Some("2x2") => LayoutPreset::TwoByTwo,
            Some("2x3") => LayoutPreset::TwoByThree,
            Some("3x3") => LayoutPreset::ThreeByThree,
            Some(other) => {
                return Err(TabbyError::Validation(format!(
                    "Unsupported layout override: {other}"
                )))
            }
            None => settings.default_layout,
        };

        Ok(Self {
            new_tab: cli_args.new_tab,
            preset,
            cwd: cli_args.cwd.or_else(|| {
                (!settings.default_working_directory.trim().is_empty())
                    .then(|| settings.default_working_directory.clone())
            }),
            profile_id: cli_args.profile.or_else(|| {
                (!settings.default_profile_id.trim().is_empty())
                    .then(|| settings.default_profile_id.clone())
            }),
            startup_command: cli_args.command.or_else(|| {
                (!settings.default_custom_command.trim().is_empty())
                    .then(|| settings.default_custom_command.clone())
            }),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct PaneConfig {
    pub profile_id: String,
    pub cwd: String,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct NewTabRequest {
    pub preset: LayoutPreset,
    pub cwd: Option<String>,
    pub profile_id: Option<String>,
    pub startup_command: Option<String>,
    #[serde(default)]
    pub pane_configs: Option<Vec<PaneConfig>>,
}

impl From<LaunchRequest> for NewTabRequest {
    fn from(value: LaunchRequest) -> Self {
        Self {
            preset: value.preset,
            cwd: value.cwd,
            profile_id: value.profile_id,
            startup_command: value.startup_command,
            pane_configs: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct SplitPaneRequest {
    pub pane_id: String,
    pub direction: SplitDirection,
    pub profile_id: Option<String>,
    pub startup_command: Option<String>,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePaneProfileRequest {
    pub pane_id: String,
    pub profile_id: String,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePaneCwdRequest {
    pub pane_id: String,
    pub cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct PtyResizeRequest {
    pub pane_id: String,
    pub cols: u16,
    pub rows: u16,
}
