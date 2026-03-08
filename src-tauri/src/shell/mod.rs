pub(crate) mod browser_surface;
pub mod error;
pub(crate) mod mapping;
pub(crate) mod pty;

use tauri::AppHandle;

use tabby_contracts::{
    LayoutPresetDto, RuntimeCommandDto, SettingsCommandDto, SettingsView, WorkspaceBootstrapView,
    WorkspaceCommandDto, WorkspaceView,
};
use tabby_settings::{
    built_in_profile_catalog, default_preferences, resolve_default_working_directory,
};
use tabby_workspace::layout::LayoutPreset;
use tabby_workspace::{PaneSpec, WorkspaceEvent};

use crate::application::{
    ProjectionPublisher, RuntimeApplicationService, SettingsApplicationService,
    WorkspaceApplicationService,
};
use crate::cli::CliArgs;
use crate::shell::error::ShellError;
use crate::shell::mapping::{
    layout_preset_from_dto, pane_spec_from_dto, profile_catalog_view_from_catalog,
    split_direction_from_dto,
};

pub const WORKSPACE_PROJECTION_UPDATED_EVENT: &str = "workspace_projection_updated";
pub const SETTINGS_PROJECTION_UPDATED_EVENT: &str = "settings_projection_updated";
pub const RUNTIME_STATUS_CHANGED_EVENT: &str = "runtime_status_changed";
pub const TERMINAL_OUTPUT_RECEIVED_EVENT: &str = "terminal_output_received";
pub const BROWSER_LOCATION_OBSERVED_EVENT: &str = "browser_location_observed";

#[derive(Debug)]
pub struct AppShell {
    settings_service: SettingsApplicationService,
    workspace_service: WorkspaceApplicationService,
    runtime_service: RuntimeApplicationService,
    publisher: ProjectionPublisher,
    launch_overrides: std::sync::Mutex<Option<CliArgs>>,
}

impl AppShell {
    pub fn new(app: AppHandle, cli_args: CliArgs) -> Result<Self, ShellError> {
        let settings_service = SettingsApplicationService::new(app.clone())?;
        Ok(Self {
            workspace_service: WorkspaceApplicationService::new(),
            runtime_service: RuntimeApplicationService::new(app.clone()),
            publisher: ProjectionPublisher::new(app),
            launch_overrides: std::sync::Mutex::new(Some(cli_args)),
            settings_service,
        })
    }

    pub fn bootstrap(&self) -> Result<WorkspaceBootstrapView, ShellError> {
        let cli_args = self
            .launch_overrides
            .lock()
            .map_err(|_| ShellError::State(String::from("Launch overrides lock poisoned")))?
            .take()
            .unwrap_or_default();

        if self.workspace_service.is_empty()? {
            if cli_args.has_launch_overrides() {
                self.apply_cli_launch_request(cli_args)?;
            } else {
                let preferences = self.settings_service.preferences()?;
                if preferences.has_completed_onboarding {
                    self.open_default_tab()?;
                }
            }
        }

        Ok(WorkspaceBootstrapView {
            workspace: self.workspace_service.workspace_view()?,
            settings: self.settings_service.settings_view()?,
            profile_catalog: profile_catalog_view_from_catalog(&built_in_profile_catalog()),
            runtime_projections: self.runtime_service.snapshot()?,
        })
    }

    pub fn apply_cli_launch_request(&self, cli_args: CliArgs) -> Result<(), ShellError> {
        if !cli_args.has_launch_overrides() {
            return Ok(());
        }

        let preferences = self.settings_service.preferences()?;
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
        let events = self
            .workspace_service
            .open_tab(layout, false, vec![pane_spec])?;
        self.apply_workspace_events(events)?;
        Ok(())
    }

