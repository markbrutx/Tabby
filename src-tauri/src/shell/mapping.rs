use tabby_contracts::{
    LayoutPresetDto, PaneRuntimeView, PaneSpecDto, PaneView, ProfileCatalogView, ProfileView,
    RuntimeKindDto, RuntimeStatusDto, SettingsView, SplitDirectionDto, SplitNodeDto, TabView,
    ThemeModeDto, WorkspaceView,
};
use tabby_runtime::{PaneRuntime, RuntimeKind, RuntimeStatus};
use tabby_settings::{ProfileCatalog, ThemeMode, UserPreferences};
use tabby_workspace::layout::{LayoutPreset, SplitDirection, SplitNode};
use tabby_workspace::{PaneSpec, WorkspaceSession};

pub fn settings_view_from_preferences(preferences: &UserPreferences) -> SettingsView {
    SettingsView {
        default_layout: match LayoutPreset::parse(&preferences.default_layout)
            .unwrap_or(LayoutPreset::OneByOne)
        {
            LayoutPreset::OneByOne => LayoutPresetDto::OneByOne,
            LayoutPreset::OneByTwo => LayoutPresetDto::OneByTwo,
            LayoutPreset::TwoByTwo => LayoutPresetDto::TwoByTwo,
            LayoutPreset::TwoByThree => LayoutPresetDto::TwoByThree,
            LayoutPreset::ThreeByThree => LayoutPresetDto::ThreeByThree,
        },
        default_terminal_profile_id: preferences.default_terminal_profile_id.clone(),
        default_working_directory: preferences.default_working_directory.clone(),
        default_custom_command: preferences.default_custom_command.clone(),
        font_size: preferences.font_size,
        theme: theme_mode_to_dto(preferences.theme),
        launch_fullscreen: preferences.launch_fullscreen,
        has_completed_onboarding: preferences.has_completed_onboarding,
        last_working_directory: preferences.last_working_directory.clone(),
    }
}

pub fn preferences_from_settings_view(view: &SettingsView) -> UserPreferences {
    UserPreferences {
        default_layout: match view.default_layout {
            LayoutPresetDto::OneByOne => String::from("1x1"),
            LayoutPresetDto::OneByTwo => String::from("1x2"),
            LayoutPresetDto::TwoByTwo => String::from("2x2"),
            LayoutPresetDto::TwoByThree => String::from("2x3"),
            LayoutPresetDto::ThreeByThree => String::from("3x3"),
        },
        default_terminal_profile_id: view.default_terminal_profile_id.clone(),
        default_working_directory: view.default_working_directory.clone(),
        default_custom_command: view.default_custom_command.clone(),
        font_size: view.font_size,
        theme: theme_mode_from_dto(view.theme),
        launch_fullscreen: view.launch_fullscreen,
        has_completed_onboarding: view.has_completed_onboarding,
        last_working_directory: view.last_working_directory.clone(),
    }
}

pub fn profile_catalog_view_from_catalog(catalog: &ProfileCatalog) -> ProfileCatalogView {
    ProfileCatalogView {
        terminal_profiles: catalog
            .terminal_profiles
            .iter()
            .map(|profile| ProfileView {
                id: profile.id.clone(),
                label: profile.label.clone(),
                description: profile.description.clone(),
                startup_command_template: profile.startup_command_template.clone(),
            })
            .collect(),
    }
}

pub fn workspace_view_from_session(session: &WorkspaceSession) -> WorkspaceView {
    WorkspaceView {
        active_tab_id: session.active_tab_id.clone().unwrap_or_default(),
        tabs: session
            .tab_summaries()
            .iter()
            .map(|tab| TabView {
                tab_id: tab.tab_id.clone(),
                title: tab.title.clone(),
                layout: split_node_to_dto(&tab.layout),
                panes: tab
                    .panes
                    .iter()
                    .map(|pane| PaneView {
                        pane_id: pane.pane_id.clone(),
                        title: pane.title.clone(),
                        spec: pane_spec_to_dto(&pane.spec),
                    })
                    .collect(),
                active_pane_id: tab.active_pane_id.clone(),
            })
            .collect(),
    }
}

pub fn pane_spec_from_dto(value: PaneSpecDto) -> PaneSpec {
    match value {
        PaneSpecDto::Terminal {
            launch_profile_id,
            working_directory,
            command_override,
        } => PaneSpec::Terminal(tabby_workspace::TerminalPaneSpec {
            launch_profile_id,
            working_directory,
            command_override,
        }),
        PaneSpecDto::Browser { initial_url } => {
            PaneSpec::Browser(tabby_workspace::BrowserPaneSpec { initial_url })
        }
    }
}

pub fn pane_spec_to_dto(value: &PaneSpec) -> PaneSpecDto {
    match value {
        PaneSpec::Terminal(spec) => PaneSpecDto::Terminal {
            launch_profile_id: spec.launch_profile_id.clone(),
            working_directory: spec.working_directory.clone(),
            command_override: spec.command_override.clone(),
        },
        PaneSpec::Browser(spec) => PaneSpecDto::Browser {
            initial_url: spec.initial_url.clone(),
        },
    }
}

pub fn layout_preset_from_dto(value: LayoutPresetDto) -> LayoutPreset {
    match value {
        LayoutPresetDto::OneByOne => LayoutPreset::OneByOne,
        LayoutPresetDto::OneByTwo => LayoutPreset::OneByTwo,
        LayoutPresetDto::TwoByTwo => LayoutPreset::TwoByTwo,
        LayoutPresetDto::TwoByThree => LayoutPreset::TwoByThree,
        LayoutPresetDto::ThreeByThree => LayoutPreset::ThreeByThree,
    }
}

pub fn split_direction_from_dto(value: SplitDirectionDto) -> SplitDirection {
    match value {
        SplitDirectionDto::Horizontal => SplitDirection::Horizontal,
        SplitDirectionDto::Vertical => SplitDirection::Vertical,
    }
}

pub fn pane_runtime_to_view(runtime: &PaneRuntime) -> PaneRuntimeView {
    PaneRuntimeView {
        pane_id: runtime.pane_id.clone(),
        runtime_session_id: runtime.runtime_session_id.clone(),
        kind: match runtime.kind {
            RuntimeKind::Terminal => RuntimeKindDto::Terminal,
            RuntimeKind::Browser => RuntimeKindDto::Browser,
        },
        status: match runtime.status {
            RuntimeStatus::Starting => RuntimeStatusDto::Starting,
            RuntimeStatus::Running => RuntimeStatusDto::Running,
            RuntimeStatus::Exited => RuntimeStatusDto::Exited,
            RuntimeStatus::Failed => RuntimeStatusDto::Failed,
        },
        last_error: runtime.last_error.clone(),
        browser_location: runtime.browser_location.clone(),
    }
}

fn theme_mode_to_dto(value: ThemeMode) -> ThemeModeDto {
    match value {
        ThemeMode::System => ThemeModeDto::System,
        ThemeMode::Dawn => ThemeModeDto::Dawn,
        ThemeMode::Midnight => ThemeModeDto::Midnight,
    }
}

fn theme_mode_from_dto(value: ThemeModeDto) -> ThemeMode {
    match value {
        ThemeModeDto::System => ThemeMode::System,
        ThemeModeDto::Dawn => ThemeMode::Dawn,
        ThemeModeDto::Midnight => ThemeMode::Midnight,
    }
}

fn split_node_to_dto(value: &SplitNode) -> SplitNodeDto {
    match value {
        SplitNode::Pane { pane_id } => SplitNodeDto::Pane {
            pane_id: pane_id.clone(),
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
