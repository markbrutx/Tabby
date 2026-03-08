pub(crate) mod browser_surface;
pub mod error;
pub(crate) mod mapping;
mod pty;

use std::sync::Mutex;

use tauri::{AppHandle, Manager};
use tracing::warn;

use tabby_contracts::{
    LayoutPresetDto, RuntimeCommandDto, SettingsCommandDto, SettingsView, WorkspaceBootstrapView,
    WorkspaceCommandDto, WorkspaceView,
};
use tabby_runtime::{RuntimeRegistry, RuntimeStatus};
use tabby_settings::{
    built_in_profile_catalog, default_preferences, resolve_default_working_directory,
    resolve_terminal_profile, SettingsError,
};
use tabby_workspace::layout::LayoutPreset;
use tabby_workspace::{PaneSpec, WorkspaceEvent};

use crate::application::{
    ProjectionPublisher, SettingsApplicationService, WorkspaceApplicationService,
};
use crate::cli::CliArgs;
use crate::shell::browser_surface::navigate_browser;
use crate::shell::error::ShellError;
use crate::shell::mapping::{
    layout_preset_from_dto, pane_runtime_to_view, pane_spec_from_dto,
    profile_catalog_view_from_catalog, split_direction_from_dto,
};
use crate::shell::pty::PtyManager;

pub const WORKSPACE_PROJECTION_UPDATED_EVENT: &str = "workspace_projection_updated";
pub const SETTINGS_PROJECTION_UPDATED_EVENT: &str = "settings_projection_updated";
pub const RUNTIME_STATUS_CHANGED_EVENT: &str = "runtime_status_changed";
pub const TERMINAL_OUTPUT_RECEIVED_EVENT: &str = "terminal_output_received";
pub const BROWSER_LOCATION_OBSERVED_EVENT: &str = "browser_location_observed";

#[derive(Debug)]
pub struct AppShell {
    app: AppHandle,
    settings_service: SettingsApplicationService,
    workspace_service: WorkspaceApplicationService,
    runtimes: Mutex<RuntimeRegistry>,
    pty_manager: PtyManager,
    publisher: ProjectionPublisher,
    launch_overrides: Mutex<Option<CliArgs>>,
}

