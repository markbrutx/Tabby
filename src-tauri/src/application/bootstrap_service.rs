use std::sync::Mutex;

use tabby_runtime::PaneRuntime;
use tabby_settings::{resolve_default_working_directory, UserPreferences};
use tabby_workspace::layout::LayoutPreset;
use tabby_workspace::{PaneSpec, WorkspaceEvent};

use crate::application::{
    RuntimeApplicationService, SettingsApplicationService, WorkspaceApplicationService,
};
use crate::cli::CliArgs;
use crate::shell::error::ShellError;

/// Domain-level result of bootstrapping — free of transport DTOs.
pub struct BootstrapSnapshot {
    pub preferences: UserPreferences,
    pub runtimes: Vec<PaneRuntime>,
}

#[derive(Debug)]
pub struct BootstrapService {
    launch_overrides: Mutex<Option<CliArgs>>,
}

impl BootstrapService {
    pub fn new(cli_args: CliArgs) -> Self {
        Self {
            launch_overrides: Mutex::new(Some(cli_args)),
        }
    }

    pub fn bootstrap(
        &self,
        workspace_service: &WorkspaceApplicationService,
        settings_service: &SettingsApplicationService,
        runtime_service: &RuntimeApplicationService,
    ) -> Result<BootstrapSnapshot, ShellError> {
        let cli_args = self
            .launch_overrides
            .lock()
            .map_err(|_| ShellError::State(String::from("Launch overrides lock poisoned")))?
            .take()
            .unwrap_or_default();

        if workspace_service.is_empty()? {
            if cli_args.has_launch_overrides() {
                self.apply_cli_launch_request(
                    cli_args,
                    workspace_service,
                    settings_service,
                    runtime_service,
                )?;
            } else {
                let preferences = settings_service.preferences()?;
                if preferences.has_completed_onboarding {
                    self.open_default_tab(workspace_service, settings_service, runtime_service)?;
                }
            }
        }

        Ok(BootstrapSnapshot {
            preferences: settings_service.preferences()?,
            runtimes: runtime_service.snapshot()?,
        })
    }

    pub fn apply_cli_launch_request(
        &self,
        cli_args: CliArgs,
        workspace_service: &WorkspaceApplicationService,
        settings_service: &SettingsApplicationService,
        runtime_service: &RuntimeApplicationService,
    ) -> Result<(), ShellError> {
        if !cli_args.has_launch_overrides() {
            return Ok(());
        }

        let preferences = settings_service.preferences()?;
        let layout = cli_args
            .layout
            .as_deref()
            .map(LayoutPreset::parse)
            .transpose()
            .map_err(|error| ShellError::Validation(error.to_string()))?
            .unwrap_or_else(|| {
                LayoutPreset::parse(&preferences.default_layout).unwrap_or(LayoutPreset::OneByOne)
            });
        let profile_id = cli_args
            .profile
            .unwrap_or_else(|| preferences.default_terminal_profile_id.clone());
        let working_directory =
            resolve_default_working_directory(cli_args.cwd.as_deref(), &preferences);
        let pane_spec = PaneSpec::Terminal(tabby_workspace::TerminalPaneSpec {
            launch_profile_id: profile_id,
            working_directory,
            command_override: cli_args.command,
        });
        let events = workspace_service.open_tab(layout, false, vec![pane_spec])?;
        Self::apply_workspace_events(events, settings_service, runtime_service)?;
        Ok(())
    }

    fn open_default_tab(
        &self,
        workspace_service: &WorkspaceApplicationService,
        settings_service: &SettingsApplicationService,
        runtime_service: &RuntimeApplicationService,
    ) -> Result<(), ShellError> {
        let preferences = settings_service.preferences()?;
        let layout =
            LayoutPreset::parse(&preferences.default_layout).unwrap_or(LayoutPreset::OneByOne);
        let pane_spec = PaneSpec::Terminal(tabby_workspace::TerminalPaneSpec {
            launch_profile_id: preferences.default_terminal_profile_id.clone(),
            working_directory: resolve_default_working_directory(None, &preferences),
            command_override: None,
        });
        let events = workspace_service.open_tab(layout, false, vec![pane_spec])?;
        Self::apply_workspace_events(events, settings_service, runtime_service)?;
        Ok(())
    }

    pub(crate) fn apply_workspace_events(
        events: Vec<WorkspaceEvent>,
        settings_service: &SettingsApplicationService,
        runtime_service: &RuntimeApplicationService,
    ) -> Result<(), ShellError> {
        let preferences = settings_service.preferences()?;
        for event in events {
            match event {
                WorkspaceEvent::PaneAdded { pane_id, spec }
                | WorkspaceEvent::PaneSpecReplaced { pane_id, spec } => {
                    runtime_service.start_runtime(&pane_id, &spec, &preferences)?;
                }
                WorkspaceEvent::PaneRemoved { pane_id, .. } => {
                    runtime_service.stop_runtime(&pane_id);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bootstrap_service_stores_cli_args() {
        let args = CliArgs {
            new_tab: true,
            profile: Some(String::from("codex")),
            ..CliArgs::default()
        };
        let service = BootstrapService::new(args);
        let taken = service
            .launch_overrides
            .lock()
            .expect("lock should not be poisoned")
            .take();
        assert!(taken.is_some());
        let taken = taken.expect("should have args");
        assert!(taken.new_tab);
        assert_eq!(taken.profile.as_deref(), Some("codex"));
    }

    #[test]
    fn launch_overrides_are_consumed_on_take() {
        let args = CliArgs {
            new_tab: true,
            ..CliArgs::default()
        };
        let service = BootstrapService::new(args);

        // First take returns the args
        let first = service.launch_overrides.lock().expect("lock").take();
        assert!(first.is_some());

        // Second take returns None (consumed)
        let second = service.launch_overrides.lock().expect("lock").take();
        assert!(second.is_none());
    }

    #[test]
    fn default_cli_args_have_no_launch_overrides() {
        let service = BootstrapService::new(CliArgs::default());
        let args = service
            .launch_overrides
            .lock()
            .expect("lock")
            .take()
            .unwrap_or_default();
        assert!(!args.has_launch_overrides());
    }
}
