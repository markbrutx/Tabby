use tabby_contracts::{
    LayoutPresetDto, PaneSpecDto, PaneView, SplitDirectionDto, SplitNodeDto, TabView,
    WorkspaceBootstrapView, WorkspaceView,
};
use tabby_runtime::PaneRuntime;
use tabby_settings::{ProfileCatalog, UserPreferences};
use tabby_workspace::layout::{LayoutPreset, SplitDirection, SplitNode};
use tabby_workspace::{PaneContentDefinition, PaneId, PaneSpec, WorkspaceSession};

use super::runtime_mappers::pane_runtime_to_view;
use super::settings_mappers::{profile_catalog_view_from_catalog, settings_view_from_preferences};

// ---------------------------------------------------------------------------
// Domain → DTO (outbound / projections)
// ---------------------------------------------------------------------------

pub(crate) fn workspace_view_from_session(session: &WorkspaceSession) -> WorkspaceView {
    WorkspaceView {
        active_tab_id: session
            .active_tab_id
            .as_ref()
            .map(|id| id.to_string())
            .unwrap_or_default(),
        tabs: session
            .tab_summaries()
            .iter()
            .map(|tab| TabView {
                tab_id: tab.tab_id.to_string(),
                title: tab.title.clone(),
                layout: split_node_to_dto(&tab.layout),
                panes: tab
                    .panes
                    .iter()
                    .map(|pane| {
                        let spec_dto = session
                            .pane_content(&pane.content_id)
                            .map(pane_content_to_spec_dto)
                            .unwrap_or_else(|| PaneSpecDto::Terminal {
                                launch_profile_id: String::new(),
                                working_directory: String::new(),
                                command_override: None,
                            });
                        PaneView {
                            pane_id: pane.pane_id.to_string(),
                            title: pane.title.clone(),
                            spec: spec_dto,
                        }
                    })
                    .collect(),
                active_pane_id: tab.active_pane_id.to_string(),
            })
            .collect(),
    }
}

pub(crate) fn pane_content_to_spec_dto(content: &PaneContentDefinition) -> PaneSpecDto {
    match content {
        PaneContentDefinition::Terminal {
            profile_id,
            working_directory,
            command_override,
            ..
        } => PaneSpecDto::Terminal {
            launch_profile_id: profile_id.clone(),
            working_directory: working_directory.clone(),
            command_override: command_override.as_ref().map(|c| c.as_str().to_string()),
        },
        PaneContentDefinition::Browser { initial_url, .. } => PaneSpecDto::Browser {
            initial_url: initial_url.as_str().to_string(),
        },
        PaneContentDefinition::Git {
            working_directory, ..
        } => PaneSpecDto::Git {
            working_directory: working_directory.clone(),
        },
    }
}

#[cfg(test)]
pub(crate) fn pane_spec_to_dto(value: &PaneSpec) -> PaneSpecDto {
    match value {
        PaneSpec::Terminal(spec) => PaneSpecDto::Terminal {
            launch_profile_id: spec.launch_profile_id.clone(),
            working_directory: spec.working_directory.clone(),
            command_override: spec
                .command_override
                .as_ref()
                .map(|c| c.as_str().to_string()),
        },
        PaneSpec::Browser(spec) => PaneSpecDto::Browser {
            initial_url: spec.initial_url.as_str().to_string(),
        },
        PaneSpec::Git(spec) => PaneSpecDto::Git {
            working_directory: spec.working_directory.clone(),
        },
    }
}

pub(crate) fn bootstrap_view(
    session: &WorkspaceSession,
    preferences: &UserPreferences,
    catalog: &ProfileCatalog,
    runtimes: &[PaneRuntime],
) -> WorkspaceBootstrapView {
    WorkspaceBootstrapView {
        workspace: workspace_view_from_session(session),
        settings: settings_view_from_preferences(preferences),
        profile_catalog: profile_catalog_view_from_catalog(catalog),
        runtime_projections: runtimes.iter().map(pane_runtime_to_view).collect(),
    }
}

// ---------------------------------------------------------------------------
// DTO → Domain (inbound)
// ---------------------------------------------------------------------------

pub(crate) fn pane_spec_from_dto(value: PaneSpecDto) -> PaneSpec {
    match value {
        PaneSpecDto::Terminal {
            launch_profile_id,
            working_directory,
            command_override,
        } => PaneSpec::Terminal(tabby_workspace::TerminalPaneSpec {
            launch_profile_id,
            working_directory,
            command_override: command_override
                .filter(|s| !s.trim().is_empty())
                .map(tabby_workspace::CommandTemplate::new),
        }),
        PaneSpecDto::Browser { initial_url } => {
            PaneSpec::Browser(tabby_workspace::BrowserPaneSpec {
                initial_url: tabby_workspace::BrowserUrl::new(initial_url),
            })
        }
        PaneSpecDto::Git { working_directory } => {
            PaneSpec::Git(tabby_workspace::GitPaneSpec { working_directory })
        }
    }
}

pub(crate) fn layout_preset_from_dto(value: LayoutPresetDto) -> LayoutPreset {
    match value {
        LayoutPresetDto::OneByOne => LayoutPreset::OneByOne,
        LayoutPresetDto::OneByTwo => LayoutPreset::OneByTwo,
        LayoutPresetDto::TwoByTwo => LayoutPreset::TwoByTwo,
        LayoutPresetDto::TwoByThree => LayoutPreset::TwoByThree,
        LayoutPresetDto::ThreeByThree => LayoutPreset::ThreeByThree,
    }
}

pub(crate) fn layout_preset_to_dto(value: LayoutPreset) -> LayoutPresetDto {
    match value {
        LayoutPreset::OneByOne => LayoutPresetDto::OneByOne,
        LayoutPreset::OneByTwo => LayoutPresetDto::OneByTwo,
        LayoutPreset::TwoByTwo => LayoutPresetDto::TwoByTwo,
        LayoutPreset::TwoByThree => LayoutPresetDto::TwoByThree,
        LayoutPreset::ThreeByThree => LayoutPresetDto::ThreeByThree,
    }
}

pub(crate) fn split_direction_from_dto(value: SplitDirectionDto) -> SplitDirection {
    match value {
        SplitDirectionDto::Horizontal => SplitDirection::Horizontal,
        SplitDirectionDto::Vertical => SplitDirection::Vertical,
    }
}

pub(crate) fn split_node_from_dto(dto: SplitNodeDto) -> SplitNode {
    match dto {
        SplitNodeDto::Pane { pane_id } => SplitNode::Pane {
            pane_id: PaneId::from(pane_id),
        },
        SplitNodeDto::Split {
            direction,
            ratio,
            first,
            second,
        } => SplitNode::Split {
            direction: split_direction_from_dto(direction),
            ratio,
            first: Box::new(split_node_from_dto(*first)),
            second: Box::new(split_node_from_dto(*second)),
        },
    }
}

pub(crate) fn split_node_to_dto(value: &SplitNode) -> SplitNodeDto {
    match value {
        SplitNode::Pane { pane_id } => SplitNodeDto::Pane {
            pane_id: pane_id.to_string(),
        },
        SplitNode::Split {
            direction,
            ratio,
            first,
            second,
        } => SplitNodeDto::Split {
            direction: match direction {
                SplitDirection::Horizontal => SplitDirectionDto::Horizontal,
                SplitDirection::Vertical => SplitDirectionDto::Vertical,
            },
            ratio: *ratio,
            first: Box::new(split_node_to_dto(first)),
            second: Box::new(split_node_to_dto(second)),
        },
    }
}
