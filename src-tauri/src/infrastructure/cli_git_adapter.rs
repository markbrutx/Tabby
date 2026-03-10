use std::path::Path;
use std::process::Command;

use tabby_git::value_objects::{BranchName, RemoteName, StashId};
use tabby_git::{
    BlameEntry, BranchInfo, CommitInfo, DiffContent, FileStatus, FileStatusKind,
    GitRepositoryState, StashEntry,
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

/// Map a single porcelain v2 XY status character to a `FileStatusKind`.
///
/// Git porcelain v2 uses these codes for the index (X) and worktree (Y) columns:
///   . = unmodified, M = modified, T = type-changed (treated as Modified),
///   A = added, D = deleted, R = renamed, C = copied, U = unmerged
fn status_char_to_kind(ch: char) -> FileStatusKind {
    match ch {
        'M' | 'T' => FileStatusKind::Modified,
        'A' => FileStatusKind::Added,
        'D' => FileStatusKind::Deleted,
        'R' => FileStatusKind::Renamed,
        'C' => FileStatusKind::Copied,
        'U' => FileStatusKind::Conflicted,
        // '.' means unmodified; treat as Modified for the "no change" slot
        // since it only appears when the other column has a real change.
        _ => FileStatusKind::Modified,
    }
}

/// Parse the full output of `git status --porcelain=v2` into domain `FileStatus` entries.
fn parse_porcelain_v2(output: &str) -> Result<Vec<FileStatus>, ShellError> {
    let mut entries = Vec::new();

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        match line.chars().next() {
            // Ordinary changed entry: "1 XY <sub> <mH> <mI> <mW> <hH> <hI> <path>"
            Some('1') => {
                let fields: Vec<&str> = line.splitn(9, ' ').collect();
                if fields.len() < 9 {
                    return Err(ShellError::Io(format!(
                        "malformed porcelain v2 ordinary entry: {line}"
                    )));
                }
                let xy = fields[1];
                let mut xy_chars = xy.chars();
                let x = xy_chars.next().unwrap_or('.');
                let y = xy_chars.next().unwrap_or('.');
                let path = fields[8];

                let index_status = status_char_to_kind(x);
                let worktree_status = status_char_to_kind(y);

                entries.push(FileStatus::new(path, None, index_status, worktree_status));
            }
            // Renamed/copied entry: "2 XY <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path>\t<origPath>"
            Some('2') => {
                let fields: Vec<&str> = line.splitn(10, ' ').collect();
                if fields.len() < 10 {
                    return Err(ShellError::Io(format!(
                        "malformed porcelain v2 rename/copy entry: {line}"
                    )));
                }
                let xy = fields[1];
                let mut xy_chars = xy.chars();
                let x = xy_chars.next().unwrap_or('.');
                let y = xy_chars.next().unwrap_or('.');

                // The last field is "path\torigPath"
                let path_field = fields[9];
                let (path, old_path) = match path_field.split_once('\t') {
                    Some((p, op)) => (p, Some(op.to_string())),
                    None => (path_field, None),
                };

                let index_status = status_char_to_kind(x);
                let worktree_status = status_char_to_kind(y);

                entries.push(FileStatus::new(
                    path,
                    old_path,
                    index_status,
                    worktree_status,
                ));
            }
            // Unmerged entry: "u XY <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>"
            Some('u') => {
                let fields: Vec<&str> = line.splitn(11, ' ').collect();
                if fields.len() < 11 {
                    return Err(ShellError::Io(format!(
                        "malformed porcelain v2 unmerged entry: {line}"
                    )));
                }
                let path = fields[10];
                entries.push(FileStatus::new(
                    path,
                    None,
                    FileStatusKind::Conflicted,
                    FileStatusKind::Conflicted,
                ));
            }
            // Untracked: "? <path>"
            Some('?') => {
                let path = &line[2..];
                entries.push(FileStatus::new(
                    path,
                    None,
                    FileStatusKind::Untracked,
                    FileStatusKind::Untracked,
                ));
            }
            // Ignored: "! <path>"
            Some('!') => {
                let path = &line[2..];
                entries.push(FileStatus::new(
                    path,
                    None,
                    FileStatusKind::Ignored,
                    FileStatusKind::Ignored,
                ));
            }
            // Header lines (# branch.oid, # branch.head, etc.) — skip
            Some('#') => continue,
            _ => continue,
        }
    }

    Ok(entries)
}

