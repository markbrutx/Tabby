use std::path::PathBuf;

use tabby_contracts::{
    BlameEntryDto, BranchInfoDto, CommitInfoDto, DiffContentDto, DiffHunkDto, DiffLineDto,
    DiffLineKindDto, FileStatusDto, FileStatusKindDto, GitCommandDto, GitRepoStateDto,
    GitResultDto, StashEntryDto,
};
use tabby_git::value_objects::{BranchName, RemoteName, StashId};
use tabby_git::{
    BlameEntry, BranchInfo, CommitInfo, DiffContent, DiffHunk, DiffLine, DiffLineKind, FileStatus,
    FileStatusKind, GitRepositoryState, StashEntry,
};

use crate::application::commands::{GitCommand, GitResult};
use crate::shell::error::ShellError;

// ---------------------------------------------------------------------------
// Git: DTO → Domain (inbound / commands)
// ---------------------------------------------------------------------------

/// Extracts the `pane_id` string from any `GitCommandDto` variant.
pub(crate) fn extract_git_pane_id(dto: &GitCommandDto) -> String {
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
pub(crate) fn git_command_from_dto(
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
pub(crate) fn git_result_to_dto(result: GitResult) -> GitResultDto {
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
pub(crate) fn file_status_to_dto(status: &FileStatus) -> FileStatusDto {
    FileStatusDto {
        path: status.path().to_string(),
        old_path: status.old_path().map(|s| s.to_string()),
        index_status: file_status_kind_to_dto(status.index_status()),
        worktree_status: file_status_kind_to_dto(status.worktree_status()),
    }
}

#[allow(dead_code)]
pub(crate) fn file_status_kind_to_dto(kind: FileStatusKind) -> FileStatusKindDto {
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
pub(crate) fn diff_content_to_dto(diff: &DiffContent) -> DiffContentDto {
    DiffContentDto {
        file_path: diff.file_path().to_string(),
        old_path: diff.old_path().map(|s| s.to_string()),
        hunks: diff.hunks().iter().map(diff_hunk_to_dto).collect(),
        is_binary: diff.is_binary(),
        file_mode_change: diff.file_mode_change().map(|s| s.to_string()),
    }
}

#[allow(dead_code)]
pub(crate) fn diff_hunk_to_dto(hunk: &DiffHunk) -> DiffHunkDto {
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
pub(crate) fn diff_line_to_dto(line: &DiffLine) -> DiffLineDto {
    DiffLineDto {
        kind: diff_line_kind_to_dto(line.kind()),
        old_line_no: line.old_line_no(),
        new_line_no: line.new_line_no(),
        content: line.content().to_string(),
    }
}

#[allow(dead_code)]
pub(crate) fn diff_line_kind_to_dto(kind: DiffLineKind) -> DiffLineKindDto {
    match kind {
        DiffLineKind::Context => DiffLineKindDto::Context,
        DiffLineKind::Addition => DiffLineKindDto::Addition,
        DiffLineKind::Deletion => DiffLineKindDto::Deletion,
        DiffLineKind::HunkHeader => DiffLineKindDto::HunkHeader,
    }
}

#[allow(dead_code)]
pub(crate) fn commit_info_to_dto(info: &CommitInfo) -> CommitInfoDto {
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
pub(crate) fn branch_info_to_dto(branch: &BranchInfo) -> BranchInfoDto {
    BranchInfoDto {
        name: branch.name().as_ref().to_string(),
        is_current: branch.is_current(),
        upstream: branch.upstream().map(|s| s.to_string()),
        ahead: branch.ahead(),
        behind: branch.behind(),
    }
}

#[allow(dead_code)]
pub(crate) fn blame_entry_to_dto(entry: &BlameEntry) -> BlameEntryDto {
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
pub(crate) fn stash_entry_to_dto(entry: &StashEntry) -> StashEntryDto {
    StashEntryDto {
        index: entry.index().index() as u32,
        message: entry.message().to_string(),
        date: entry.date().to_string(),
    }
}

#[allow(dead_code)]
pub(crate) fn git_repo_state_to_dto(state: &GitRepositoryState) -> GitRepoStateDto {
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

/// Parse a line-range string like `"10-20"` into a `(u32, u32)` tuple.
#[allow(dead_code)]
pub(crate) fn parse_line_range(s: &str) -> Result<(u32, u32), ShellError> {
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
pub(crate) fn remote_name_or_default(name: Option<&str>) -> Result<RemoteName, ShellError> {
    let raw = name.unwrap_or("origin");
    RemoteName::try_new(raw).map_err(|e| ShellError::Validation(e.to_string()))
}

/// Resolve an optional branch name, returning an error with `context` if `None`.
#[allow(dead_code)]
pub(crate) fn branch_name_required(
    name: Option<&str>,
    context: &str,
) -> Result<BranchName, ShellError> {
    let raw = name.ok_or_else(|| ShellError::Validation(context.to_string()))?;
    BranchName::try_new(raw).map_err(|e| ShellError::Validation(e.to_string()))
}
