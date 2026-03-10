use crate::application::commands::{GitCommand, GitResult};
use crate::application::ports::GitOperationsPort;
use crate::shell::error::ShellError;

#[derive(Debug)]
pub struct GitApplicationService {
    git_port: Box<dyn GitOperationsPort>,
}

impl GitApplicationService {
    pub fn new(git_port: Box<dyn GitOperationsPort>) -> Self {
        Self { git_port }
    }

    pub fn dispatch_command(&self, command: GitCommand) -> Result<GitResult, ShellError> {
        match command {
            GitCommand::Status { repo_path } => {
                let files = self.git_port.status(&repo_path)?;
                Ok(GitResult::Status(files))
            }
            GitCommand::Diff { repo_path, staged } => {
                let diffs = self.git_port.diff(&repo_path, staged)?;
                Ok(GitResult::Diff(diffs))
            }
            GitCommand::Stage { repo_path, paths } => {
                let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
                self.git_port.stage(&repo_path, &path_refs)?;
                Ok(GitResult::Stage)
            }
            GitCommand::Unstage { repo_path, paths } => {
                let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
                self.git_port.unstage(&repo_path, &path_refs)?;
                Ok(GitResult::Unstage)
            }
            GitCommand::StageLines {
                repo_path,
                file_path,
                line_ranges,
            } => {
                self.git_port
                    .stage_lines(&repo_path, &file_path, &line_ranges)?;
                Ok(GitResult::StageLines)
            }
            GitCommand::Commit {
                repo_path,
                message,
                amend,
            } => {
                let info = self.git_port.commit(&repo_path, &message, amend)?;
                Ok(GitResult::Commit(info))
            }
            GitCommand::Push {
                repo_path,
                remote,
                branch,
            } => {
                self.git_port.push(&repo_path, &remote, &branch)?;
                Ok(GitResult::Push)
            }
            GitCommand::Pull {
                repo_path,
                remote,
                branch,
            } => {
                self.git_port.pull(&repo_path, &remote, &branch)?;
                Ok(GitResult::Pull)
            }
            GitCommand::Fetch { repo_path, remote } => {
                self.git_port.fetch(&repo_path, &remote)?;
                Ok(GitResult::Fetch)
            }
            GitCommand::Branches { repo_path } => {
                let branches = self.git_port.branches(&repo_path)?;
                Ok(GitResult::Branches(branches))
            }
            GitCommand::CheckoutBranch { repo_path, branch } => {
                self.git_port.checkout_branch(&repo_path, &branch)?;
                Ok(GitResult::CheckoutBranch)
            }
            GitCommand::CreateBranch {
                repo_path,
                branch,
                start_point,
            } => {
                self.git_port
                    .create_branch(&repo_path, &branch, start_point.as_ref())?;
                Ok(GitResult::CreateBranch)
            }
            GitCommand::DeleteBranch {
                repo_path,
                branch,
                force,
            } => {
                self.git_port.delete_branch(&repo_path, &branch, force)?;
                Ok(GitResult::DeleteBranch)
            }
            GitCommand::MergeBranch { repo_path, branch } => {
                self.git_port.merge_branch(&repo_path, &branch)?;
                Ok(GitResult::MergeBranch)
            }
            GitCommand::Log {
                repo_path,
                max_count,
                skip,
            } => {
                let commits = self.git_port.log(&repo_path, max_count, skip)?;
                Ok(GitResult::Log(commits))
            }
            GitCommand::ShowCommit { repo_path, hash } => {
                let diffs = self.git_port.show_commit(&repo_path, &hash)?;
                Ok(GitResult::ShowCommit(diffs))
            }
            GitCommand::Blame {
                repo_path,
                file_path,
            } => {
                let entries = self.git_port.blame(&repo_path, &file_path)?;
                Ok(GitResult::Blame(entries))
            }
            GitCommand::StashPush { repo_path, message } => {
                self.git_port.stash_push(&repo_path, message.as_deref())?;
                Ok(GitResult::StashPush)
            }
            GitCommand::StashPop { repo_path } => {
                self.git_port.stash_pop(&repo_path)?;
                Ok(GitResult::StashPop)
            }
            GitCommand::StashList { repo_path } => {
                let entries = self.git_port.stash_list(&repo_path)?;
                Ok(GitResult::StashList(entries))
            }
            GitCommand::StashDrop {
                repo_path,
                stash_id,
            } => {
                self.git_port.stash_drop(&repo_path, stash_id)?;
                Ok(GitResult::StashDrop)
            }
            GitCommand::DiscardChanges { repo_path, paths } => {
                let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
                self.git_port.discard_changes(&repo_path, &path_refs)?;
                Ok(GitResult::DiscardChanges)
            }
            GitCommand::RepoState { repo_path } => {
                let state = self.git_port.repo_state(&repo_path)?;
                Ok(GitResult::RepoState(state))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use tabby_git::value_objects::{BranchName, CommitHash, RemoteName, StashId};
    use tabby_git::{
        BlameEntry, BranchInfo, CommitInfo, DiffContent, FileStatus, FileStatusKind,
        GitRepositoryState, StashEntry,
    };
    use tabby_kernel::WorkingDirectory;

    use super::*;

    /// Records which port method was called and with what repo path.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum PortCall {
        Status(PathBuf),
        Diff(PathBuf, bool),
        Stage(PathBuf, Vec<String>),
        Unstage(PathBuf, Vec<String>),
        StageLines(PathBuf, String, Vec<(u32, u32)>),
        Commit(PathBuf, String, bool),
        Push(PathBuf, String, String),
        Pull(PathBuf, String, String),
        Fetch(PathBuf, String),
        Branches(PathBuf),
        CheckoutBranch(PathBuf, String),
        CreateBranch(PathBuf, String, Option<String>),
        DeleteBranch(PathBuf, String, bool),
        MergeBranch(PathBuf, String),
        Log(PathBuf, u32, u32),
        ShowCommit(PathBuf, String),
        Blame(PathBuf, String),
        StashPush(PathBuf, Option<String>),
        StashPop(PathBuf),
        StashList(PathBuf),
        StashDrop(PathBuf, usize),
        DiscardChanges(PathBuf, Vec<String>),
        RepoState(PathBuf),
    }

    #[derive(Debug)]
    struct MockGitPort {
        calls: Mutex<Vec<PortCall>>,
    }

    impl MockGitPort {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }

        fn recorded_calls(&self) -> Vec<PortCall> {
            self.calls.lock().map_or_else(|_| vec![], |g| g.clone())
        }
    }

    impl GitOperationsPort for MockGitPort {
        fn status(&self, repo_path: &Path) -> Result<Vec<FileStatus>, ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Status(repo_path.to_path_buf()));
            Ok(vec![FileStatus::new(
                "README.md",
                None,
                FileStatusKind::Modified,
                FileStatusKind::Modified,
            )])
        }

