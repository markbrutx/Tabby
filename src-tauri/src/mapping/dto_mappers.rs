use std::path::PathBuf;

use tabby_contracts::{
    BlameEntryDto, BranchInfoDto, CommitInfoDto, DiffContentDto, DiffHunkDto, DiffLineDto,
    DiffLineKindDto, FileStatusDto, FileStatusKindDto, GitCommandDto, GitRepoStateDto,
    GitResultDto, LayoutPresetDto, PaneRuntimeView, PaneSpecDto, PaneView, ProfileCatalogView,
    ProfileView, RuntimeCommandDto, RuntimeKindDto, RuntimeStatusDto, SettingsCommandDto,
    SettingsView, SplitDirectionDto, SplitNodeDto, StashEntryDto, TabView, ThemeModeDto,
    WorkspaceBootstrapView, WorkspaceCommandDto, WorkspaceView,
};
use tabby_git::value_objects::{BranchName, RemoteName, StashId};
use tabby_git::{
    BlameEntry, BranchInfo, CommitInfo, DiffContent, DiffHunk, DiffLine, DiffLineKind, FileStatus,
    FileStatusKind, GitRepositoryState, StashEntry,
};
use tabby_runtime::{PaneRuntime, RuntimeKind, RuntimeStatus};
use tabby_settings::{
    FontSize, ProfileCatalog, ProfileId, SettingsError, ThemeMode, UserPreferences,
    WorkingDirectory,
};
use tabby_workspace::layout::{LayoutPreset, SplitDirection, SplitNode};
use tabby_workspace::{PaneContentDefinition, PaneId, PaneSpec, TabId, WorkspaceSession};

use crate::application::commands::{
    CloseTabCommand, GitCommand, GitResult, OpenTabCommand, ReplacePaneSpecCommand, RuntimeCommand,
    SettingsCommand, SplitPaneCommand, UpdateSettingsCommand, WorkspaceCommand,
};
use crate::shell::error::ShellError;

// ---------------------------------------------------------------------------
// Domain → DTO (outbound / projections)
// ---------------------------------------------------------------------------

