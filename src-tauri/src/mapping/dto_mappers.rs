use tabby_contracts::{
    LayoutPresetDto, PaneRuntimeView, PaneSpecDto, PaneView, ProfileCatalogView, ProfileView,
    RuntimeCommandDto, RuntimeKindDto, RuntimeStatusDto, SettingsCommandDto, SettingsView,
    SplitDirectionDto, SplitNodeDto, TabView, ThemeModeDto, WorkspaceBootstrapView,
    WorkspaceCommandDto, WorkspaceView,
};
use tabby_runtime::{PaneRuntime, RuntimeKind, RuntimeStatus};
use tabby_settings::{
    FontSize, ProfileCatalog, ProfileId, SettingsError, ThemeMode, UserPreferences,
    WorkingDirectory,
};
use tabby_workspace::layout::{LayoutPreset, SplitDirection, SplitNode};
use tabby_workspace::{PaneContentDefinition, PaneId, PaneSpec, TabId, WorkspaceSession};

use crate::application::commands::{
    CloseTabCommand, OpenTabCommand, ReplacePaneSpecCommand, RuntimeCommand, SettingsCommand,
    SplitPaneCommand, UpdateSettingsCommand, WorkspaceCommand,
};

// ---------------------------------------------------------------------------
// Domain → DTO (outbound / projections)
// ---------------------------------------------------------------------------

pub fn settings_view_from_preferences(preferences: &UserPreferences) -> SettingsView {
    SettingsView {
        default_layout: layout_preset_to_dto(
            LayoutPreset::parse(&preferences.default_layout).unwrap_or(LayoutPreset::OneByOne),
        ),
        default_terminal_profile_id: preferences.default_terminal_profile_id.as_str().to_string(),
        default_working_directory: preferences.default_working_directory.as_str().to_string(),
        default_custom_command: preferences.default_custom_command.clone(),
        font_size: preferences.font_size.value(),
        theme: theme_mode_to_dto(preferences.theme),
        launch_fullscreen: preferences.launch_fullscreen,
        has_completed_onboarding: preferences.has_completed_onboarding,
        last_working_directory: preferences.last_working_directory.clone(),
    }
}

pub fn preferences_from_settings_view(
    view: &SettingsView,
) -> Result<UserPreferences, SettingsError> {
    Ok(UserPreferences {
        default_layout: layout_preset_to_string(view.default_layout),
        default_terminal_profile_id: ProfileId::new(view.default_terminal_profile_id.clone()),
        default_working_directory: WorkingDirectory::new(view.default_working_directory.clone())?,
        default_custom_command: view.default_custom_command.clone(),
        font_size: FontSize::new(view.font_size)?,
        theme: theme_mode_from_dto(view.theme),
        launch_fullscreen: view.launch_fullscreen,
        has_completed_onboarding: view.has_completed_onboarding,
        last_working_directory: view.last_working_directory.clone(),
    })
}

pub fn profile_catalog_view_from_catalog(catalog: &ProfileCatalog) -> ProfileCatalogView {
    ProfileCatalogView {
        terminal_profiles: catalog
            .terminal_profiles
            .iter()
            .map(|profile| ProfileView {
                id: profile.id.as_str().to_string(),
                label: profile.label.clone(),
                description: profile.description.clone(),
                startup_command_template: profile.startup_command_template.clone(),
            })
            .collect(),
    }
}

