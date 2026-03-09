pub(crate) mod browser_surface;
pub mod error;
pub(crate) mod pty;

use std::sync::Arc;

use tauri::AppHandle;

use tabby_contracts::{
    BrowserSurfaceCommandDto, RuntimeCommandDto, SettingsCommandDto, SettingsView,
    WorkspaceBootstrapView, WorkspaceCommandDto, WorkspaceView,
};
use tabby_settings::{built_in_profile_catalog, default_preferences};
use tabby_workspace::layout::LayoutPreset;
use tabby_workspace::WorkspaceDomainEvent;

use crate::application::commands::WorkspaceCommand;
use crate::application::ports::ProjectionPublisherPort;
use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
use crate::application::{
    BootstrapService, RuntimeApplicationService, RuntimeCoordinator, SettingsApplicationService,
    WorkspaceApplicationService,
};
use crate::cli::CliArgs;
use crate::infrastructure::{
    TauriBrowserSurfaceAdapter, TauriProjectionPublisher, TauriStorePreferencesRepository,
};
use crate::mapping::dto_mappers;
use crate::shell::error::ShellError;
use crate::shell::pty::PtyManager;

pub const WORKSPACE_PROJECTION_UPDATED_EVENT: &str = "workspace_projection_updated";
pub const SETTINGS_PROJECTION_UPDATED_EVENT: &str = "settings_projection_updated";
pub const RUNTIME_STATUS_CHANGED_EVENT: &str = "runtime_status_changed";
pub const TERMINAL_OUTPUT_RECEIVED_EVENT: &str = "terminal_output_received";

#[derive(Debug)]
pub struct AppShell {
    bootstrap_service: BootstrapService,
    settings_service: SettingsApplicationService,
    workspace_service: WorkspaceApplicationService,
    runtime_service: Arc<RuntimeApplicationService>,
    publisher: Box<dyn ProjectionPublisherPort>,
}

impl AppShell {
    pub fn new(app: AppHandle, cli_args: CliArgs) -> Result<Self, ShellError> {
        let preferences_repository = TauriStorePreferencesRepository::new(app.clone());
        let settings_service = SettingsApplicationService::new(Box::new(preferences_repository))?;

        let terminal_port = PtyManager::new(app.clone());
        let browser_port = TauriBrowserSurfaceAdapter::new(app.clone());
        let runtime_publisher = TauriProjectionPublisher::new(app.clone());
        let publisher = TauriProjectionPublisher::new(app);

        Ok(Self {
            bootstrap_service: BootstrapService::new(cli_args),
            workspace_service: WorkspaceApplicationService::new(),
            runtime_service: Arc::new(RuntimeApplicationService::new(
                Box::new(terminal_port),
                Box::new(browser_port),
                Box::new(runtime_publisher),
            )),
            publisher: Box::new(publisher),
            settings_service,
        })
    }

    pub fn bootstrap(&self) -> Result<WorkspaceBootstrapView, ShellError> {
        let snapshot = self.bootstrap_service.bootstrap(
            &self.workspace_service,
            &self.settings_service,
            &self.runtime_service,
            self.observation_receiver(),
        )?;

        let view = self.workspace_service.with_session(|session| {
            dto_mappers::bootstrap_view(
                session,
                &snapshot.preferences,
                &built_in_profile_catalog(),
                &snapshot.runtimes,
            )
        })?;

        Ok(view)
    }

    pub fn apply_cli_launch_request(&self, cli_args: CliArgs) -> Result<(), ShellError> {
        self.bootstrap_service.apply_cli_launch_request(
            cli_args,
            &self.workspace_service,
            &self.settings_service,
            &self.runtime_service,
            self.observation_receiver(),
        )
    }

    pub fn dispatch_workspace_command(
        &self,
        dto: WorkspaceCommandDto,
    ) -> Result<WorkspaceView, ShellError> {
        let default_layout = self.resolve_default_layout();
        let command = dto_mappers::workspace_command_from_dto(dto, default_layout);
        self.execute_workspace_command(command)?;

        let view = self
            .workspace_service
            .with_session(dto_mappers::workspace_view_from_session)?;
        self.publisher.publish_workspace_projection(&view);
        Ok(view)
    }