pub fn settings_view_from_preferences(preferences: &UserPreferences) -> SettingsView {
    SettingsView {
        default_layout: layout_preset_to_dto(preferences.default_layout),
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
        default_layout: layout_preset_from_dto(view.default_layout),
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
                startup_command_template: profile
                    .startup_command_template
                    .as_ref()
                    .map(|c| c.as_str().to_string()),
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
fn pane_spec_to_dto(value: &PaneSpec) -> PaneSpecDto {
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

pub fn pane_runtime_to_view(runtime: &PaneRuntime) -> PaneRuntimeView {
    PaneRuntimeView {
        pane_id: runtime.pane_id.to_string(),
        runtime_session_id: runtime.runtime_session_id.as_ref().map(|id| id.to_string()),
        kind: runtime_kind_to_dto(runtime.kind),
        status: runtime_status_to_dto(runtime.status),
        last_error: runtime.last_error.clone(),
        browser_location: runtime
            .browser_location
            .as_ref()
            .map(|u| u.as_str().to_string()),
        terminal_cwd: runtime
            .terminal_cwd
            .as_ref()
            .map(|w| w.as_str().to_string()),
        git_repo_path: runtime
            .git_repo_path
            .as_ref()
            .map(|w| w.as_str().to_string()),
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
// Git: DTO → Domain (inbound / commands)
// ---------------------------------------------------------------------------

/// Extracts the `pane_id` string from any `GitCommandDto` variant.
pub fn extract_git_pane_id(dto: &GitCommandDto) -> String {
    match dto {
        GitCommandDto::Status { pane_id, .. }
        | GitCommandDto::Diff { pane_id, .. }
        | GitCommandDto::Stage { pane_id, .. }
        | GitCommandDto::Unstage { pane_id, .. }
        | GitCommandDto::StageLines { pane_id, .. }
        | GitCommandDto::Commit { pane_id, .. }
        | GitCommandDto::Push { pane_id, .. }
        | GitCommandDto::Pull { pane_id, .. }
        | GitCommandDto::Fetch { pane_id, .. }
        | GitCommandDto::Branches { pane_id, .. }
        | GitCommandDto::CheckoutBranch { pane_id, .. }
        | GitCommandDto::CreateBranch { pane_id, .. }
        | GitCommandDto::DeleteBranch { pane_id, .. }
        | GitCommandDto::MergeBranch { pane_id, .. }
        | GitCommandDto::Log { pane_id, .. }
        | GitCommandDto::ShowCommit { pane_id, .. }
        | GitCommandDto::Blame { pane_id, .. }
        | GitCommandDto::StashPush { pane_id, .. }
        | GitCommandDto::StashPop { pane_id, .. }
        | GitCommandDto::StashList { pane_id, .. }
        | GitCommandDto::StashDrop { pane_id, .. }
        | GitCommandDto::DiscardChanges { pane_id, .. }
        | GitCommandDto::RepoState { pane_id, .. } => pane_id.clone(),
    }
}

/// Maps a `GitCommandDto` (transport) into a `GitCommand` (domain).
///
/// The `repo_path` is resolved externally (e.g. from the pane's working directory)
/// because `GitCommandDto` carries only a `pane_id`.
pub fn git_command_from_dto(
    dto: GitCommandDto,
    repo_path: PathBuf,
) -> Result<GitCommand, ShellError> {
    let cmd = match dto {
        GitCommandDto::Status { .. } => GitCommand::Status { repo_path },
        GitCommandDto::Diff { staged, .. } => GitCommand::Diff { repo_path, staged },
        GitCommandDto::Stage { paths, .. } => GitCommand::Stage { repo_path, paths },
        GitCommandDto::Unstage { paths, .. } => GitCommand::Unstage { repo_path, paths },
        GitCommandDto::StageLines {
            path, line_ranges, ..
        } => {
            let parsed = line_ranges
                .iter()
                .map(|r| parse_line_range(r))
                .collect::<Result<Vec<_>, _>>()?;
            GitCommand::StageLines {
                repo_path,
                file_path: path,
                line_ranges: parsed,
            }
        }
        GitCommandDto::Commit { message, amend, .. } => GitCommand::Commit {
            repo_path,
            message,
            amend,
        },
        GitCommandDto::Push { remote, branch, .. } => {
            let remote = remote_name_or_default(remote.as_deref())?;
            let branch = branch_name_required(branch.as_deref(), "Push requires a branch name")?;
            GitCommand::Push {
                repo_path,
                remote,
                branch,
            }
        }
        GitCommandDto::Pull { remote, branch, .. } => {
            let remote = remote_name_or_default(remote.as_deref())?;
            let branch = branch_name_required(branch.as_deref(), "Pull requires a branch name")?;
            GitCommand::Pull {
                repo_path,
                remote,
                branch,
            }
        }
        GitCommandDto::Fetch { remote, .. } => {
            let remote = remote_name_or_default(remote.as_deref())?;
            GitCommand::Fetch { repo_path, remote }
        }
        GitCommandDto::Branches { .. } => GitCommand::Branches { repo_path },
        GitCommandDto::CheckoutBranch { name, .. } => {
            let branch =
                BranchName::try_new(&name).map_err(|e| ShellError::Validation(e.to_string()))?;
            GitCommand::CheckoutBranch { repo_path, branch }
        }
        GitCommandDto::CreateBranch {
            name, start_point, ..
        } => {
            let branch =
                BranchName::try_new(&name).map_err(|e| ShellError::Validation(e.to_string()))?;
            let start_point = start_point
                .map(|sp| {
                    BranchName::try_new(&sp).map_err(|e| ShellError::Validation(e.to_string()))
                })
                .transpose()?;
            GitCommand::CreateBranch {
                repo_path,
                branch,
                start_point,
            }
        }
        GitCommandDto::DeleteBranch { name, force, .. } => {
            let branch =
                BranchName::try_new(&name).map_err(|e| ShellError::Validation(e.to_string()))?;
            GitCommand::DeleteBranch {
                repo_path,
                branch,
                force,
            }
        }
        GitCommandDto::MergeBranch { name, .. } => {
            let branch =
                BranchName::try_new(&name).map_err(|e| ShellError::Validation(e.to_string()))?;
            GitCommand::MergeBranch { repo_path, branch }
        }
        GitCommandDto::Log {
            max_count, skip, ..
        } => GitCommand::Log {
            repo_path,
            max_count: max_count.unwrap_or(50),
            skip: skip.unwrap_or(0),
        },
        GitCommandDto::ShowCommit { hash, .. } => GitCommand::ShowCommit { repo_path, hash },
        GitCommandDto::Blame { path, .. } => GitCommand::Blame {
            repo_path,
            file_path: path,
        },
        GitCommandDto::StashPush { message, .. } => GitCommand::StashPush { repo_path, message },
        GitCommandDto::StashPop { .. } => GitCommand::StashPop { repo_path },
        GitCommandDto::StashList { .. } => GitCommand::StashList { repo_path },
        GitCommandDto::StashDrop { index, .. } => GitCommand::StashDrop {
            repo_path,
            stash_id: StashId::new(index as usize),
        },
        GitCommandDto::DiscardChanges { paths, .. } => {
            GitCommand::DiscardChanges { repo_path, paths }
        }
        GitCommandDto::RepoState { .. } => GitCommand::RepoState { repo_path },
    };
    Ok(cmd)
}

// ---------------------------------------------------------------------------
// Git: Domain → DTO (outbound / results)
// ---------------------------------------------------------------------------

/// Maps a `GitResult` (domain) into a `GitResultDto` (transport).
pub fn git_result_to_dto(result: GitResult) -> GitResultDto {
    match result {
        GitResult::Status(files) => GitResultDto::Status {
            files: files.iter().map(file_status_to_dto).collect(),
        },
        GitResult::Diff(diffs) => GitResultDto::Diff {
            diffs: diffs.iter().map(diff_content_to_dto).collect(),
        },
        GitResult::Stage => GitResultDto::Stage,
        GitResult::Unstage => GitResultDto::Unstage,
        GitResult::StageLines => GitResultDto::StageLines,
        GitResult::Commit(info) => GitResultDto::Commit {
            hash: info.short_hash().to_string(),
        },
        GitResult::Push => GitResultDto::Push,
        GitResult::Pull => GitResultDto::Pull,
        GitResult::Fetch => GitResultDto::Fetch,
        GitResult::Branches(branches) => GitResultDto::Branches {
            branches: branches.iter().map(branch_info_to_dto).collect(),
        },
        GitResult::CheckoutBranch => GitResultDto::CheckoutBranch,
        GitResult::CreateBranch => GitResultDto::CreateBranch,
        GitResult::DeleteBranch => GitResultDto::DeleteBranch,
        GitResult::MergeBranch => GitResultDto::MergeBranch {
            message: String::new(),
        },
        GitResult::Log(commits) => GitResultDto::Log {
            commits: commits.iter().map(commit_info_to_dto).collect(),
        },
        GitResult::ShowCommit(diffs) => GitResultDto::ShowCommit {
            diffs: diffs.iter().map(diff_content_to_dto).collect(),
        },
        GitResult::Blame(entries) => GitResultDto::Blame {
            entries: entries.iter().map(blame_entry_to_dto).collect(),
        },
        GitResult::StashPush => GitResultDto::StashPush,
        GitResult::StashPop => GitResultDto::StashPop,
        GitResult::StashList(entries) => GitResultDto::StashList {
            entries: entries.iter().map(stash_entry_to_dto).collect(),
        },
        GitResult::StashDrop => GitResultDto::StashDrop,
        GitResult::DiscardChanges => GitResultDto::DiscardChanges,
        GitResult::RepoState(state) => GitResultDto::RepoState {
            state: git_repo_state_to_dto(&state),
        },
    }
}

// ---------------------------------------------------------------------------
// Git type mappers: Domain → DTO
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn file_status_to_dto(status: &FileStatus) -> FileStatusDto {
    FileStatusDto {
        path: status.path().to_string(),
        old_path: status.old_path().map(|s| s.to_string()),
        index_status: file_status_kind_to_dto(status.index_status()),
        worktree_status: file_status_kind_to_dto(status.worktree_status()),
    }
}

#[allow(dead_code)]
pub fn file_status_kind_to_dto(kind: FileStatusKind) -> FileStatusKindDto {
    match kind {
        FileStatusKind::Modified => FileStatusKindDto::Modified,
        FileStatusKind::Added => FileStatusKindDto::Added,
        FileStatusKind::Deleted => FileStatusKindDto::Deleted,
        FileStatusKind::Renamed => FileStatusKindDto::Renamed,
        FileStatusKind::Copied => FileStatusKindDto::Copied,
        FileStatusKind::Untracked => FileStatusKindDto::Untracked,
        FileStatusKind::Ignored => FileStatusKindDto::Ignored,
        FileStatusKind::Conflicted => FileStatusKindDto::Conflicted,
    }
}

#[allow(dead_code)]
pub fn diff_content_to_dto(diff: &DiffContent) -> DiffContentDto {
    DiffContentDto {
        file_path: diff.file_path().to_string(),
        old_path: diff.old_path().map(|s| s.to_string()),
        hunks: diff.hunks().iter().map(diff_hunk_to_dto).collect(),
        is_binary: diff.is_binary(),
        file_mode_change: diff.file_mode_change().map(|s| s.to_string()),
    }
}

#[allow(dead_code)]
pub fn diff_hunk_to_dto(hunk: &DiffHunk) -> DiffHunkDto {
    DiffHunkDto {
        old_start: hunk.old_start(),
        old_count: hunk.old_count(),
        new_start: hunk.new_start(),
        new_count: hunk.new_count(),
        header: hunk.header().to_string(),
        lines: hunk.lines().iter().map(diff_line_to_dto).collect(),
    }
}

#[allow(dead_code)]
pub fn diff_line_to_dto(line: &DiffLine) -> DiffLineDto {
    DiffLineDto {
        kind: diff_line_kind_to_dto(line.kind()),
        old_line_no: line.old_line_no(),
        new_line_no: line.new_line_no(),
        content: line.content().to_string(),
    }
}

#[allow(dead_code)]
pub fn diff_line_kind_to_dto(kind: DiffLineKind) -> DiffLineKindDto {
    match kind {
        DiffLineKind::Context => DiffLineKindDto::Context,
        DiffLineKind::Addition => DiffLineKindDto::Addition,
        DiffLineKind::Deletion => DiffLineKindDto::Deletion,
        DiffLineKind::HunkHeader => DiffLineKindDto::HunkHeader,
    }
}

#[allow(dead_code)]
pub fn commit_info_to_dto(info: &CommitInfo) -> CommitInfoDto {
    CommitInfoDto {
        hash: info.hash().to_string(),
        short_hash: info.short_hash().to_string(),
        author_name: info.author_name().to_string(),
        author_email: info.author_email().to_string(),
        date: info.date().to_string(),
        message: info.message().to_string(),
        parent_hashes: info.parent_hashes().iter().map(|h| h.to_string()).collect(),
    }
}

#[allow(dead_code)]
pub fn branch_info_to_dto(branch: &BranchInfo) -> BranchInfoDto {
    BranchInfoDto {
        name: branch.name().as_ref().to_string(),
        is_current: branch.is_current(),
        upstream: branch.upstream().map(|s| s.to_string()),
        ahead: branch.ahead(),
        behind: branch.behind(),
    }
}

#[allow(dead_code)]
pub fn blame_entry_to_dto(entry: &BlameEntry) -> BlameEntryDto {
    BlameEntryDto {
        hash: entry.hash().to_string(),
        author: entry.author().to_string(),
        date: entry.date().to_string(),
        line_start: entry.line_start(),
        line_count: entry.line_count(),
        content: entry.content().to_string(),
    }
}

#[allow(dead_code)]
pub fn stash_entry_to_dto(entry: &StashEntry) -> StashEntryDto {
    StashEntryDto {
        index: entry.index().index() as u32,
        message: entry.message().to_string(),
        date: entry.date().to_string(),
    }
}

#[allow(dead_code)]
pub fn git_repo_state_to_dto(state: &GitRepositoryState) -> GitRepoStateDto {
    GitRepoStateDto {
        repo_path: state.repo_path().as_str().to_string(),
        head_branch: state.head_branch().map(|b| b.as_ref().to_string()),
        is_detached: state.is_detached(),
        status_clean: state.status_clean(),
    }
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
        RuntimeKind::Git => RuntimeKindDto::Git,
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

/// Parse a line-range string like `"10-20"` into a `(u32, u32)` tuple.
#[allow(dead_code)]
fn parse_line_range(s: &str) -> Result<(u32, u32), ShellError> {
    let parts: Vec<&str> = s.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(ShellError::Validation(format!(
            "Invalid line range '{s}': expected format 'start-end'"
        )));
    }
    let start: u32 = parts[0]
        .parse()
        .map_err(|_| ShellError::Validation(format!("Invalid line range start in '{s}'")))?;
    let end: u32 = parts[1]
        .parse()
        .map_err(|_| ShellError::Validation(format!("Invalid line range end in '{s}'")))?;
    Ok((start, end))
}

/// Resolve an optional remote name to a `RemoteName`, defaulting to `"origin"`.
#[allow(dead_code)]
fn remote_name_or_default(name: Option<&str>) -> Result<RemoteName, ShellError> {
    let raw = name.unwrap_or("origin");
    RemoteName::try_new(raw).map_err(|e| ShellError::Validation(e.to_string()))
}

/// Resolve an optional branch name, returning an error with `context` if `None`.
#[allow(dead_code)]
fn branch_name_required(name: Option<&str>, context: &str) -> Result<BranchName, ShellError> {
    let raw = name.ok_or_else(|| ShellError::Validation(context.to_string()))?;
    BranchName::try_new(raw).map_err(|e| ShellError::Validation(e.to_string()))
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tabby_contracts::PaneId;
    use tabby_kernel::WorkingDirectory;
    use tabby_runtime::{PaneRuntime, RuntimeKind, RuntimeSessionId, RuntimeStatus};
    use tabby_settings::{default_preferences, ProfileCatalog, TerminalProfile, UserPreferences};
    use tabby_workspace::{
        BrowserPaneSpec, BrowserUrl, CommandTemplate, PaneSpec, TerminalPaneSpec,
    };

    // -- PaneSpec round-trip ------------------------------------------------

    #[test]
    fn terminal_pane_spec_round_trips_through_dto() {
        let spec = PaneSpec::Terminal(TerminalPaneSpec {
            launch_profile_id: String::from("claude"),
            working_directory: String::from("/home/user"),
            command_override: Some(CommandTemplate::new("bash")),
        });

        let dto = pane_spec_to_dto(&spec);
        let restored = pane_spec_from_dto(dto);

        match restored {
            PaneSpec::Terminal(t) => {
                assert_eq!(t.launch_profile_id, "claude");
                assert_eq!(t.working_directory, "/home/user");
                assert_eq!(
                    t.command_override.as_ref().map(|c| c.as_str()),
                    Some("bash")
                );
            }
            other => panic!("Expected Terminal spec, got {other:?}"),
        }
    }

    #[test]
    fn browser_pane_spec_round_trips_through_dto() {
        let spec = PaneSpec::Browser(BrowserPaneSpec {
            initial_url: BrowserUrl::new("https://example.com"),
        });

        let dto = pane_spec_to_dto(&spec);
        let restored = pane_spec_from_dto(dto);

        match restored {
            PaneSpec::Browser(b) => {
                assert_eq!(b.initial_url.as_str(), "https://example.com");
            }
            other => panic!("Expected Browser spec, got {other:?}"),
        }
    }

    // -- SettingsView <-> UserPreferences round-trip ------------------------

    #[test]
    fn settings_round_trip_preserves_all_fields() {
        let preferences = UserPreferences {
            default_layout: LayoutPreset::TwoByTwo,
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

        assert_eq!(restored.default_layout, LayoutPreset::TwoByTwo);
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
            pane_id: PaneId::from(String::from("pane-1")),
            runtime_session_id: Some(RuntimeSessionId::from(String::from("pty-abc"))),
            kind: RuntimeKind::Terminal,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: None,
            terminal_cwd: None,
            git_repo_path: None,
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
            pane_id: PaneId::from(String::from("pane-2")),
            runtime_session_id: Some(RuntimeSessionId::from(String::from("browser-xyz"))),
            kind: RuntimeKind::Browser,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: Some(BrowserUrl::new("https://example.com")),
            terminal_cwd: None,
            git_repo_path: None,
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
            pane_id: PaneId::from(String::from("pane-3")),
            runtime_session_id: None,
            kind: RuntimeKind::Terminal,
            status: RuntimeStatus::Failed,
            last_error: Some(String::from("spawn failed")),
            browser_location: None,
            terminal_cwd: None,
            git_repo_path: None,
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
                    startup_command_template: Some(CommandTemplate::new("claude")),
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
                        assert_eq!(
                            t.command_override.as_ref().map(|c| c.as_str()),
                            Some("codex")
                        );
                    }
                    other => panic!("Expected Terminal, got {other:?}"),
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
            pane_id: PaneId::from(String::from("pane-1")),
            runtime_session_id: Some(RuntimeSessionId::from(wire_session.clone())),
            kind: RuntimeKind::Terminal,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: None,
            terminal_cwd: None,
            git_repo_path: None,
        };
        let view = pane_runtime_to_view(&runtime);
        assert_eq!(
            view.runtime_session_id.as_deref(),
            Some(wire_session.as_str())
        );
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

    // -----------------------------------------------------------------------
    // Git DTO mapping tests
    // -----------------------------------------------------------------------

    fn test_repo() -> PathBuf {
        PathBuf::from("/tmp/test-repo")
    }

    // -- git_command_from_dto -----------------------------------------------

    #[test]
    fn git_command_from_dto_status() {
        let dto = GitCommandDto::Status {
            pane_id: "p1".to_string(),
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::Status { repo_path } => assert_eq!(repo_path, test_repo()),
            other => panic!("Expected Status, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_diff() {
        let dto = GitCommandDto::Diff {
            pane_id: "p1".to_string(),
            path: Some("file.rs".to_string()),
            staged: true,
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::Diff { repo_path, staged } => {
                assert_eq!(repo_path, test_repo());
                assert!(staged);
            }
            other => panic!("Expected Diff, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_stage() {
        let dto = GitCommandDto::Stage {
            pane_id: "p1".to_string(),
            paths: vec!["a.rs".to_string(), "b.rs".to_string()],
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::Stage { paths, .. } => {
                assert_eq!(paths, vec!["a.rs", "b.rs"]);
            }
            other => panic!("Expected Stage, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_stage_lines_parses_ranges() {
        let dto = GitCommandDto::StageLines {
            pane_id: "p1".to_string(),
            path: "file.rs".to_string(),
            line_ranges: vec!["1-5".to_string(), "10-20".to_string()],
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::StageLines {
                file_path,
                line_ranges,
                ..
            } => {
                assert_eq!(file_path, "file.rs");
                assert_eq!(line_ranges, vec![(1, 5), (10, 20)]);
            }
            other => panic!("Expected StageLines, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_stage_lines_rejects_bad_range() {
        let dto = GitCommandDto::StageLines {
            pane_id: "p1".to_string(),
            path: "f.rs".to_string(),
            line_ranges: vec!["bad".to_string()],
        };
        let err = git_command_from_dto(dto, test_repo()).expect_err("should reject");
        assert!(err.to_string().contains("line range"));
    }

    #[test]
    fn git_command_from_dto_commit() {
        let dto = GitCommandDto::Commit {
            pane_id: "p1".to_string(),
            message: "feat: hello".to_string(),
            amend: false,
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::Commit { message, amend, .. } => {
                assert_eq!(message, "feat: hello");
                assert!(!amend);
            }
            other => panic!("Expected Commit, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_push_with_remote_and_branch() {
        let dto = GitCommandDto::Push {
            pane_id: "p1".to_string(),
            remote: Some("upstream".to_string()),
            branch: Some("main".to_string()),
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::Push { remote, branch, .. } => {
                assert_eq!(remote.as_ref(), "upstream");
                assert_eq!(branch.as_ref(), "main");
            }
            other => panic!("Expected Push, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_push_defaults_remote_to_origin() {
        let dto = GitCommandDto::Push {
            pane_id: "p1".to_string(),
            remote: None,
            branch: Some("main".to_string()),
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::Push { remote, .. } => assert_eq!(remote.as_ref(), "origin"),
            other => panic!("Expected Push, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_push_requires_branch() {
        let dto = GitCommandDto::Push {
            pane_id: "p1".to_string(),
            remote: None,
            branch: None,
        };
        let err = git_command_from_dto(dto, test_repo()).expect_err("should reject");
        assert!(err.to_string().contains("branch"));
    }

    #[test]
    fn git_command_from_dto_fetch_defaults_remote() {
        let dto = GitCommandDto::Fetch {
            pane_id: "p1".to_string(),
            remote: None,
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::Fetch { remote, .. } => assert_eq!(remote.as_ref(), "origin"),
            other => panic!("Expected Fetch, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_checkout_branch() {
        let dto = GitCommandDto::CheckoutBranch {
            pane_id: "p1".to_string(),
            name: "feature/test".to_string(),
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::CheckoutBranch { branch, .. } => {
                assert_eq!(branch.as_ref(), "feature/test");
            }
            other => panic!("Expected CheckoutBranch, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_log_defaults_max_count() {
        let dto = GitCommandDto::Log {
            pane_id: "p1".to_string(),
            max_count: None,
            skip: None,
            path: None,
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::Log {
                max_count, skip, ..
            } => {
                assert_eq!(max_count, 50);
                assert_eq!(skip, 0);
            }
            other => panic!("Expected Log, got {other:?}"),
        }
    }

    #[test]
    fn git_command_from_dto_stash_drop() {
        let dto = GitCommandDto::StashDrop {
            pane_id: "p1".to_string(),
            index: 3,
        };
        let cmd = git_command_from_dto(dto, test_repo()).expect("should map");
        match cmd {
            GitCommand::StashDrop { stash_id, .. } => assert_eq!(stash_id.index(), 3),
            other => panic!("Expected StashDrop, got {other:?}"),
        }
    }

    // -- FileStatus → FileStatusDto -----------------------------------------

    #[test]
    fn file_status_to_dto_maps_all_fields() {
        let status = FileStatus::new(
            "src/main.rs",
            Some("src/old.rs".to_string()),
            FileStatusKind::Renamed,
            FileStatusKind::Modified,
        );
        let dto = file_status_to_dto(&status);
        assert_eq!(dto.path, "src/main.rs");
        assert_eq!(dto.old_path.as_deref(), Some("src/old.rs"));
        assert_eq!(dto.index_status, FileStatusKindDto::Renamed);
        assert_eq!(dto.worktree_status, FileStatusKindDto::Modified);
    }

    #[test]
    fn file_status_kind_to_dto_maps_all_variants() {
        let pairs = [
            (FileStatusKind::Modified, FileStatusKindDto::Modified),
            (FileStatusKind::Added, FileStatusKindDto::Added),
            (FileStatusKind::Deleted, FileStatusKindDto::Deleted),
            (FileStatusKind::Renamed, FileStatusKindDto::Renamed),
            (FileStatusKind::Copied, FileStatusKindDto::Copied),
            (FileStatusKind::Untracked, FileStatusKindDto::Untracked),
            (FileStatusKind::Ignored, FileStatusKindDto::Ignored),
            (FileStatusKind::Conflicted, FileStatusKindDto::Conflicted),
        ];
        for (domain, expected_dto) in pairs {
            assert_eq!(file_status_kind_to_dto(domain), expected_dto);
        }
    }

    // -- DiffContent → DiffContentDto ---------------------------------------

    #[test]
    fn diff_content_to_dto_maps_complete_diff() {
        use tabby_git::{DiffContent, DiffHunk, DiffLine, DiffLineKind};

        let line = DiffLine::new(DiffLineKind::Addition, None, Some(1), "new line");
        let hunk = DiffHunk::new(0, 0, 1, 1, "@@ -0,0 +1,1 @@", vec![line]);
        let diff = DiffContent::new(
            "src/lib.rs",
            Some("src/old_lib.rs".to_string()),
            vec![hunk],
            false,
            Some("100644 -> 100755".to_string()),
        );

        let dto = diff_content_to_dto(&diff);
        assert_eq!(dto.file_path, "src/lib.rs");
        assert_eq!(dto.old_path.as_deref(), Some("src/old_lib.rs"));
        assert!(!dto.is_binary);
        assert_eq!(dto.file_mode_change.as_deref(), Some("100644 -> 100755"));
        assert_eq!(dto.hunks.len(), 1);
        assert_eq!(dto.hunks[0].old_start, 0);
        assert_eq!(dto.hunks[0].new_count, 1);
        assert_eq!(dto.hunks[0].lines.len(), 1);
        assert_eq!(dto.hunks[0].lines[0].kind, DiffLineKindDto::Addition);
        assert_eq!(dto.hunks[0].lines[0].content, "new line");
    }

    // -- CommitInfo → CommitInfoDto -----------------------------------------

    #[test]
    fn commit_info_to_dto_maps_all_fields() {
        use tabby_git::value_objects::CommitHash;

        let hash = CommitHash::try_new("abc123def456").expect("valid");
        let parent = CommitHash::try_new("1111aaaa").expect("valid");
        let info = CommitInfo::new(
            hash,
            "abc123d".to_string(),
            "Alice".to_string(),
            "alice@test.com".to_string(),
            "2026-03-10".to_string(),
            "feat: test".to_string(),
            vec![parent],
        );
        let dto = commit_info_to_dto(&info);
        assert_eq!(dto.hash, "abc123def456");
        assert_eq!(dto.short_hash, "abc123d");
        assert_eq!(dto.author_name, "Alice");
        assert_eq!(dto.author_email, "alice@test.com");
        assert_eq!(dto.message, "feat: test");
        assert_eq!(dto.parent_hashes, vec!["1111aaaa"]);
    }

    // -- BranchInfo → BranchInfoDto -----------------------------------------

    #[test]
    fn branch_info_to_dto_maps_all_fields() {
        let branch = BranchInfo::new(
            BranchName::try_new("main").expect("valid"),
            true,
            Some("origin/main".to_string()),
            3,
            1,
        );
        let dto = branch_info_to_dto(&branch);
        assert_eq!(dto.name, "main");
        assert!(dto.is_current);
        assert_eq!(dto.upstream.as_deref(), Some("origin/main"));
        assert_eq!(dto.ahead, 3);
        assert_eq!(dto.behind, 1);
    }

    // -- BlameEntry → BlameEntryDto -----------------------------------------

    #[test]
    fn blame_entry_to_dto_maps_all_fields() {
        use tabby_git::value_objects::CommitHash;

        let entry = BlameEntry::new(
            CommitHash::try_new("deadbeef").expect("valid"),
            "Alice".to_string(),
            "2026-03-10".to_string(),
            1,
            5,
            "fn main() {}".to_string(),
        );
        let dto = blame_entry_to_dto(&entry);
        assert_eq!(dto.hash, "deadbeef");
        assert_eq!(dto.author, "Alice");
        assert_eq!(dto.line_start, 1);
        assert_eq!(dto.line_count, 5);
    }

    // -- StashEntry → StashEntryDto -----------------------------------------

    #[test]
    fn stash_entry_to_dto_maps_all_fields() {
        let entry = StashEntry::new(
            StashId::new(2),
            "WIP on main".to_string(),
            "2026-03-10".to_string(),
        );
        let dto = stash_entry_to_dto(&entry);
        assert_eq!(dto.index, 2);
        assert_eq!(dto.message, "WIP on main");
        assert_eq!(dto.date, "2026-03-10");
    }

    // -- GitRepositoryState → GitRepoStateDto -------------------------------

    #[test]
    fn git_repo_state_to_dto_maps_all_fields() {
        let state = GitRepositoryState::new(
            WorkingDirectory::new("/home/user/project").expect("valid"),
            Some(BranchName::try_new("main").expect("valid")),
            false,
            true,
        );
        let dto = git_repo_state_to_dto(&state);
        assert_eq!(dto.repo_path, "/home/user/project");
        assert_eq!(dto.head_branch.as_deref(), Some("main"));
        assert!(!dto.is_detached);
        assert!(dto.status_clean);
    }

    #[test]
    fn git_repo_state_to_dto_detached_head() {
        let state = GitRepositoryState::new(
            WorkingDirectory::new("/repo").expect("valid"),
            None,
            true,
            false,
        );
        let dto = git_repo_state_to_dto(&state);
        assert!(dto.head_branch.is_none());
        assert!(dto.is_detached);
        assert!(!dto.status_clean);
    }

    // -- git_result_to_dto round-trip tests ---------------------------------

    #[test]
    fn git_result_to_dto_status_maps_file_statuses() {
        let files = vec![FileStatus::new(
            "README.md",
            None,
            FileStatusKind::Modified,
            FileStatusKind::Modified,
        )];
        let dto = git_result_to_dto(GitResult::Status(files));
        match dto {
            GitResultDto::Status { files } => {
                assert_eq!(files.len(), 1);
                assert_eq!(files[0].path, "README.md");
            }
            other => panic!("Expected Status, got {other:?}"),
        }
    }

    #[test]
    fn git_result_to_dto_unit_variants() {
        assert!(matches!(
            git_result_to_dto(GitResult::Stage),
            GitResultDto::Stage
        ));
        assert!(matches!(
            git_result_to_dto(GitResult::Unstage),
            GitResultDto::Unstage
        ));
        assert!(matches!(
            git_result_to_dto(GitResult::Push),
            GitResultDto::Push
        ));
        assert!(matches!(
            git_result_to_dto(GitResult::Pull),
            GitResultDto::Pull
        ));
        assert!(matches!(
            git_result_to_dto(GitResult::Fetch),
            GitResultDto::Fetch
        ));
        assert!(matches!(
            git_result_to_dto(GitResult::CheckoutBranch),
            GitResultDto::CheckoutBranch
        ));
        assert!(matches!(
            git_result_to_dto(GitResult::DiscardChanges),
            GitResultDto::DiscardChanges
        ));
    }

    #[test]
    fn git_result_to_dto_commit_uses_short_hash() {
        use tabby_git::value_objects::CommitHash;

        let info = CommitInfo::new(
            CommitHash::try_new("abc1234def5678").expect("valid"),
            "abc1234".to_string(),
            "Test".to_string(),
            "test@test.com".to_string(),
            "2026-03-10".to_string(),
            "feat: test".to_string(),
            vec![],
        );
        let dto = git_result_to_dto(GitResult::Commit(info));
        match dto {
            GitResultDto::Commit { hash } => assert_eq!(hash, "abc1234"),
            other => panic!("Expected Commit, got {other:?}"),
        }
    }

    #[test]
    fn git_result_to_dto_branches() {
        let branches = vec![BranchInfo::new(
            BranchName::try_new("develop").expect("valid"),
            false,
            None,
            0,
            0,
        )];
        let dto = git_result_to_dto(GitResult::Branches(branches));
        match dto {
            GitResultDto::Branches { branches } => {
                assert_eq!(branches.len(), 1);
                assert_eq!(branches[0].name, "develop");
            }
            other => panic!("Expected Branches, got {other:?}"),
        }
    }

    #[test]
    fn git_result_to_dto_repo_state() {
        let state = GitRepositoryState::new(
            WorkingDirectory::new("/repo").expect("valid"),
            Some(BranchName::try_new("main").expect("valid")),
            false,
            true,
        );
        let dto = git_result_to_dto(GitResult::RepoState(state));
        match dto {
            GitResultDto::RepoState { state } => {
                assert_eq!(state.repo_path, "/repo");
                assert!(state.status_clean);
            }
            other => panic!("Expected RepoState, got {other:?}"),
        }
    }

    // -- Git PaneSpec round-trip tests ---------------------------------------

    #[test]
    fn git_pane_spec_round_trips_through_dto() {
        let spec = PaneSpec::Git(tabby_workspace::GitPaneSpec {
            working_directory: String::from("/home/user/project"),
        });

        let dto = pane_spec_to_dto(&spec);
        let restored = pane_spec_from_dto(dto);

        match restored {
            PaneSpec::Git(g) => {
                assert_eq!(g.working_directory, "/home/user/project");
            }
            other => panic!("Expected Git spec, got {other:?}"),
        }
    }

    #[test]
    fn pane_spec_from_dto_git_maps_working_directory() {
        let dto = PaneSpecDto::Git {
            working_directory: String::from("/repos/my-project"),
        };
        let spec = pane_spec_from_dto(dto);
        match spec {
            PaneSpec::Git(g) => assert_eq!(g.working_directory, "/repos/my-project"),
            other => panic!("Expected Git spec, got {other:?}"),
        }
    }

    // -- Git PaneContentDefinition → PaneSpecDto ----------------------------

    #[test]
    fn pane_content_to_spec_dto_maps_git_content() {
        use tabby_workspace::{PaneContentDefinition, PaneContentId};

        let content = PaneContentDefinition::git(
            PaneContentId::from(String::from("cid-1")),
            "/home/user/repo",
        );
        let dto = pane_content_to_spec_dto(&content);
        match dto {
            PaneSpecDto::Git { working_directory } => {
                assert_eq!(working_directory, "/home/user/repo");
            }
            other => panic!("Expected Git PaneSpecDto, got {other:?}"),
        }
    }

    // -- Git PaneRuntime → PaneRuntimeView ----------------------------------

    #[test]
    fn pane_runtime_to_view_maps_git_with_repo_path() {
        let runtime = PaneRuntime {
            pane_id: PaneId::from(String::from("pane-git-1")),
            runtime_session_id: Some(RuntimeSessionId::from(String::from("git-session-abc"))),
            kind: RuntimeKind::Git,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: None,
            terminal_cwd: None,
            git_repo_path: Some(WorkingDirectory::new("/home/user/project").expect("valid path")),
        };

        let view = pane_runtime_to_view(&runtime);

        assert_eq!(view.pane_id, "pane-git-1");
        assert_eq!(view.runtime_session_id.as_deref(), Some("git-session-abc"));
        assert!(matches!(view.kind, RuntimeKindDto::Git));
        assert!(matches!(view.status, RuntimeStatusDto::Running));
        assert_eq!(view.git_repo_path.as_deref(), Some("/home/user/project"));
        assert!(view.browser_location.is_none());
        assert!(view.terminal_cwd.is_none());
    }

    #[test]
    fn pane_runtime_to_view_maps_git_without_repo_path() {
        let runtime = PaneRuntime {
            pane_id: PaneId::from(String::from("pane-git-2")),
            runtime_session_id: None,
            kind: RuntimeKind::Git,
            status: RuntimeStatus::Starting,
            last_error: None,
            browser_location: None,
            terminal_cwd: None,
            git_repo_path: None,
        };

        let view = pane_runtime_to_view(&runtime);

        assert!(matches!(view.kind, RuntimeKindDto::Git));
        assert!(matches!(view.status, RuntimeStatusDto::Starting));
        assert!(view.git_repo_path.is_none());
        assert!(view.runtime_session_id.is_none());
    }

    // -- RuntimeKind::Git → RuntimeKindDto::Git -----------------------------

    #[test]
    fn runtime_kind_to_dto_maps_git() {
        assert!(matches!(
            runtime_kind_to_dto(RuntimeKind::Git),
            RuntimeKindDto::Git
        ));
    }

    // -- Bootstrap view with Git pane data ----------------------------------

    #[test]
    fn bootstrap_view_includes_git_runtime_projections() {
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
        let runtimes = vec![PaneRuntime {
            pane_id: PaneId::from(String::from("pane-git-boot")),
            runtime_session_id: Some(RuntimeSessionId::from(String::from("git-boot-1"))),
            kind: RuntimeKind::Git,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: None,
            terminal_cwd: None,
            git_repo_path: Some(WorkingDirectory::new("/home/user/repo").expect("valid path")),
        }];

        let view = bootstrap_view(&session, &preferences, &catalog, &runtimes);

        assert_eq!(view.runtime_projections.len(), 1);
        let git_proj = &view.runtime_projections[0];
        assert_eq!(git_proj.pane_id, "pane-git-boot");
        assert!(matches!(git_proj.kind, RuntimeKindDto::Git));
        assert_eq!(git_proj.git_repo_path.as_deref(), Some("/home/user/repo"));
    }

    // -- parse_line_range helper tests --------------------------------------

    #[test]
    fn parse_line_range_valid() {
        assert_eq!(parse_line_range("1-5").expect("valid"), (1, 5));
        assert_eq!(parse_line_range("0-100").expect("valid"), (0, 100));
    }

    #[test]
    fn parse_line_range_invalid_format() {
        assert!(parse_line_range("bad").is_err());
        assert!(parse_line_range("1:5").is_err());
    }

    #[test]
    fn parse_line_range_invalid_numbers() {
        assert!(parse_line_range("abc-5").is_err());
        assert!(parse_line_range("1-xyz").is_err());
    }
}