        fn diff(&self, repo_path: &Path, staged: bool) -> Result<Vec<DiffContent>, ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Diff(repo_path.to_path_buf(), staged));
            Ok(vec![])
        }

        fn stage(&self, repo_path: &Path, paths: &[&str]) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Stage(
                    repo_path.to_path_buf(),
                    paths.iter().map(|s| s.to_string()).collect(),
                ));
            Ok(())
        }

        fn unstage(&self, repo_path: &Path, paths: &[&str]) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Unstage(
                    repo_path.to_path_buf(),
                    paths.iter().map(|s| s.to_string()).collect(),
                ));
            Ok(())
        }

        fn stage_lines(
            &self,
            repo_path: &Path,
            file_path: &str,
            line_ranges: &[(u32, u32)],
        ) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::StageLines(
                    repo_path.to_path_buf(),
                    file_path.to_string(),
                    line_ranges.to_vec(),
                ));
            Ok(())
        }

        fn commit(
            &self,
            repo_path: &Path,
            message: &str,
            amend: bool,
        ) -> Result<CommitInfo, ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Commit(
                    repo_path.to_path_buf(),
                    message.to_string(),
                    amend,
                ));
            Ok(CommitInfo::new(
                CommitHash::try_new("abc1234def5678")
                    .map_err(|e| ShellError::Validation(e.to_string()))?,
                "abc1234".to_string(),
                "Test".to_string(),
                "test@example.com".to_string(),
                "2026-03-10".to_string(),
                message.to_string(),
                vec![],
            ))
        }

        fn push(
            &self,
            repo_path: &Path,
            remote: &RemoteName,
            branch: &BranchName,
        ) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Push(
                    repo_path.to_path_buf(),
                    remote.to_string(),
                    branch.to_string(),
                ));
            Ok(())
        }

        fn pull(
            &self,
            repo_path: &Path,
            remote: &RemoteName,
            branch: &BranchName,
        ) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Pull(
                    repo_path.to_path_buf(),
                    remote.to_string(),
                    branch.to_string(),
                ));
            Ok(())
        }

        fn fetch(&self, repo_path: &Path, remote: &RemoteName) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Fetch(repo_path.to_path_buf(), remote.to_string()));
            Ok(())
        }

        fn branches(&self, repo_path: &Path) -> Result<Vec<BranchInfo>, ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Branches(repo_path.to_path_buf()));
            Ok(vec![BranchInfo::new(
                BranchName::try_new("main").map_err(|e| ShellError::Validation(e.to_string()))?,
                true,
                None,
                0,
                0,
            )])
        }

        fn checkout_branch(&self, repo_path: &Path, branch: &BranchName) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::CheckoutBranch(
                    repo_path.to_path_buf(),
                    branch.to_string(),
                ));
            Ok(())
        }

        fn create_branch(
            &self,
            repo_path: &Path,
            branch: &BranchName,
            start_point: Option<&BranchName>,
        ) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::CreateBranch(
                    repo_path.to_path_buf(),
                    branch.to_string(),
                    start_point.map(|sp| sp.to_string()),
                ));
            Ok(())
        }

        fn delete_branch(
            &self,
            repo_path: &Path,
            branch: &BranchName,
            force: bool,
        ) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::DeleteBranch(
                    repo_path.to_path_buf(),
                    branch.to_string(),
                    force,
                ));
            Ok(())
        }

        fn merge_branch(&self, repo_path: &Path, branch: &BranchName) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::MergeBranch(
                    repo_path.to_path_buf(),
                    branch.to_string(),
                ));
            Ok(())
        }

        fn log(
            &self,
            repo_path: &Path,
            max_count: u32,
            skip: u32,
        ) -> Result<Vec<CommitInfo>, ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Log(repo_path.to_path_buf(), max_count, skip));
            Ok(vec![])
        }

        fn show_commit(
            &self,
            repo_path: &Path,
            hash: &str,
        ) -> Result<Vec<DiffContent>, ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::ShowCommit(
                    repo_path.to_path_buf(),
                    hash.to_string(),
                ));
            Ok(vec![])
        }

        fn blame(&self, repo_path: &Path, file_path: &str) -> Result<Vec<BlameEntry>, ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::Blame(
                    repo_path.to_path_buf(),
                    file_path.to_string(),
                ));
            Ok(vec![])
        }

        fn stash_push(&self, repo_path: &Path, message: Option<&str>) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::StashPush(
                    repo_path.to_path_buf(),
                    message.map(|s| s.to_string()),
                ));
            Ok(())
        }

        fn stash_pop(&self, repo_path: &Path) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::StashPop(repo_path.to_path_buf()));
            Ok(())
        }

        fn stash_list(&self, repo_path: &Path) -> Result<Vec<StashEntry>, ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::StashList(repo_path.to_path_buf()));
            Ok(vec![])
        }

        fn stash_drop(&self, repo_path: &Path, stash_id: StashId) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::StashDrop(
                    repo_path.to_path_buf(),
                    stash_id.index(),
                ));
            Ok(())
        }

        fn discard_changes(&self, repo_path: &Path, paths: &[&str]) -> Result<(), ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::DiscardChanges(
                    repo_path.to_path_buf(),
                    paths.iter().map(|s| s.to_string()).collect(),
                ));
            Ok(())
        }

        fn repo_state(&self, repo_path: &Path) -> Result<GitRepositoryState, ShellError> {
            self.calls
                .lock()
                .map_err(|e| ShellError::State(e.to_string()))?
                .push(PortCall::RepoState(repo_path.to_path_buf()));
            Ok(GitRepositoryState::new(
                WorkingDirectory::new(repo_path.to_string_lossy())
                    .map_err(|e| ShellError::Validation(e.to_string()))?,
                Some(
                    BranchName::try_new("main")
                        .map_err(|e| ShellError::Validation(e.to_string()))?,
                ),
                false,
                true,
            ))
        }
    }

    fn repo() -> PathBuf {
        PathBuf::from("/tmp/test-repo")
    }

    fn make_service() -> (GitApplicationService, std::sync::Arc<MockGitPort>) {
        let mock = std::sync::Arc::new(MockGitPort::new());
        // Create a second reference for assertion; service owns a Box wrapper.
        let mock_for_assertions = mock.clone();

        // We need a way to share the Arc-backed mock. Wrap it in a thin adapter
        // that delegates to the Arc.
        #[derive(Debug)]
        struct ArcAdapter(std::sync::Arc<MockGitPort>);

        impl GitOperationsPort for ArcAdapter {
            fn status(&self, p: &Path) -> Result<Vec<FileStatus>, ShellError> {
                self.0.status(p)
            }
            fn diff(&self, p: &Path, s: bool) -> Result<Vec<DiffContent>, ShellError> {
                self.0.diff(p, s)
            }
            fn stage(&self, p: &Path, paths: &[&str]) -> Result<(), ShellError> {
                self.0.stage(p, paths)
            }
            fn unstage(&self, p: &Path, paths: &[&str]) -> Result<(), ShellError> {
                self.0.unstage(p, paths)
            }
            fn stage_lines(&self, p: &Path, fp: &str, lr: &[(u32, u32)]) -> Result<(), ShellError> {
                self.0.stage_lines(p, fp, lr)
            }
            fn commit(&self, p: &Path, m: &str, amend: bool) -> Result<CommitInfo, ShellError> {
                self.0.commit(p, m, amend)
            }
            fn push(&self, p: &Path, r: &RemoteName, b: &BranchName) -> Result<(), ShellError> {
                self.0.push(p, r, b)
            }
            fn pull(&self, p: &Path, r: &RemoteName, b: &BranchName) -> Result<(), ShellError> {
                self.0.pull(p, r, b)
            }
            fn fetch(&self, p: &Path, r: &RemoteName) -> Result<(), ShellError> {
                self.0.fetch(p, r)
            }
            fn branches(&self, p: &Path) -> Result<Vec<BranchInfo>, ShellError> {
                self.0.branches(p)
            }
            fn checkout_branch(&self, p: &Path, b: &BranchName) -> Result<(), ShellError> {
                self.0.checkout_branch(p, b)
            }
            fn create_branch(
                &self,
                p: &Path,
                b: &BranchName,
                sp: Option<&BranchName>,
            ) -> Result<(), ShellError> {
                self.0.create_branch(p, b, sp)
            }
            fn delete_branch(
                &self,
                p: &Path,
                b: &BranchName,
                force: bool,
            ) -> Result<(), ShellError> {
                self.0.delete_branch(p, b, force)
            }
            fn merge_branch(&self, p: &Path, b: &BranchName) -> Result<(), ShellError> {
                self.0.merge_branch(p, b)
            }
            fn log(&self, p: &Path, mc: u32, skip: u32) -> Result<Vec<CommitInfo>, ShellError> {
                self.0.log(p, mc, skip)
            }
            fn show_commit(&self, p: &Path, hash: &str) -> Result<Vec<DiffContent>, ShellError> {
                self.0.show_commit(p, hash)
            }
            fn blame(&self, p: &Path, fp: &str) -> Result<Vec<BlameEntry>, ShellError> {
                self.0.blame(p, fp)
            }
            fn stash_push(&self, p: &Path, m: Option<&str>) -> Result<(), ShellError> {
                self.0.stash_push(p, m)
            }
            fn stash_pop(&self, p: &Path) -> Result<(), ShellError> {
                self.0.stash_pop(p)
            }
            fn stash_list(&self, p: &Path) -> Result<Vec<StashEntry>, ShellError> {
                self.0.stash_list(p)
            }
            fn stash_drop(&self, p: &Path, sid: StashId) -> Result<(), ShellError> {
                self.0.stash_drop(p, sid)
            }
            fn discard_changes(&self, p: &Path, paths: &[&str]) -> Result<(), ShellError> {
                self.0.discard_changes(p, paths)
            }
            fn repo_state(&self, p: &Path) -> Result<GitRepositoryState, ShellError> {
                self.0.repo_state(p)
            }
        }

        let service = GitApplicationService::new(Box::new(ArcAdapter(mock)));
        (service, mock_for_assertions)
    }

    #[test]
    fn dispatch_status_delegates_to_port() {
        let (service, mock) = make_service();
        let result = service
            .dispatch_command(GitCommand::Status { repo_path: repo() })
            .expect("should succeed");

        assert!(matches!(result, GitResult::Status(files) if files.len() == 1));
        assert_eq!(mock.recorded_calls(), vec![PortCall::Status(repo())]);
    }

    #[test]
    fn dispatch_diff_delegates_to_port() {
        let (service, mock) = make_service();
        let result = service
            .dispatch_command(GitCommand::Diff {
                repo_path: repo(),
                staged: true,
            })
            .expect("should succeed");

        assert!(matches!(result, GitResult::Diff(_)));
        assert_eq!(mock.recorded_calls(), vec![PortCall::Diff(repo(), true)]);
    }

    #[test]
    fn dispatch_commit_delegates_to_port() {
        let (service, mock) = make_service();
        let result = service
            .dispatch_command(GitCommand::Commit {
                repo_path: repo(),
                message: "feat: test".to_string(),
                amend: false,
            })
            .expect("should succeed");

        assert!(matches!(result, GitResult::Commit(info) if info.short_hash() == "abc1234"));
        assert_eq!(
            mock.recorded_calls(),
            vec![PortCall::Commit(repo(), "feat: test".to_string(), false)]
        );
    }

    #[test]
    fn dispatch_branches_delegates_to_port() {
        let (service, mock) = make_service();
        let result = service
            .dispatch_command(GitCommand::Branches { repo_path: repo() })
            .expect("should succeed");

        assert!(matches!(result, GitResult::Branches(branches) if branches.len() == 1));
        assert_eq!(mock.recorded_calls(), vec![PortCall::Branches(repo())]);
    }

    #[test]
    fn dispatch_checkout_branch_delegates_to_port() {
        let (service, mock) = make_service();
        let branch = BranchName::try_new("feature/test").expect("valid branch");
        let result = service
            .dispatch_command(GitCommand::CheckoutBranch {
                repo_path: repo(),
                branch,
            })
            .expect("should succeed");

        assert!(matches!(result, GitResult::CheckoutBranch));
        assert_eq!(
            mock.recorded_calls(),
            vec![PortCall::CheckoutBranch(repo(), "feature/test".to_string())]
        );
    }

    #[test]
    fn dispatch_stage_delegates_to_port() {
        let (service, mock) = make_service();
        let result = service
            .dispatch_command(GitCommand::Stage {
                repo_path: repo(),
                paths: vec!["file.rs".to_string()],
            })
            .expect("should succeed");

        assert!(matches!(result, GitResult::Stage));
        assert_eq!(
            mock.recorded_calls(),
            vec![PortCall::Stage(repo(), vec!["file.rs".to_string()])]
        );
    }

    #[test]
    fn dispatch_repo_state_delegates_to_port() {
        let (service, mock) = make_service();
        let result = service
            .dispatch_command(GitCommand::RepoState { repo_path: repo() })
            .expect("should succeed");

        assert!(
            matches!(result, GitResult::RepoState(state) if state.head_branch().map(|b| b.as_ref()) == Some("main"))
        );
        assert_eq!(mock.recorded_calls(), vec![PortCall::RepoState(repo())]);
    }

    #[test]
    fn dispatch_stash_push_delegates_to_port() {
        let (service, mock) = make_service();
        let result = service
            .dispatch_command(GitCommand::StashPush {
                repo_path: repo(),
                message: Some("WIP".to_string()),
            })
            .expect("should succeed");

        assert!(matches!(result, GitResult::StashPush));
        assert_eq!(
            mock.recorded_calls(),
            vec![PortCall::StashPush(repo(), Some("WIP".to_string()))]
        );
    }

    #[test]
    fn dispatch_log_delegates_to_port() {
        let (service, mock) = make_service();
        let result = service
            .dispatch_command(GitCommand::Log {
                repo_path: repo(),
                max_count: 50,
                skip: 0,
            })
            .expect("should succeed");

        assert!(matches!(result, GitResult::Log(_)));
        assert_eq!(mock.recorded_calls(), vec![PortCall::Log(repo(), 50, 0)]);
    }

    #[test]
    fn dispatch_discard_changes_delegates_to_port() {
        let (service, mock) = make_service();
        let result = service
            .dispatch_command(GitCommand::DiscardChanges {
                repo_path: repo(),
                paths: vec!["a.rs".to_string(), "b.rs".to_string()],
            })
            .expect("should succeed");

        assert!(matches!(result, GitResult::DiscardChanges));
        assert_eq!(
            mock.recorded_calls(),
            vec![PortCall::DiscardChanges(
                repo(),
                vec!["a.rs".to_string(), "b.rs".to_string()]
            )]
        );
    }
}
