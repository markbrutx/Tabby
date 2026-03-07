pub mod commands;
pub mod error;
pub mod events;
pub mod snapshot;
pub mod split_tree;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::cli::CliArgs;
    use crate::domain::commands::LaunchRequest;
    use crate::domain::snapshot::{PaneRuntimeStatus, TabSnapshot};
    use crate::domain::split_tree::tree_from_preset;
    use crate::domain::types::{default_settings, LayoutPreset, PaneSeed};

    fn pane_seed(id: &str) -> PaneSeed {
        PaneSeed {
            pane_id: format!("pane-{id}"),
            session_id: format!("session-{id}"),
            cwd: String::from("/tmp/workspace"),
            profile_id: String::from("terminal"),
            profile_label: String::from("Terminal"),
            startup_command: None,
        }
    }

    #[test]
    fn launch_request_uses_settings_defaults_when_cli_is_empty() {
        let settings = default_settings(String::from("/Users/mark"));
        let request = LaunchRequest::from_cli_args(CliArgs::default(), &settings)
            .expect("launch request should be created");

        assert_eq!(request.preset, LayoutPreset::OneByOne);
        assert_eq!(request.cwd.as_deref(), Some("/Users/mark"));
        assert_eq!(request.profile_id.as_deref(), Some("terminal"));
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