impl AppShell {
    pub fn new(app: AppHandle, cli_args: CliArgs) -> Result<Self, ShellError> {
        let settings_service = SettingsApplicationService::new(app.clone())?;
        Ok(Self {
            workspace_service: WorkspaceApplicationService::new(),
            runtimes: Mutex::new(RuntimeRegistry::default()),
            pty_manager: PtyManager::new(app.clone()),
            publisher: ProjectionPublisher::new(app.clone()),
            launch_overrides: Mutex::new(Some(cli_args)),
            settings_service,
            app,
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
            runtime_projections: self
                .runtimes
                .lock()
                .map_err(|_| ShellError::State(String::from("Runtime lock poisoned")))?
                .snapshot()
                .iter()
                .map(pane_runtime_to_view)
                .collect(),
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
                self.stop_runtime_for_pane(&pane_id);
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
                self.stop_runtime_for_pane(&pane_id);
                self.start_runtime_for_pane(&pane_id, &spec)?;
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
        match command {
            RuntimeCommandDto::WriteTerminalInput { pane_id, input } => {
                let runtime_session_id = self
                    .runtimes
                    .lock()
                    .map_err(|_| ShellError::State(String::from("Runtime lock poisoned")))?
                    .terminal_session_id(&pane_id)
                    .ok_or_else(|| ShellError::NotFound(format!("runtime for pane {pane_id}")))?;
                self.pty_manager.write(&runtime_session_id, &input)?;
            }
            RuntimeCommandDto::ResizeTerminal {
                pane_id,
                cols,
                rows,
            } => {
                let runtime_session_id = self
                    .runtimes
                    .lock()
                    .map_err(|_| ShellError::State(String::from("Runtime lock poisoned")))?
                    .terminal_session_id(&pane_id)
                    .ok_or_else(|| ShellError::NotFound(format!("runtime for pane {pane_id}")))?;
                self.pty_manager.resize(&runtime_session_id, cols, rows)?;
            }
            RuntimeCommandDto::NavigateBrowser { pane_id, url } => {
                navigate_browser(window, &pane_id, &url)?;
                let maybe_runtime = self
                    .runtimes
                    .lock()
                    .map_err(|_| ShellError::State(String::from("Runtime lock poisoned")))?
                    .update_browser_location(&pane_id, url)
                    .ok();
                if let Some(runtime) = maybe_runtime {
                    self.emit_runtime_status(&pane_runtime_to_view(&runtime));
                }
            }
        }

        Ok(())
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
        for event in events {
            match event {
                WorkspaceEvent::PaneAdded { pane_id, spec }
                | WorkspaceEvent::PaneSpecReplaced { pane_id, spec } => {
                    self.start_runtime_for_pane(&pane_id, &spec)?;
                }
                WorkspaceEvent::PaneRemoved { pane_id, .. } => {
                    self.stop_runtime_for_pane(&pane_id);
                }
            }
        }
        Ok(())
    }

    fn start_runtime_for_pane(&self, pane_id: &str, spec: &PaneSpec) -> Result<(), ShellError> {
        let preferences = self.settings_service.preferences()?;
        let runtime_view = match spec {
            PaneSpec::Terminal(spec) => {
                let resolved = resolve_terminal_profile(
                    &spec.launch_profile_id,
                    spec.command_override.clone(),
                    &preferences.default_custom_command,
                )
                .map_err(settings_error_to_shell)?;
                let runtime_session_id = self.pty_manager.spawn(
                    pane_id,
                    &spec.working_directory,
                    resolved.command.as_deref(),
                )?;
                let runtime = self
                    .runtimes
                    .lock()
                    .map_err(|_| ShellError::State(String::from("Runtime lock poisoned")))?
                    .register_terminal(pane_id, runtime_session_id);
                pane_runtime_to_view(&runtime)
            }
            PaneSpec::Browser(spec) => {
                let runtime = self
                    .runtimes
                    .lock()
                    .map_err(|_| ShellError::State(String::from("Runtime lock poisoned")))?
                    .register_browser(
                        pane_id,
                        format!("browser-{}", uuid::Uuid::new_v4()),
                        spec.initial_url.clone(),
                    );
                pane_runtime_to_view(&runtime)
            }
        };
        self.emit_runtime_status(&runtime_view);
        Ok(())
    }

    fn stop_runtime_for_pane(&self, pane_id: &str) {
        let runtime = match self
            .runtimes
            .lock()
            .map_err(|_| ShellError::State(String::from("Runtime lock poisoned")))
        {
            Ok(mut runtimes) => runtimes.remove(pane_id),
            Err(error) => {
                warn!(?error, "Failed to lock runtime registry during stop");
                None
            }
        };

        if let Some(runtime) = runtime {
            if let Some(runtime_session_id) = runtime.runtime_session_id.clone() {
                match runtime.kind {
                    tabby_runtime::RuntimeKind::Terminal => {
                        if let Err(error) = self.pty_manager.kill(&runtime_session_id) {
                            warn!(?error, pane_id, "Failed to kill terminal runtime");
                        }
                    }
                    tabby_runtime::RuntimeKind::Browser => {
                        if let Some(window) = self.app.get_webview_window("main") {
                            if let Some(webview) =
                                window.get_webview(&browser_surface::webview_label(pane_id))
                            {
                                if let Err(error) = webview.close() {
                                    warn!(?error, pane_id, "Failed to close browser surface");
                                }
                            }
                        }
                    }
                }
            }

            let mut exited = runtime;
            exited.status = RuntimeStatus::Exited;
            self.emit_runtime_status(&pane_runtime_to_view(&exited));
        }
    }

    fn emit_workspace_projection(&self, workspace: &WorkspaceView) {
        self.publisher.emit_workspace_projection(workspace);
    }

    fn emit_settings_projection(&self, settings: &SettingsView) {
        self.publisher.emit_settings_projection(settings);
    }

    fn emit_runtime_status(&self, runtime: &tabby_contracts::PaneRuntimeView) {
        self.publisher.emit_runtime_status(runtime);
    }
}

fn settings_error_to_shell(error: SettingsError) -> ShellError {
    match error {
        SettingsError::Validation(message) => ShellError::Validation(message),
    }
}