impl GitOperationsPort for CliGitAdapter {
    fn status(&self, repo_path: &Path) -> Result<Vec<FileStatus>, ShellError> {
        let output = self.run_git(repo_path, &["status", "--porcelain=v2"])?;
        parse_porcelain_v2(&output)
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

    // -----------------------------------------------------------------------
    // run_git tests (existing)
    // -----------------------------------------------------------------------

    #[test]
    fn run_git_version_succeeds() {
        let adapter = CliGitAdapter::new();
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

    // -----------------------------------------------------------------------
    // parse_porcelain_v2 tests (GIT-015)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_clean_repo_returns_empty_vec() {
        let output = "";
        let result = parse_porcelain_v2(output).expect("should parse empty output");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_clean_repo_with_headers_only() {
        let output = "# branch.oid abc123\n# branch.head main\n";
        let result = parse_porcelain_v2(output).expect("should parse header-only output");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_modified_file() {
        // Ordinary entry: index unmodified, worktree modified
        let output = "1 .M N... 100644 100644 100644 abc123 def456 src/main.rs\n";
        let result = parse_porcelain_v2(output).expect("should parse modified entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "src/main.rs");
        assert_eq!(result[0].old_path(), None);
        assert_eq!(result[0].worktree_status(), FileStatusKind::Modified);
    }

    #[test]
    fn parse_added_file_in_index() {
        // Ordinary entry: added in index, unmodified in worktree
        let output = "1 A. N... 000000 100644 100644 0000000 abc1234 new_file.rs\n";
        let result = parse_porcelain_v2(output).expect("should parse added entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "new_file.rs");
        assert_eq!(result[0].index_status(), FileStatusKind::Added);
    }

    #[test]
    fn parse_deleted_file() {
        // Deleted in worktree
        let output = "1 .D N... 100644 100644 000000 abc1234 def5678 removed.rs\n";
        let result = parse_porcelain_v2(output).expect("should parse deleted entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "removed.rs");
        assert_eq!(result[0].worktree_status(), FileStatusKind::Deleted);
    }

    #[test]
    fn parse_renamed_file() {
        // Rename entry with tab-separated path\torigPath
        let output =
            "2 R. N... 100644 100644 100644 abc1234 def5678 R100 new_name.rs\told_name.rs\n";
        let result = parse_porcelain_v2(output).expect("should parse renamed entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "new_name.rs");
        assert_eq!(result[0].old_path(), Some("old_name.rs"));
        assert_eq!(result[0].index_status(), FileStatusKind::Renamed);
    }

    #[test]
    fn parse_copied_file() {
        let output = "2 C. N... 100644 100644 100644 abc1234 def5678 C100 copy.rs\toriginal.rs\n";
        let result = parse_porcelain_v2(output).expect("should parse copied entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "copy.rs");
        assert_eq!(result[0].old_path(), Some("original.rs"));
        assert_eq!(result[0].index_status(), FileStatusKind::Copied);
    }

    #[test]
    fn parse_untracked_file() {
        let output = "? untracked_file.txt\n";
        let result = parse_porcelain_v2(output).expect("should parse untracked entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "untracked_file.txt");
        assert_eq!(result[0].index_status(), FileStatusKind::Untracked);
        assert_eq!(result[0].worktree_status(), FileStatusKind::Untracked);
    }

    #[test]
    fn parse_ignored_file() {
        let output = "! build/output.o\n";
        let result = parse_porcelain_v2(output).expect("should parse ignored entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "build/output.o");
        assert_eq!(result[0].index_status(), FileStatusKind::Ignored);
        assert_eq!(result[0].worktree_status(), FileStatusKind::Ignored);
    }

    #[test]
    fn parse_conflicted_file() {
        // Unmerged entry
        let output =
            "u UU N... 100644 100644 100644 100644 abc1234 def5678 ghi9012 conflicted.rs\n";
        let result = parse_porcelain_v2(output).expect("should parse conflicted entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "conflicted.rs");
        assert_eq!(result[0].index_status(), FileStatusKind::Conflicted);
        assert_eq!(result[0].worktree_status(), FileStatusKind::Conflicted);
    }

    #[test]
    fn parse_mixed_status_output() {
        let output = "\
# branch.oid abc123def456
# branch.head main
1 .M N... 100644 100644 100644 abc1234 def5678 src/lib.rs
1 A. N... 000000 100644 100644 0000000 abc1234 src/new.rs
2 R. N... 100644 100644 100644 abc1234 def5678 R100 renamed.rs\told.rs
? untracked.txt
u UU N... 100644 100644 100644 100644 abc1234 def5678 ghi9012 merge_conflict.rs
! ignored.log
";
        let result = parse_porcelain_v2(output).expect("should parse mixed output");
        assert_eq!(result.len(), 6);

        assert_eq!(result[0].path(), "src/lib.rs");
        assert_eq!(result[0].worktree_status(), FileStatusKind::Modified);

        assert_eq!(result[1].path(), "src/new.rs");
        assert_eq!(result[1].index_status(), FileStatusKind::Added);

        assert_eq!(result[2].path(), "renamed.rs");
        assert_eq!(result[2].old_path(), Some("old.rs"));
        assert_eq!(result[2].index_status(), FileStatusKind::Renamed);

        assert_eq!(result[3].path(), "untracked.txt");
        assert_eq!(result[3].index_status(), FileStatusKind::Untracked);

        assert_eq!(result[4].path(), "merge_conflict.rs");
        assert_eq!(result[4].index_status(), FileStatusKind::Conflicted);

        assert_eq!(result[5].path(), "ignored.log");
        assert_eq!(result[5].index_status(), FileStatusKind::Ignored);
    }

    #[test]
    fn parse_empty_repo_no_commits_yet() {
        // In a fresh repo with no commits, git status --porcelain=v2 outputs header lines
        // with "(initial)" and file entries with A. status
        let output = "\
# branch.oid (initial)
# branch.head main
1 A. N... 000000 100644 100644 0000000 abc1234 README.md
";
        let result = parse_porcelain_v2(output).expect("should parse initial commit output");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "README.md");
        assert_eq!(result[0].index_status(), FileStatusKind::Added);
    }

    #[test]
    fn parse_type_changed_file() {
        // T = type-changed (e.g., regular file → symlink), mapped to Modified
        let output = "1 .T N... 100644 120000 120000 abc1234 def5678 link.txt\n";
        let result = parse_porcelain_v2(output).expect("should parse type-changed entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].worktree_status(), FileStatusKind::Modified);
    }

    #[test]
    fn parse_index_deleted_worktree_unmodified() {
        let output = "1 D. N... 100644 000000 000000 abc1234 0000000 deleted_staged.rs\n";
        let result = parse_porcelain_v2(output).expect("should parse staged deletion");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "deleted_staged.rs");
        assert_eq!(result[0].index_status(), FileStatusKind::Deleted);
    }

    #[test]
    fn status_char_mapping_covers_all_codes() {
        assert_eq!(status_char_to_kind('M'), FileStatusKind::Modified);
        assert_eq!(status_char_to_kind('T'), FileStatusKind::Modified);
        assert_eq!(status_char_to_kind('A'), FileStatusKind::Added);
        assert_eq!(status_char_to_kind('D'), FileStatusKind::Deleted);
        assert_eq!(status_char_to_kind('R'), FileStatusKind::Renamed);
        assert_eq!(status_char_to_kind('C'), FileStatusKind::Copied);
        assert_eq!(status_char_to_kind('U'), FileStatusKind::Conflicted);
        // '.' and unknown chars fall through to Modified
        assert_eq!(status_char_to_kind('.'), FileStatusKind::Modified);
    }
}
