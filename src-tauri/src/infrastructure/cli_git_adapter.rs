use std::path::Path;
use std::process::Command;

use tabby_git::value_objects::{BranchName, RemoteName, StashId};
use tabby_git::{
    BlameEntry, BranchInfo, CommitInfo, DiffContent, FileStatus, GitRepositoryState, StashEntry,
};

use crate::application::ports::GitOperationsPort;
use crate::shell::error::ShellError;

/// Infrastructure adapter that implements `GitOperationsPort` by shelling out
/// to the `git` CLI.
///
/// All operations delegate to `run_git`, which spawns `git` as a child process,
/// captures stdout/stderr, and maps non-zero exit codes to `ShellError`.
// Will be wired into AppShell in a follow-up story; currently only used by tests.
#[derive(Debug)]
#[allow(dead_code)]
pub struct CliGitAdapter;

#[allow(dead_code)]
impl CliGitAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Run a git command in the given repository directory.
    ///
    /// Spawns `git` with the provided arguments, sets the working directory to
    /// `repo_path`, and captures stdout + stderr. Returns stdout on success, or
    /// a `ShellError::Io` with stderr content on non-zero exit.
    fn run_git(&self, repo_path: &Path, args: &[&str]) -> Result<String, ShellError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .output()
            .map_err(|e| ShellError::Io(format!("failed to spawn git: {e}")))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(ShellError::Io(format!(
                "git {} failed (exit {}): {}",
                args.join(" "),
                output
                    .status
                    .code()
                    .map_or("unknown".to_string(), |c| c.to_string()),
                stderr.trim()
            )))
        }
    }
}

impl GitOperationsPort for CliGitAdapter {
    fn status(&self, _repo_path: &Path) -> Result<Vec<FileStatus>, ShellError> {
        todo!("GIT-014: status will be implemented in a follow-up story")
    }

    fn diff(&self, _repo_path: &Path, _staged: bool) -> Result<Vec<DiffContent>, ShellError> {
        todo!("GIT-014: diff will be implemented in a follow-up story")
    }

    fn stage(&self, _repo_path: &Path, _paths: &[&str]) -> Result<(), ShellError> {
        todo!("GIT-014: stage will be implemented in a follow-up story")
    }

    fn unstage(&self, _repo_path: &Path, _paths: &[&str]) -> Result<(), ShellError> {
        todo!("GIT-014: unstage will be implemented in a follow-up story")
    }

    fn stage_lines(
        &self,
        _repo_path: &Path,
        _file_path: &str,
        _line_ranges: &[(u32, u32)],
    ) -> Result<(), ShellError> {
        todo!("GIT-014: stage_lines will be implemented in a follow-up story")
    }

    fn commit(&self, _repo_path: &Path, _message: &str) -> Result<CommitInfo, ShellError> {
        todo!("GIT-014: commit will be implemented in a follow-up story")
    }

    fn push(
        &self,
        _repo_path: &Path,
        _remote: &RemoteName,
        _branch: &BranchName,
    ) -> Result<(), ShellError> {
        todo!("GIT-014: push will be implemented in a follow-up story")
    }

    fn pull(
        &self,
        _repo_path: &Path,
        _remote: &RemoteName,
        _branch: &BranchName,
    ) -> Result<(), ShellError> {
        todo!("GIT-014: pull will be implemented in a follow-up story")
    }

    fn fetch(&self, _repo_path: &Path, _remote: &RemoteName) -> Result<(), ShellError> {
        todo!("GIT-014: fetch will be implemented in a follow-up story")
    }

    fn branches(&self, _repo_path: &Path) -> Result<Vec<BranchInfo>, ShellError> {
        todo!("GIT-014: branches will be implemented in a follow-up story")
    }

    fn checkout_branch(&self, _repo_path: &Path, _branch: &BranchName) -> Result<(), ShellError> {
        todo!("GIT-014: checkout_branch will be implemented in a follow-up story")
    }

    fn create_branch(&self, _repo_path: &Path, _branch: &BranchName) -> Result<(), ShellError> {
        todo!("GIT-014: create_branch will be implemented in a follow-up story")
    }

    fn delete_branch(&self, _repo_path: &Path, _branch: &BranchName) -> Result<(), ShellError> {
        todo!("GIT-014: delete_branch will be implemented in a follow-up story")
    }

    fn merge_branch(&self, _repo_path: &Path, _branch: &BranchName) -> Result<(), ShellError> {
        todo!("GIT-014: merge_branch will be implemented in a follow-up story")
    }

    fn log(&self, _repo_path: &Path, _max_count: u32) -> Result<Vec<CommitInfo>, ShellError> {
        todo!("GIT-014: log will be implemented in a follow-up story")
    }

    fn blame(&self, _repo_path: &Path, _file_path: &str) -> Result<Vec<BlameEntry>, ShellError> {
        todo!("GIT-014: blame will be implemented in a follow-up story")
    }

    fn stash_push(&self, _repo_path: &Path, _message: Option<&str>) -> Result<(), ShellError> {
        todo!("GIT-014: stash_push will be implemented in a follow-up story")
    }

    fn stash_pop(&self, _repo_path: &Path) -> Result<(), ShellError> {
        todo!("GIT-014: stash_pop will be implemented in a follow-up story")
    }

    fn stash_list(&self, _repo_path: &Path) -> Result<Vec<StashEntry>, ShellError> {
        todo!("GIT-014: stash_list will be implemented in a follow-up story")
    }

    fn stash_drop(&self, _repo_path: &Path, _stash_id: StashId) -> Result<(), ShellError> {
        todo!("GIT-014: stash_drop will be implemented in a follow-up story")
    }

    fn discard_changes(&self, _repo_path: &Path, _paths: &[&str]) -> Result<(), ShellError> {
        todo!("GIT-014: discard_changes will be implemented in a follow-up story")
    }

    fn repo_state(&self, _repo_path: &Path) -> Result<GitRepositoryState, ShellError> {
        todo!("GIT-014: repo_state will be implemented in a follow-up story")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn run_git_version_succeeds() {
        let adapter = CliGitAdapter::new();
        // Use the repo root itself as a valid directory; git --version ignores cwd
        // but Command requires an existing directory.
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let output = adapter
            .run_git(&cwd, &["--version"])
            .expect("git --version should succeed");
        assert!(
            output.starts_with("git version"),
            "unexpected output: {output}"
        );
    }

    #[test]
    fn run_git_returns_error_on_invalid_command() {
        let adapter = CliGitAdapter::new();
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let result = adapter.run_git(&cwd, &["not-a-real-subcommand"]);
        assert!(result.is_err(), "should fail for invalid git subcommand");
        let err = result.unwrap_err();
        match err {
            ShellError::Io(msg) => {
                assert!(
                    msg.contains("failed"),
                    "error should mention failure: {msg}"
                );
            }
            other => panic!("expected ShellError::Io, got: {other:?}"),
        }
    }

    #[test]
    fn run_git_returns_error_for_nonexistent_directory() {
        let adapter = CliGitAdapter::new();
        let bad_path = PathBuf::from("/tmp/tabby-nonexistent-dir-for-test-12345");
        let result = adapter.run_git(&bad_path, &["status"]);
        assert!(result.is_err(), "should fail when repo_path does not exist");
    }
}