    pub fn dispatch_settings_command(
        &self,
        dto: SettingsCommandDto,
    ) -> Result<SettingsView, ShellError> {
        let command = dto_mappers::settings_command_from_dto(dto).map_err(|error| match error {
            tabby_settings::SettingsError::Validation(msg) => ShellError::Validation(msg),
        })?;
        let preferences = self.settings_service.dispatch_settings_command(command)?;
        let settings = dto_mappers::settings_view_from_preferences(&preferences);
        self.publisher.publish_settings_projection(&preferences);
        Ok(settings)
    }

    pub fn dispatch_runtime_command(&self, dto: RuntimeCommandDto) -> Result<(), ShellError> {
        use crate::application::commands::RuntimeCommand;

        let command = dto_mappers::runtime_command_from_dto(dto);
        match command {
            RuntimeCommand::ObserveTerminalCwd {
                pane_id,
                working_directory,
            } => {
                self.runtime_service.observe_terminal_cwd(
                    &pane_id,
                    &working_directory,
                    &self.settings_service,
                )?;
            }
            RuntimeCommand::ObserveBrowserLocation { pane_id, url } => {
                self.runtime_service
                    .observe_browser_location(&pane_id, &url)?;
            }
            other => {
                self.runtime_service.dispatch_runtime_command(other)?;
            }
        }
        Ok(())
    }

    pub fn dispatch_browser_surface_command(
        &self,
        command: BrowserSurfaceCommandDto,
    ) -> Result<(), ShellError> {
        self.runtime_service
            .dispatch_browser_surface_command(command)
    }

    pub fn handle_browser_location_observation(
        &self,
        pane_id: &str,
        url: &str,
    ) -> Result<(), ShellError> {
        let pane_id = tabby_workspace::PaneId::from(String::from(pane_id));
        self.runtime_service.observe_browser_location(&pane_id, url)
    }

    fn execute_workspace_command(&self, command: WorkspaceCommand) -> Result<(), ShellError> {
        match command {
            WorkspaceCommand::OpenTab(cmd) => {
                let events =
                    self.workspace_service
                        .open_tab(cmd.layout, cmd.auto_layout, cmd.pane_specs)?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommand::CloseTab(cmd) => {
                let events = self.workspace_service.close_tab(&cmd.tab_id)?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommand::SetActiveTab { tab_id } => {
                let events = self.workspace_service.set_active_tab(&tab_id)?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommand::FocusPane { tab_id, pane_id } => {
                let events = self.workspace_service.focus_pane(&tab_id, &pane_id)?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommand::SplitPane(cmd) => {
                let events =
                    self.workspace_service
                        .split_pane(&cmd.pane_id, cmd.direction, cmd.spec)?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommand::ClosePane { pane_id } => {
                let events = self.workspace_service.close_pane(&pane_id)?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommand::SwapPaneSlots {
                pane_id_a,
                pane_id_b,
            } => {
                self.workspace_service
                    .swap_pane_slots(&pane_id_a, &pane_id_b)?;
            }
            WorkspaceCommand::ReplacePaneSpec(cmd) => {
                let events = self
                    .workspace_service
                    .replace_pane_spec(&cmd.pane_id, cmd.spec)?;
                self.apply_workspace_events(events)?;
            }
            WorkspaceCommand::RestartPaneRuntime { pane_id } => {
                let spec = self
                    .workspace_service
                    .pane_spec(&pane_id)?
                    .ok_or_else(|| ShellError::NotFound(format!("pane {pane_id}")))?;
                let preferences = self.settings_service.preferences()?;
                self.runtime_service.restart_runtime(
                    &pane_id,
                    &spec,
                    &preferences,
                    self.observation_receiver(),
                )?;
            }
        }
        Ok(())
    }

    fn resolve_default_layout(&self) -> LayoutPreset {
        let preferences = self
            .settings_service
            .preferences()
            .unwrap_or_else(|_| default_preferences());
        LayoutPreset::parse(&preferences.default_layout).unwrap_or(LayoutPreset::OneByOne)
    }

    fn observation_receiver(&self) -> Arc<dyn RuntimeObservationReceiver> {
        Arc::clone(&self.runtime_service) as Arc<dyn RuntimeObservationReceiver>
    }

    fn apply_workspace_events(&self, events: Vec<WorkspaceDomainEvent>) -> Result<(), ShellError> {
        RuntimeCoordinator::handle_workspace_events(
            events,
            &self.settings_service,
            &self.runtime_service,
            self.observation_receiver(),
        )
    }
}
