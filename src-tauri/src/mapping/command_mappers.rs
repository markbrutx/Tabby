use tabby_contracts::{RuntimeCommandDto, SettingsCommandDto, WorkspaceCommandDto};
use tabby_settings::SettingsError;
use tabby_workspace::layout::LayoutPreset;
use tabby_workspace::{PaneId, TabId};

use crate::application::commands::{
    CloseTabCommand, OpenTabCommand, ReplacePaneSpecCommand, RuntimeCommand, SettingsCommand,
    SplitPaneCommand, UpdateSettingsCommand, WorkspaceCommand,
};

use super::settings_mappers::preferences_from_settings_view;
use super::workspace_mappers::{
    layout_preset_from_dto, pane_spec_from_dto, split_direction_from_dto, split_node_from_dto,
};

// ---------------------------------------------------------------------------
// DTO → Domain (inbound / commands)
// ---------------------------------------------------------------------------

pub(crate) fn workspace_command_from_dto(
    dto: WorkspaceCommandDto,
    default_layout: LayoutPreset,
) -> WorkspaceCommand {
    match dto {
        WorkspaceCommandDto::OpenTab {
            layout,
            auto_layout,
            layout_tree,
            pane_specs,
        } => {
            let layout = layout.map(layout_preset_from_dto).unwrap_or(default_layout);
            WorkspaceCommand::OpenTab(OpenTabCommand {
                layout,
                auto_layout,
                layout_tree: layout_tree.map(split_node_from_dto),
                pane_specs: pane_specs.into_iter().map(pane_spec_from_dto).collect(),
            })
        }
        WorkspaceCommandDto::CloseTab { tab_id } => WorkspaceCommand::CloseTab(CloseTabCommand {
            tab_id: TabId::from(tab_id),
        }),
        WorkspaceCommandDto::SetActiveTab { tab_id } => WorkspaceCommand::SetActiveTab {
            tab_id: TabId::from(tab_id),
        },
        WorkspaceCommandDto::FocusPane { tab_id, pane_id } => WorkspaceCommand::FocusPane {
            tab_id: TabId::from(tab_id),
            pane_id: PaneId::from(pane_id),
        },
        WorkspaceCommandDto::SplitPane {
            pane_id,
            direction,
            pane_spec,
        } => WorkspaceCommand::SplitPane(SplitPaneCommand {
            pane_id: PaneId::from(pane_id),
            direction: split_direction_from_dto(direction),
            spec: pane_spec_from_dto(pane_spec),
        }),
        WorkspaceCommandDto::ClosePane { pane_id } => WorkspaceCommand::ClosePane {
            pane_id: PaneId::from(pane_id),
        },
        WorkspaceCommandDto::SwapPaneSlots {
            pane_id_a,
            pane_id_b,
        } => WorkspaceCommand::SwapPaneSlots {
            pane_id_a: PaneId::from(pane_id_a),
            pane_id_b: PaneId::from(pane_id_b),
        },
        WorkspaceCommandDto::ReplacePaneSpec { pane_id, pane_spec } => {
            WorkspaceCommand::ReplacePaneSpec(ReplacePaneSpecCommand {
                pane_id: PaneId::from(pane_id),
                spec: pane_spec_from_dto(pane_spec),
            })
        }
        WorkspaceCommandDto::RestartPaneRuntime { pane_id } => {
            WorkspaceCommand::RestartPaneRuntime {
                pane_id: PaneId::from(pane_id),
            }
        }
        WorkspaceCommandDto::RenameTab { tab_id, title } => WorkspaceCommand::RenameTab {
            tab_id: TabId::from(tab_id),
            title,
        },
    }
}

pub(crate) fn settings_command_from_dto(
    dto: SettingsCommandDto,
) -> Result<SettingsCommand, SettingsError> {
    match dto {
        SettingsCommandDto::Update { settings } => {
            Ok(SettingsCommand::Update(UpdateSettingsCommand {
                preferences: preferences_from_settings_view(&settings)?,
            }))
        }
        SettingsCommandDto::Reset => Ok(SettingsCommand::Reset),
    }
}

pub(crate) fn runtime_command_from_dto(dto: RuntimeCommandDto) -> RuntimeCommand {
    match dto {
        RuntimeCommandDto::WriteTerminalInput { pane_id, input } => {
            RuntimeCommand::WriteTerminalInput {
                pane_id: PaneId::from(pane_id),
                input,
            }
        }
        RuntimeCommandDto::ResizeTerminal {
            pane_id,
            cols,
            rows,
        } => RuntimeCommand::ResizeTerminal {
            pane_id: PaneId::from(pane_id),
            cols,
            rows,
        },
        RuntimeCommandDto::NavigateBrowser { pane_id, url } => RuntimeCommand::NavigateBrowser {
            pane_id: PaneId::from(pane_id),
            url,
        },
        RuntimeCommandDto::ObserveTerminalCwd {
            pane_id,
            working_directory,
        } => RuntimeCommand::ObserveTerminalCwd {
            pane_id: PaneId::from(pane_id),
            working_directory,
        },
        RuntimeCommandDto::ObserveBrowserLocation { pane_id, url } => {
            RuntimeCommand::ObserveBrowserLocation {
                pane_id: PaneId::from(pane_id),
                url,
            }
        }
    }
}