pub fn workspace_view_from_session(session: &WorkspaceSession) -> WorkspaceView {
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

fn pane_content_to_spec_dto(content: &PaneContentDefinition) -> PaneSpecDto {
    match content {
        PaneContentDefinition::Terminal {
            profile_id,
            working_directory,
            command_override,
            ..
        } => PaneSpecDto::Terminal {
            launch_profile_id: profile_id.clone(),
            working_directory: working_directory.clone(),
            command_override: command_override.clone(),
        },
        PaneContentDefinition::Browser { initial_url, .. } => PaneSpecDto::Browser {
            initial_url: initial_url.as_str().to_string(),
        },
    }
}

#[cfg(test)]
fn pane_spec_to_dto(value: &PaneSpec) -> PaneSpecDto {
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

pub fn pane_runtime_to_view(runtime: &PaneRuntime) -> PaneRuntimeView {
    PaneRuntimeView {
        pane_id: runtime.pane_id.clone(),
        runtime_session_id: runtime.runtime_session_id.as_ref().map(|id| id.to_string()),
        kind: runtime_kind_to_dto(runtime.kind),
        status: runtime_status_to_dto(runtime.status),
        last_error: runtime.last_error.clone(),
        browser_location: runtime.browser_location.clone(),
    }
}

pub fn bootstrap_view(
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
// DTO → Domain (inbound / commands)
// ---------------------------------------------------------------------------

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

pub fn workspace_command_from_dto(
    dto: WorkspaceCommandDto,
    default_layout: LayoutPreset,
) -> WorkspaceCommand {
    match dto {
        WorkspaceCommandDto::OpenTab {
            layout,
            auto_layout,
            pane_specs,
        } => {
            let layout = layout.map(layout_preset_from_dto).unwrap_or(default_layout);
            WorkspaceCommand::OpenTab(OpenTabCommand {
                layout,
                auto_layout,
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
    }
}

pub fn settings_command_from_dto(
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

pub fn runtime_command_from_dto(dto: RuntimeCommandDto) -> RuntimeCommand {
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

// ---------------------------------------------------------------------------
// Persistence helpers — keep DTO knowledge in the mapping layer
// ---------------------------------------------------------------------------

pub fn serialize_preferences(
    preferences: &UserPreferences,
) -> Result<serde_json::Value, serde_json::Error> {
    let view = settings_view_from_preferences(preferences);
    serde_json::to_value(view)
}

pub fn deserialize_preferences(value: serde_json::Value) -> Result<UserPreferences, SettingsError> {
    let view: SettingsView =
        serde_json::from_value(value).map_err(|e| SettingsError::Validation(e.to_string()))?;
    preferences_from_settings_view(&view)
}

// ---------------------------------------------------------------------------
// Internal conversion helpers
// ---------------------------------------------------------------------------

fn layout_preset_to_dto(value: LayoutPreset) -> LayoutPresetDto {
    match value {
        LayoutPreset::OneByOne => LayoutPresetDto::OneByOne,
        LayoutPreset::OneByTwo => LayoutPresetDto::OneByTwo,
        LayoutPreset::TwoByTwo => LayoutPresetDto::TwoByTwo,
        LayoutPreset::TwoByThree => LayoutPresetDto::TwoByThree,
        LayoutPreset::ThreeByThree => LayoutPresetDto::ThreeByThree,
    }
}

fn layout_preset_to_string(value: LayoutPresetDto) -> String {
    match value {
        LayoutPresetDto::OneByOne => String::from("1x1"),
        LayoutPresetDto::OneByTwo => String::from("1x2"),
        LayoutPresetDto::TwoByTwo => String::from("2x2"),
        LayoutPresetDto::TwoByThree => String::from("2x3"),
        LayoutPresetDto::ThreeByThree => String::from("3x3"),
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

fn runtime_kind_to_dto(value: RuntimeKind) -> RuntimeKindDto {
    match value {
        RuntimeKind::Terminal => RuntimeKindDto::Terminal,
        RuntimeKind::Browser => RuntimeKindDto::Browser,
    }
}

fn runtime_status_to_dto(value: RuntimeStatus) -> RuntimeStatusDto {
    match value {
        RuntimeStatus::Starting => RuntimeStatusDto::Starting,
        RuntimeStatus::Running => RuntimeStatusDto::Running,
        RuntimeStatus::Exited => RuntimeStatusDto::Exited,
        RuntimeStatus::Failed => RuntimeStatusDto::Failed,
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tabby_runtime::{PaneRuntime, RuntimeKind, RuntimeSessionId, RuntimeStatus};
    use tabby_settings::{default_preferences, ProfileCatalog, TerminalProfile, UserPreferences};
    use tabby_workspace::{BrowserPaneSpec, PaneSpec, TerminalPaneSpec};

    // -- PaneSpec round-trip ------------------------------------------------

    #[test]
    fn terminal_pane_spec_round_trips_through_dto() {
        let spec = PaneSpec::Terminal(TerminalPaneSpec {
            launch_profile_id: String::from("claude"),
            working_directory: String::from("/home/user"),
            command_override: Some(String::from("bash")),
        });

        let dto = pane_spec_to_dto(&spec);
        let restored = pane_spec_from_dto(dto);

        match restored {
            PaneSpec::Terminal(t) => {
                assert_eq!(t.launch_profile_id, "claude");
                assert_eq!(t.working_directory, "/home/user");
                assert_eq!(t.command_override.as_deref(), Some("bash"));
            }
            PaneSpec::Browser(_) => panic!("Expected Terminal spec"),
        }
    }

    #[test]
    fn browser_pane_spec_round_trips_through_dto() {
        let spec = PaneSpec::Browser(BrowserPaneSpec {
            initial_url: String::from("https://example.com"),
        });

        let dto = pane_spec_to_dto(&spec);
        let restored = pane_spec_from_dto(dto);

        match restored {
            PaneSpec::Browser(b) => {
                assert_eq!(b.initial_url, "https://example.com");
            }
            PaneSpec::Terminal(_) => panic!("Expected Browser spec"),
        }
    }

    // -- SettingsView <-> UserPreferences round-trip ------------------------

    #[test]
    fn settings_round_trip_preserves_all_fields() {
        let preferences = UserPreferences {
            default_layout: String::from("2x2"),
            default_terminal_profile_id: ProfileId::new("claude"),
            default_working_directory: WorkingDirectory::new("/tmp").expect("valid path"),
            default_custom_command: String::from("fish"),
            font_size: FontSize::new(16).expect("valid size"),
            theme: ThemeMode::Dawn,
            launch_fullscreen: true,
            has_completed_onboarding: true,
            last_working_directory: Some(String::from("/home")),
        };

        let view = settings_view_from_preferences(&preferences);
        let restored = preferences_from_settings_view(&view).expect("should round-trip");

        assert_eq!(restored.default_layout, "2x2");
        assert_eq!(restored.default_terminal_profile_id, "claude");
        assert_eq!(restored.default_working_directory.as_str(), "/tmp");
        assert_eq!(restored.default_custom_command, "fish");
        assert_eq!(restored.font_size.value(), 16);
        assert!(matches!(restored.theme, ThemeMode::Dawn));
        assert!(restored.launch_fullscreen);
        assert!(restored.has_completed_onboarding);
        assert_eq!(restored.last_working_directory.as_deref(), Some("/home"));
    }

    #[test]
    fn settings_round_trip_with_defaults() {
        let defaults = default_preferences();
        let view = settings_view_from_preferences(&defaults);
        let restored = preferences_from_settings_view(&view).expect("should round-trip");

        assert_eq!(restored.default_layout, defaults.default_layout);
        assert_eq!(
            restored.default_terminal_profile_id,
            defaults.default_terminal_profile_id
        );
        assert_eq!(restored.font_size, defaults.font_size);
    }

    #[test]
    fn preferences_from_settings_view_rejects_invalid_font_size() {
        let mut view = settings_view_from_preferences(&default_preferences());
        view.font_size = 6;
        let err = preferences_from_settings_view(&view).expect_err("should reject font size 6");
        assert!(err.to_string().contains("Font size"));
    }

    // -- Layout preset mapping ----------------------------------------------

    #[test]
    fn layout_preset_from_dto_maps_all_variants() {
        assert!(matches!(
            layout_preset_from_dto(LayoutPresetDto::OneByOne),
            LayoutPreset::OneByOne
        ));
        assert!(matches!(
            layout_preset_from_dto(LayoutPresetDto::OneByTwo),
            LayoutPreset::OneByTwo
        ));
        assert!(matches!(
            layout_preset_from_dto(LayoutPresetDto::TwoByTwo),
            LayoutPreset::TwoByTwo
        ));
        assert!(matches!(
            layout_preset_from_dto(LayoutPresetDto::TwoByThree),
            LayoutPreset::TwoByThree
        ));
        assert!(matches!(
            layout_preset_from_dto(LayoutPresetDto::ThreeByThree),
            LayoutPreset::ThreeByThree
        ));
    }

    // -- Split direction mapping --------------------------------------------

    #[test]
    fn split_direction_from_dto_maps_both_variants() {
        assert!(matches!(
            split_direction_from_dto(SplitDirectionDto::Horizontal),
            SplitDirection::Horizontal
        ));
        assert!(matches!(
            split_direction_from_dto(SplitDirectionDto::Vertical),
            SplitDirection::Vertical
        ));
    }

    // -- Theme mode mapping -------------------------------------------------

    #[test]
    fn theme_mode_round_trips() {
        let modes = [ThemeMode::System, ThemeMode::Dawn, ThemeMode::Midnight];
        for mode in modes {
            let dto = theme_mode_to_dto(mode);
            let restored = theme_mode_from_dto(dto);
            assert_eq!(
                std::mem::discriminant(&mode),
                std::mem::discriminant(&restored)
            );
        }
    }

    // -- PaneRuntime → PaneRuntimeView --------------------------------------

    #[test]
    fn pane_runtime_to_view_maps_terminal() {
        let runtime = PaneRuntime {
            pane_id: String::from("pane-1"),
            runtime_session_id: Some(RuntimeSessionId::from(String::from("pty-abc"))),
            kind: RuntimeKind::Terminal,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: None,
        };

        let view = pane_runtime_to_view(&runtime);

        assert_eq!(view.pane_id, "pane-1");
        assert_eq!(view.runtime_session_id.as_deref(), Some("pty-abc"));
        assert!(matches!(view.kind, RuntimeKindDto::Terminal));
        assert!(matches!(view.status, RuntimeStatusDto::Running));
        assert!(view.last_error.is_none());
    }

    #[test]
    fn pane_runtime_to_view_maps_browser() {
        let runtime = PaneRuntime {
            pane_id: String::from("pane-2"),
            runtime_session_id: Some(RuntimeSessionId::from(String::from("browser-xyz"))),
            kind: RuntimeKind::Browser,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: Some(String::from("https://example.com")),
        };

        let view = pane_runtime_to_view(&runtime);

        assert!(matches!(view.kind, RuntimeKindDto::Browser));
        assert_eq!(
            view.browser_location.as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn pane_runtime_to_view_maps_failed_status_with_error() {
        let runtime = PaneRuntime {
            pane_id: String::from("pane-3"),
            runtime_session_id: None,
            kind: RuntimeKind::Terminal,
            status: RuntimeStatus::Failed,
            last_error: Some(String::from("spawn failed")),
            browser_location: None,
        };

        let view = pane_runtime_to_view(&runtime);

        assert!(matches!(view.status, RuntimeStatusDto::Failed));
        assert_eq!(view.last_error.as_deref(), Some("spawn failed"));
    }

    // -- ProfileCatalog → ProfileCatalogView --------------------------------

    #[test]
    fn profile_catalog_view_maps_profiles() {
        let catalog = ProfileCatalog {
            terminal_profiles: vec![
                TerminalProfile {
                    id: ProfileId::new("terminal"),
                    label: String::from("Terminal"),
                    description: String::from("Default terminal"),
                    startup_command_template: None,
                },
                TerminalProfile {
                    id: ProfileId::new("claude"),
                    label: String::from("Claude Code"),
                    description: String::from("AI assistant"),
                    startup_command_template: Some(String::from("claude")),
                },
            ],
        };

        let view = profile_catalog_view_from_catalog(&catalog);

        assert_eq!(view.terminal_profiles.len(), 2);
        assert_eq!(view.terminal_profiles[0].id, "terminal");
        assert_eq!(view.terminal_profiles[1].id, "claude");
        assert!(view.terminal_profiles[0].startup_command_template.is_none());
        assert_eq!(
            view.terminal_profiles[1]
                .startup_command_template
                .as_deref(),
            Some("claude")
        );
    }

    // -- WorkspaceCommandDto → WorkspaceCommand -----------------------------

    #[test]
    fn workspace_command_open_tab_with_layout() {
        let dto = WorkspaceCommandDto::OpenTab {
            layout: Some(LayoutPresetDto::TwoByTwo),
            auto_layout: false,
            pane_specs: vec![PaneSpecDto::Terminal {
                launch_profile_id: String::from("terminal"),
                working_directory: String::from("/tmp"),
                command_override: None,
            }],
        };

        let cmd = workspace_command_from_dto(dto, LayoutPreset::OneByOne);

        match cmd {
            WorkspaceCommand::OpenTab(open) => {
                assert!(matches!(open.layout, LayoutPreset::TwoByTwo));
                assert!(!open.auto_layout);
                assert_eq!(open.pane_specs.len(), 1);
            }
            other => panic!("Expected OpenTab, got {other:?}"),
        }
    }

    #[test]
    fn workspace_command_open_tab_uses_default_layout_when_none() {
        let dto = WorkspaceCommandDto::OpenTab {
            layout: None,
            auto_layout: false,
            pane_specs: vec![],
        };

        let cmd = workspace_command_from_dto(dto, LayoutPreset::TwoByThree);

        match cmd {
            WorkspaceCommand::OpenTab(open) => {
                assert!(matches!(open.layout, LayoutPreset::TwoByThree));
            }
            other => panic!("Expected OpenTab, got {other:?}"),
        }
    }

    #[test]
    fn workspace_command_split_pane_maps_direction_and_spec() {
        let dto = WorkspaceCommandDto::SplitPane {
            pane_id: String::from("pane-1"),
            direction: SplitDirectionDto::Vertical,
            pane_spec: PaneSpecDto::Browser {
                initial_url: String::from("https://example.com"),
            },
        };

        let cmd = workspace_command_from_dto(dto, LayoutPreset::OneByOne);

        match cmd {
            WorkspaceCommand::SplitPane(split) => {
                assert_eq!(split.pane_id.as_ref(), "pane-1");
                assert!(matches!(split.direction, SplitDirection::Vertical));
                assert!(matches!(split.spec, PaneSpec::Browser(_)));
            }
            other => panic!("Expected SplitPane, got {other:?}"),
        }
    }

    #[test]
    fn workspace_command_close_tab_maps_id() {
        let dto = WorkspaceCommandDto::CloseTab {
            tab_id: String::from("tab-42"),
        };
        let cmd = workspace_command_from_dto(dto, LayoutPreset::OneByOne);
        match cmd {
            WorkspaceCommand::CloseTab(close) => assert_eq!(close.tab_id.as_ref(), "tab-42"),
            other => panic!("Expected CloseTab, got {other:?}"),
        }
    }

    #[test]
    fn workspace_command_replace_pane_spec_maps_correctly() {
        let dto = WorkspaceCommandDto::ReplacePaneSpec {
            pane_id: String::from("pane-5"),
            pane_spec: PaneSpecDto::Terminal {
                launch_profile_id: String::from("codex"),
                working_directory: String::from("/home"),
                command_override: Some(String::from("codex")),
            },
        };

        let cmd = workspace_command_from_dto(dto, LayoutPreset::OneByOne);

        match cmd {
            WorkspaceCommand::ReplacePaneSpec(replace) => {
                assert_eq!(replace.pane_id.as_ref(), "pane-5");
                match replace.spec {
                    PaneSpec::Terminal(t) => {
                        assert_eq!(t.launch_profile_id, "codex");
                        assert_eq!(t.command_override.as_deref(), Some("codex"));
                    }
                    PaneSpec::Browser(_) => panic!("Expected Terminal"),
                }
            }
            other => panic!("Expected ReplacePaneSpec, got {other:?}"),
        }
    }

    // -- SettingsCommandDto → SettingsCommand --------------------------------

    #[test]
    fn settings_command_update_maps_preferences() {
        let view = settings_view_from_preferences(&default_preferences());
        let dto = SettingsCommandDto::Update { settings: view };
        let cmd = settings_command_from_dto(dto).expect("should map");

        match cmd {
            SettingsCommand::Update(update) => {
                assert_eq!(
                    update.preferences.default_terminal_profile_id,
                    default_preferences().default_terminal_profile_id
                );
            }
            SettingsCommand::Reset => panic!("Expected Update"),
        }
    }

    #[test]
    fn settings_command_reset_maps_correctly() {
        let dto = SettingsCommandDto::Reset;
        let cmd = settings_command_from_dto(dto).expect("should map");
        assert!(matches!(cmd, SettingsCommand::Reset));
    }

    #[test]
    fn settings_command_rejects_invalid_font_size() {
        let mut view = settings_view_from_preferences(&default_preferences());
        view.font_size = 200;
        let err = settings_command_from_dto(SettingsCommandDto::Update { settings: view })
            .expect_err("should reject invalid font size");
        assert!(err.to_string().contains("Font size"));
    }

    // -- RuntimeCommandDto → RuntimeCommand ----------------------------------

    #[test]
    fn runtime_command_write_input_maps_correctly() {
        let dto = RuntimeCommandDto::WriteTerminalInput {
            pane_id: String::from("pane-1"),
            input: String::from("ls\n"),
        };
        let cmd = runtime_command_from_dto(dto);
        match cmd {
            RuntimeCommand::WriteTerminalInput { pane_id, input } => {
                assert_eq!(pane_id.as_ref(), "pane-1");
                assert_eq!(input, "ls\n");
            }
            other => panic!("Expected WriteTerminalInput, got {other:?}"),
        }
    }

    #[test]
    fn runtime_command_resize_maps_correctly() {
        let dto = RuntimeCommandDto::ResizeTerminal {
            pane_id: String::from("pane-1"),
            cols: 120,
            rows: 40,
        };
        let cmd = runtime_command_from_dto(dto);
        match cmd {
            RuntimeCommand::ResizeTerminal {
                pane_id,
                cols,
                rows,
            } => {
                assert_eq!(pane_id.as_ref(), "pane-1");
                assert_eq!(cols, 120);
                assert_eq!(rows, 40);
            }
            other => panic!("Expected ResizeTerminal, got {other:?}"),
        }
    }

    #[test]
    fn runtime_command_navigate_browser_maps_correctly() {
        let dto = RuntimeCommandDto::NavigateBrowser {
            pane_id: String::from("pane-b"),
            url: String::from("https://rust-lang.org"),
        };
        let cmd = runtime_command_from_dto(dto);
        match cmd {
            RuntimeCommand::NavigateBrowser { pane_id, url } => {
                assert_eq!(pane_id.as_ref(), "pane-b");
                assert_eq!(url, "https://rust-lang.org");
            }
            other => panic!("Expected NavigateBrowser, got {other:?}"),
        }
    }

    #[test]
    fn runtime_command_observe_terminal_cwd_maps_correctly() {
        let dto = RuntimeCommandDto::ObserveTerminalCwd {
            pane_id: String::from("pane-t"),
            working_directory: String::from("/tmp"),
        };
        let cmd = runtime_command_from_dto(dto);
        match cmd {
            RuntimeCommand::ObserveTerminalCwd {
                pane_id,
                working_directory,
            } => {
                assert_eq!(pane_id.as_ref(), "pane-t");
                assert_eq!(working_directory, "/tmp");
            }
            other => panic!("Expected ObserveTerminalCwd, got {other:?}"),
        }
    }

    #[test]
    fn runtime_command_observe_browser_location_maps_correctly() {
        let dto = RuntimeCommandDto::ObserveBrowserLocation {
            pane_id: String::from("pane-b"),
            url: String::from("https://example.com/page"),
        };
        let cmd = runtime_command_from_dto(dto);
        match cmd {
            RuntimeCommand::ObserveBrowserLocation { pane_id, url } => {
                assert_eq!(pane_id.as_ref(), "pane-b");
                assert_eq!(url, "https://example.com/page");
            }
            other => panic!("Expected ObserveBrowserLocation, got {other:?}"),
        }
    }

    // -- Round-trip conversion tests for new value types ----------------------

    #[test]
    fn runtime_command_pane_id_converts_string_to_pane_id() {
        let wire_id = String::from("pane-round-trip");
        let dto = RuntimeCommandDto::WriteTerminalInput {
            pane_id: wire_id.clone(),
            input: String::from("echo hi"),
        };
        let cmd = runtime_command_from_dto(dto);
        match cmd {
            RuntimeCommand::WriteTerminalInput { pane_id, .. } => {
                assert_eq!(pane_id, PaneId::from(wire_id));
                assert_eq!(pane_id.to_string(), "pane-round-trip");
            }
            other => panic!("Expected WriteTerminalInput, got {other:?}"),
        }
    }

    #[test]
    fn workspace_command_tab_id_round_trips_through_string() {
        let wire_id = String::from("tab-round-trip");
        let dto = WorkspaceCommandDto::CloseTab {
            tab_id: wire_id.clone(),
        };
        let cmd = workspace_command_from_dto(dto, LayoutPreset::OneByOne);
        match cmd {
            WorkspaceCommand::CloseTab(close) => {
                assert_eq!(close.tab_id, TabId::from(wire_id));
                assert_eq!(close.tab_id.to_string(), "tab-round-trip");
            }
            other => panic!("Expected CloseTab, got {other:?}"),
        }
    }

    #[test]
    fn workspace_command_pane_id_round_trips_through_string() {
        let wire_id = String::from("pane-round-trip");
        let dto = WorkspaceCommandDto::ClosePane {
            pane_id: wire_id.clone(),
        };
        let cmd = workspace_command_from_dto(dto, LayoutPreset::OneByOne);
        match cmd {
            WorkspaceCommand::ClosePane { pane_id } => {
                assert_eq!(pane_id, PaneId::from(wire_id));
                assert_eq!(pane_id.to_string(), "pane-round-trip");
            }
            other => panic!("Expected ClosePane, got {other:?}"),
        }
    }

    #[test]
    fn settings_font_size_round_trips_through_u16() {
        let wire_size: u16 = 20;
        let preferences = UserPreferences {
            font_size: FontSize::new(wire_size).expect("valid size"),
            ..default_preferences()
        };
        let view = settings_view_from_preferences(&preferences);
        assert_eq!(view.font_size, wire_size);

        let restored = preferences_from_settings_view(&view).expect("should round-trip");
        assert_eq!(restored.font_size.value(), wire_size);
    }

    #[test]
    fn settings_working_directory_round_trips_through_string() {
        let wire_dir = String::from("/usr/local/bin");
        let preferences = UserPreferences {
            default_working_directory: WorkingDirectory::new(wire_dir.clone()).expect("valid path"),
            ..default_preferences()
        };
        let view = settings_view_from_preferences(&preferences);
        assert_eq!(view.default_working_directory, wire_dir);

        let restored = preferences_from_settings_view(&view).expect("should round-trip");
        assert_eq!(
            restored.default_working_directory.as_str(),
            wire_dir.as_str()
        );
    }

    #[test]
    fn settings_profile_id_round_trips_through_string() {
        let wire_id = String::from("custom-profile");
        let preferences = UserPreferences {
            default_terminal_profile_id: ProfileId::new(&wire_id),
            ..default_preferences()
        };
        let view = settings_view_from_preferences(&preferences);
        assert_eq!(view.default_terminal_profile_id, wire_id);

        let restored = preferences_from_settings_view(&view).expect("should round-trip");
        assert_eq!(
            restored.default_terminal_profile_id.as_str(),
            wire_id.as_str()
        );
    }

    #[test]
    fn pane_runtime_session_id_round_trips_through_string() {
        let wire_session = String::from("pty-session-round-trip");
        let runtime = PaneRuntime {
            pane_id: String::from("pane-1"),
            runtime_session_id: Some(RuntimeSessionId::from(wire_session.clone())),
            kind: RuntimeKind::Terminal,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: None,
        };
        let view = pane_runtime_to_view(&runtime);
        assert_eq!(
            view.runtime_session_id.as_deref(),
            Some(wire_session.as_str())
        );
    }

    // -- Persistence helpers ------------------------------------------------

    #[test]
    fn serialize_deserialize_preferences_round_trips() {
        let preferences = UserPreferences {
            default_layout: String::from("1x2"),
            default_terminal_profile_id: ProfileId::new("claude"),
            default_working_directory: WorkingDirectory::new("/home").expect("valid path"),
            default_custom_command: String::new(),
            font_size: FontSize::new(18).expect("valid size"),
            theme: ThemeMode::Midnight,
            launch_fullscreen: false,
            has_completed_onboarding: true,
            last_working_directory: Some(String::from("/var")),
        };

        let value = serialize_preferences(&preferences).expect("should serialize");
        let restored = deserialize_preferences(value).expect("should deserialize");

        assert_eq!(restored.default_layout, "1x2");
        assert_eq!(restored.default_terminal_profile_id, "claude");
        assert_eq!(restored.font_size.value(), 18);
        assert!(matches!(restored.theme, ThemeMode::Midnight));
        assert_eq!(restored.last_working_directory.as_deref(), Some("/var"));
    }

    // -- Bootstrap view composition -----------------------------------------

    #[test]
    fn bootstrap_view_assembles_all_projections() {
        let session = WorkspaceSession::default();
        let preferences = default_preferences();
        let catalog = ProfileCatalog {
            terminal_profiles: vec![TerminalProfile {
                id: ProfileId::new("terminal"),
                label: String::from("Terminal"),
                description: String::from("Default"),
                startup_command_template: None,
            }],
        };
        let runtimes: Vec<PaneRuntime> = vec![];

        let view = bootstrap_view(&session, &preferences, &catalog, &runtimes);

        assert!(view.workspace.tabs.is_empty());
        assert_eq!(view.profile_catalog.terminal_profiles.len(), 1);
        assert!(view.runtime_projections.is_empty());
    }
}
