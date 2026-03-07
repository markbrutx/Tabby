pub mod commands;
pub mod error;
pub mod events;
pub mod layout;
pub mod pane;
pub mod profiles;
pub mod settings;
pub mod snapshot;
pub mod split_tree;

#[cfg(test)]
mod tests {
    use crate::cli::CliArgs;
    use crate::domain::commands::{LaunchRequest, NewTabRequest, PaneConfig};
    use crate::domain::snapshot::{PaneRuntimeStatus, TabSnapshot};
    use crate::domain::split_tree::tree_from_preset;
    use crate::domain::layout::LayoutPreset;
    use crate::domain::pane::PaneSeed;
    use crate::domain::settings::default_settings;

    fn pane_seed(id: &str) -> PaneSeed {
        PaneSeed {
            pane_id: format!("pane-{id}"),
            session_id: format!("session-{id}"),
            cwd: String::from("/tmp/workspace"),
            profile_id: String::from("terminal"),
            profile_label: String::from("Terminal"),
            startup_command: None,
            pane_kind: crate::domain::pane::PaneKind::Terminal,
            url: None,
        }
    }

    #[test]
    fn launch_request_returns_none_when_defaults_are_empty() {
        let settings = default_settings();
        let request = LaunchRequest::from_cli_args(CliArgs::default(), &settings)
            .expect("launch request should be created");

        assert_eq!(request.preset, LayoutPreset::OneByOne);
        assert_eq!(request.cwd, None);
        assert_eq!(request.profile_id, None);
    }

    #[test]
    fn launch_request_uses_settings_defaults_when_set() {
        let mut settings = default_settings();
        settings.default_working_directory = String::from("/Users/mark");
        settings.default_profile_id = String::from("terminal");

        let request = LaunchRequest::from_cli_args(CliArgs::default(), &settings)
            .expect("launch request should be created");

        assert_eq!(request.preset, LayoutPreset::OneByOne);
        assert_eq!(request.cwd.as_deref(), Some("/Users/mark"));
        assert_eq!(request.profile_id.as_deref(), Some("terminal"));
    }

    #[test]
    fn new_tab_request_backward_compat_defaults_pane_configs_to_none() {
        let json = r#"{"preset":"1x1","cwd":null,"profileId":null,"startupCommand":null}"#;
        let request: NewTabRequest = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(request.pane_configs, None);
    }

    #[test]
    fn new_tab_request_deserializes_with_pane_configs() {
        let json = r#"{
            "preset": "1x2",
            "cwd": null,
            "profileId": null,
            "startupCommand": null,
            "paneConfigs": [
                {"profileId": "terminal", "cwd": "/tmp/a", "startupCommand": null},
                {"profileId": "claude", "cwd": "/tmp/b", "startupCommand": null}
            ]
        }"#;
        let request: NewTabRequest = serde_json::from_str(json).expect("should deserialize");
        let configs = request.pane_configs.expect("should have pane_configs");
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].profile_id, "terminal");
        assert_eq!(configs[0].cwd, "/tmp/a");
        assert_eq!(configs[1].profile_id, "claude");
        assert_eq!(configs[1].cwd, "/tmp/b");
    }

    #[test]
    fn pane_config_serializes_correctly() {
        let config = PaneConfig {
            profile_id: String::from("terminal"),
            cwd: String::from("/home/user"),
            startup_command: Some(String::from("npm start")),
            url: None,
        };
        let json = serde_json::to_string(&config).expect("should serialize");
        assert!(json.contains("profileId"));
        assert!(json.contains("npm start"));
    }

    #[test]
    fn tab_snapshot_maps_runtime_status_for_all_seeded_panes() {
        let seeds = vec![pane_seed("a"), pane_seed("b")];
        let pane_ids: Vec<String> = seeds.iter().map(|s| s.pane_id.clone()).collect();
        let layout = tree_from_preset(LayoutPreset::OneByTwo, &pane_ids);

        let snapshot = TabSnapshot::from_seeds(
            String::from("tab-1"),
            String::from("Workspace 1"),
            layout,
            seeds,
            PaneRuntimeStatus::Starting,
        )
        .expect("tab snapshot should be built");

        assert_eq!(snapshot.active_pane_id, "pane-a");
        assert_eq!(snapshot.panes.len(), 2);
        assert!(snapshot
            .panes
            .iter()
            .all(|pane| pane.status == PaneRuntimeStatus::Starting));
    }
}