    pub fn dispatch_workspace_command(
        &self,
        command: WorkspaceCommandDto,
    ) -> Result<WorkspaceView, ShellError> {
        match command {
            WorkspaceCommandDto::OpenTab {
                layout,
                auto_layout,
                pane_specs,
            } => {
                let layout = layout.unwrap_or_else(|| {
                    let preferences = self
                        .settings_service
                        .preferences()
                        .unwrap_or_else(|_| default_preferences());
                    match LayoutPreset::parse(&preferences.default_layout)
                        .unwrap_or(LayoutPreset::OneByOne)
                    {
                        LayoutPreset::OneByOne => LayoutPresetDto::OneByOne,
                        LayoutPreset::OneByTwo => LayoutPresetDto::OneByTwo,
                        LayoutPreset::TwoByTwo => LayoutPresetDto::TwoByTwo,
                        LayoutPreset::TwoByThree => LayoutPresetDto::TwoByThree,
                        LayoutPreset::ThreeByThree => LayoutPresetDto::ThreeByThree,
                    }
                });
                let specs = pane_specs
                    .into_iter()
                    .map(pane_spec_from_dto)
                    .collect::<Vec<_>>();
                let events = self.workspace_service.open_tab(
                    layout_preset_from_dto(layout),
                    auto_layout,
                    specs,
                )?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommandDto::CloseTab { tab_id } => {
                let events = self.workspace_service.close_tab(&tab_id)?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommandDto::SetActiveTab { tab_id } => {
                self.workspace_service.set_active_tab(&tab_id)?;
            }
            WorkspaceCommandDto::FocusPane { tab_id, pane_id } => {
                self.workspace_service.focus_pane(&tab_id, &pane_id)?;
            }
            WorkspaceCommandDto::SplitPane {
                pane_id,
                direction,
                pane_spec,
            } => {
                let events = self.workspace_service.split_pane(
                    &pane_id,
                    split_direction_from_dto(direction),
                    pane_spec_from_dto(pane_spec),
                )?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommandDto::ClosePane { pane_id } => {
                let events = self.workspace_service.close_pane(&pane_id)?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommandDto::SwapPaneSlots {
                pane_id_a,
                pane_id_b,
            } => {
                self.workspace_service
                    .swap_pane_slots(&pane_id_a, &pane_id_b)?;
            }
            WorkspaceCommandDto::ReplacePaneSpec { pane_id, pane_spec } => {
                self.runtime_service.stop_runtime(&pane_id);
                let events = self
                    .workspace_service
                    .replace_pane_spec(&pane_id, pane_spec_from_dto(pane_spec))?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommandDto::RestartPaneRuntime { pane_id } => {
                let spec = self
                    .workspace_service
                    .pane_spec(&pane_id)?
                    .ok_or_else(|| ShellError::NotFound(format!("pane {pane_id}")))?;
                let preferences = self.settings_service.preferences()?;
                self.runtime_service
                    .restart_runtime(&pane_id, &spec, &preferences)?;
            }
            WorkspaceCommandDto::TrackTerminalWorkingDirectory {
                pane_id,
                working_directory,
            } => {
                self.workspace_service
                    .track_terminal_working_directory(&pane_id, &working_directory)?;
                let mut preferences = self.settings_service.preferences()?;
                preferences.last_working_directory = Some(working_directory);
                self.settings_service.persist_preferences(&preferences)?;
            }
        }

        let view = self.workspace_service.workspace_view()?;
        self.emit_workspace_projection(&view);
        Ok(view)
    }

    pub fn dispatch_settings_command(
        &self,
        command: SettingsCommandDto,
    ) -> Result<SettingsView, ShellError> {
        let (_preferences, settings) = self.settings_service.dispatch_settings_command(command)?;
        self.emit_settings_projection(&settings);
        Ok(settings)
    }

    pub fn dispatch_runtime_command(
        &self,
        window: &tauri::Window,
        command: RuntimeCommandDto,
    ) -> Result<(), ShellError> {
        self.runtime_service
            .dispatch_runtime_command(window, command)
    }

    fn open_default_tab(&self) -> Result<(), ShellError> {
        let preferences = self.settings_service.preferences()?;
        let layout =
            LayoutPreset::parse(&preferences.default_layout).unwrap_or(LayoutPreset::OneByOne);
        let pane_spec = PaneSpec::Terminal(tabby_workspace::TerminalPaneSpec {
            launch_profile_id: preferences.default_terminal_profile_id.clone(),
            working_directory: resolve_default_working_directory(None, &preferences),
            command_override: None,
        });
        let events = self
            .workspace_service
            .open_tab(layout, false, vec![pane_spec])?;
        self.apply_workspace_events(events)?;
        Ok(())
    }

    fn apply_workspace_events(&self, events: Vec<WorkspaceEvent>) -> Result<(), ShellError> {
        let preferences = self.settings_service.preferences()?;
        for event in events {
            match event {
                WorkspaceEvent::PaneAdded { pane_id, spec }
                | WorkspaceEvent::PaneSpecReplaced { pane_id, spec } => {
                    self.runtime_service
                        .start_runtime(&pane_id, &spec, &preferences)?;
                }
                WorkspaceEvent::PaneRemoved { pane_id, .. } => {
                    self.runtime_service.stop_runtime(&pane_id);
                }
            }
        }
        Ok(())
    }

    fn emit_workspace_projection(&self, workspace: &WorkspaceView) {
        self.publisher.emit_workspace_projection(workspace);
    }

    fn emit_settings_projection(&self, settings: &SettingsView) {
        self.publisher.emit_settings_projection(settings);
    }
}
