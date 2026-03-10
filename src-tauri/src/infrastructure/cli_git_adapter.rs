use std::path::Path;
use std::process::Command;

use tabby_git::value_objects::{BranchName, RemoteName, StashId};
use tabby_git::{
    BlameEntry, BranchInfo, CommitInfo, DiffContent, FileStatus, FileStatusKind,
    GitRepositoryState, StashEntry,
};

use crate::application::ports::GitOperationsPort;
use crate::shell::error::ShellError;

use super::blame_parser::parse_blame_porcelain;
use super::diff_parser::{filter_diff_to_line_ranges, parse_unified_diff};
use super::porcelain_parser::{
    parse_branch_list, parse_commit_show_output, parse_log_output, parse_porcelain_v2,
};
#[cfg(test)]
use super::porcelain_parser::{parse_tracking_info, status_char_to_kind};
use super::stash_parser::parse_stash_list_output;

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

// Standalone parse functions have been moved to dedicated parser modules:
// - porcelain_parser (status, branch, log, commit)
// - diff_parser (unified diff, hunk, line range filter)
// - blame_parser (blame porcelain)
// - stash_parser (stash list)

impl GitOperationsPort for CliGitAdapter {
    fn status(&self, repo_path: &Path) -> Result<Vec<FileStatus>, ShellError> {
        let output = self.run_git(repo_path, &["status", "--porcelain=v2"])?;
        parse_porcelain_v2(&output)
    }

    fn diff(&self, repo_path: &Path, staged: bool) -> Result<Vec<DiffContent>, ShellError> {
        let mut args = vec!["diff", "--find-renames"];
        if staged {
            args.push("--staged");
        }
        let output = self.run_git(repo_path, &args)?;
        Ok(parse_unified_diff(&output))
    }

    fn stage(&self, repo_path: &Path, paths: &[&str]) -> Result<(), ShellError> {
        if paths.is_empty() {
            return Err(ShellError::Validation(
                "stage requires at least one path".to_string(),
            ));
        }
        let mut args = vec!["add", "--"];
        args.extend(paths);
        self.run_git(repo_path, &args)?;
        Ok(())
    }

    fn unstage(&self, repo_path: &Path, paths: &[&str]) -> Result<(), ShellError> {
        if paths.is_empty() {
            return Err(ShellError::Validation(
                "unstage requires at least one path".to_string(),
            ));
        }
        let mut args = vec!["restore", "--staged", "--"];
        args.extend(paths);
        self.run_git(repo_path, &args)?;
        Ok(())
    }

    fn stage_lines(
        &self,
        repo_path: &Path,
        file_path: &str,
        line_ranges: &[(u32, u32)],
    ) -> Result<(), ShellError> {
        if line_ranges.is_empty() {
            return Err(ShellError::Validation(
                "stage_lines requires at least one line range".to_string(),
            ));
        }

        // Get the unstaged diff for the file to extract relevant hunks
        let diff_output = self.run_git(repo_path, &["diff", "--", file_path])?;
        if diff_output.trim().is_empty() {
            return Err(ShellError::Validation(format!(
                "no unstaged changes found for {file_path}"
            )));
        }

        // Filter the diff to only include lines within the requested ranges,
        // then apply the filtered patch to the index.
        let filtered_patch = filter_diff_to_line_ranges(&diff_output, line_ranges);
        if filtered_patch.is_empty() {
            return Err(ShellError::Validation(
                "no matching lines found in diff for the given ranges".to_string(),
            ));
        }

        // Apply the filtered patch to the index via stdin
        let output = Command::new("git")
            .args(["apply", "--cached", "--allow-empty", "-"])
            .current_dir(repo_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ShellError::Io(format!("failed to spawn git apply: {e}")))?;

        use std::io::Write;
        let mut child = output;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(filtered_patch.as_bytes())
                .map_err(|e| ShellError::Io(format!("failed to write patch to stdin: {e}")))?;
        }

        let result = child
            .wait_with_output()
            .map_err(|e| ShellError::Io(format!("failed to wait for git apply: {e}")))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(ShellError::Io(format!(
                "git apply --cached failed (exit {}): {}",
                result
                    .status
                    .code()
                    .map_or("unknown".to_string(), |c| c.to_string()),
                stderr.trim()
            )));
        }

        Ok(())
    }

    fn commit(
        &self,
        repo_path: &Path,
        message: &str,
        amend: bool,
    ) -> Result<CommitInfo, ShellError> {
        if message.trim().is_empty() {
            return Err(ShellError::Validation(
                "commit message must not be empty".to_string(),
            ));
        }
        let mut args = vec!["commit", "-m", message];
        if amend {
            args.push("--amend");
        }
        let output = self.run_git(repo_path, &args)?;

        // Parse the commit hash from `git show` after committing
        let show_output = self.run_git(
            repo_path,
            &[
                "show",
                "-s",
                "--format=%H%n%h%n%an%n%ae%n%aI%n%P%n%s",
                "HEAD",
            ],
        )?;

        parse_commit_show_output(&show_output, &output)
    }

    fn push(
        &self,
        repo_path: &Path,
        remote: &RemoteName,
        branch: &BranchName,
    ) -> Result<(), ShellError> {
        self.run_git(repo_path, &["push", remote.as_ref(), branch.as_ref()])?;
        Ok(())
    }

    fn pull(
        &self,
        repo_path: &Path,
        remote: &RemoteName,
        branch: &BranchName,
    ) -> Result<(), ShellError> {
        self.run_git(repo_path, &["pull", remote.as_ref(), branch.as_ref()])?;
        Ok(())
    }

    fn fetch(&self, repo_path: &Path, remote: &RemoteName) -> Result<(), ShellError> {
        self.run_git(repo_path, &["fetch", remote.as_ref()])?;
        Ok(())
    }

    fn branches(&self, repo_path: &Path) -> Result<Vec<BranchInfo>, ShellError> {
        let output = self.run_git(
            repo_path,
            &[
                "branch",
                "-vv",
                "--format=%(HEAD)%(refname:short)\t%(upstream:short)\t%(upstream:track,nobracket)",
            ],
        )?;
        parse_branch_list(&output)
    }

    fn checkout_branch(&self, repo_path: &Path, branch: &BranchName) -> Result<(), ShellError> {
        self.run_git(repo_path, &["checkout", branch.as_ref()])?;
        Ok(())
    }

    fn create_branch(
        &self,
        repo_path: &Path,
        branch: &BranchName,
        start_point: Option<&BranchName>,
    ) -> Result<(), ShellError> {
        match start_point {
            Some(sp) => {
                self.run_git(repo_path, &["checkout", "-b", branch.as_ref(), sp.as_ref()])?;
            }
            None => {
                self.run_git(repo_path, &["checkout", "-b", branch.as_ref()])?;
            }
        }
        Ok(())
    }

    fn delete_branch(
        &self,
        repo_path: &Path,
        branch: &BranchName,
        force: bool,
    ) -> Result<(), ShellError> {
        let flag = if force { "-D" } else { "-d" };
        self.run_git(repo_path, &["branch", flag, branch.as_ref()])?;
        Ok(())
    }

    fn merge_branch(&self, repo_path: &Path, branch: &BranchName) -> Result<(), ShellError> {
        self.run_git(repo_path, &["merge", branch.as_ref()])?;
        Ok(())
    }

    fn log(
        &self,
        repo_path: &Path,
        max_count: u32,
        skip: u32,
    ) -> Result<Vec<CommitInfo>, ShellError> {
        // Custom format: fields separated by record-separator (0x1e), commits by group-separator (0x1d)
        let format = "%H%x1e%h%x1e%an%x1e%ae%x1e%aI%x1e%s%x1e%P%x1d";
        let mut args = vec![
            "log".to_string(),
            format!("--format={format}"),
            format!("-n{max_count}"),
        ];
        if skip > 0 {
            args.push(format!("--skip={skip}"));
        }
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = self.run_git(repo_path, &arg_refs)?;
        parse_log_output(&output)
    }

    fn show_commit(&self, repo_path: &Path, hash: &str) -> Result<Vec<DiffContent>, ShellError> {
        let output = self.run_git(repo_path, &["show", "--format=", "--find-renames", hash])?;
        Ok(parse_unified_diff(&output))
    }

    fn blame(&self, repo_path: &Path, file_path: &str) -> Result<Vec<BlameEntry>, ShellError> {
        let output = self.run_git(repo_path, &["blame", "--porcelain", file_path])?;
        parse_blame_porcelain(&output)
    }

    fn stash_push(&self, repo_path: &Path, message: Option<&str>) -> Result<(), ShellError> {
        match message {
            Some(msg) => {
                self.run_git(repo_path, &["stash", "push", "-m", msg])?;
            }
            None => {
                self.run_git(repo_path, &["stash", "push"])?;
            }
        }
        Ok(())
    }

    fn stash_pop(&self, repo_path: &Path) -> Result<(), ShellError> {
        self.run_git(repo_path, &["stash", "pop"])?;
        Ok(())
    }

    fn stash_list(&self, repo_path: &Path) -> Result<Vec<StashEntry>, ShellError> {
        // Format: index<RS>message<RS>date(ISO)<LF>
        let output = self.run_git(repo_path, &["stash", "list", "--format=%gd%x1e%gs%x1e%aI"])?;
        parse_stash_list_output(&output)
    }

    fn stash_drop(&self, repo_path: &Path, stash_id: StashId) -> Result<(), ShellError> {
        let stash_ref = format!("{stash_id}");
        self.run_git(repo_path, &["stash", "drop", &stash_ref])?;
        Ok(())
    }

    fn discard_changes(&self, repo_path: &Path, paths: &[&str]) -> Result<(), ShellError> {
        if paths.is_empty() {
            return Err(ShellError::Validation(
                "discard_changes requires at least one path".to_string(),
            ));
        }

        // Separate tracked files (use git restore) from untracked (use git clean).
        // First get the status to determine which files are untracked.
        let status_output = self.run_git(repo_path, &["status", "--porcelain=v2"])?;
        let statuses = parse_porcelain_v2(&status_output)?;

        let mut tracked_paths: Vec<&str> = Vec::new();
        let mut untracked_paths: Vec<&str> = Vec::new();

        for path in paths {
            let is_untracked = statuses
                .iter()
                .any(|s| s.path() == *path && s.worktree_status() == FileStatusKind::Untracked);

            if is_untracked {
                untracked_paths.push(path);
            } else {
                tracked_paths.push(path);
            }
        }

        // Restore tracked files
        if !tracked_paths.is_empty() {
            let mut args = vec!["restore", "--"];
            args.extend(tracked_paths.iter());
            self.run_git(repo_path, &args)?;
        }

        // Clean untracked files
        if !untracked_paths.is_empty() {
            let mut args = vec!["clean", "-f", "--"];
            args.extend(untracked_paths.iter());
            self.run_git(repo_path, &args)?;
        }

        Ok(())
    }

    fn repo_state(&self, repo_path: &Path) -> Result<GitRepositoryState, ShellError> {
        // Get HEAD branch name (returns "HEAD" if detached)
        let head_output = self.run_git(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        let head_ref = head_output.trim();
        let is_detached = head_ref == "HEAD";
        let head_branch = if is_detached {
            None
        } else {
            BranchName::try_new(head_ref).ok()
        };

        // Check if working tree is clean
        let status_output = self.run_git(repo_path, &["status", "--porcelain"])?;
        let status_clean = status_output.trim().is_empty();

        let repo_dir = tabby_kernel::WorkingDirectory::new(repo_path.to_string_lossy().as_ref())
            .map_err(|e| ShellError::Io(format!("invalid repo path: {e}")))?;

        Ok(GitRepositoryState::new(
            repo_dir,
            head_branch,
            is_detached,
            status_clean,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tabby_git::DiffLineKind;

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

    // -----------------------------------------------------------------------
    // parse_unified_diff tests (GIT-016)
    // -----------------------------------------------------------------------

    #[test]
    fn diff_parse_empty_output_returns_empty_vec() {
        let result = parse_unified_diff("");
        assert!(result.is_empty());
    }

    #[test]
    fn diff_parse_whitespace_only_returns_empty_vec() {
        let result = parse_unified_diff("   \n  \n");
        assert!(result.is_empty());
    }

    #[test]
    fn diff_parse_single_hunk_modification() {
        let input = "\
diff --git a/src/main.rs b/src/main.rs
index abc1234..def5678 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
-    println!(\"hello\");
+    println!(\"hello world\");
+    println!(\"goodbye\");
 }
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);

        let diff = &result[0];
        assert_eq!(diff.file_path(), "src/main.rs");
        assert_eq!(diff.old_path(), None);
        assert!(!diff.is_binary());
        assert_eq!(diff.hunks().len(), 1);

        let hunk = &diff.hunks()[0];
        assert_eq!(hunk.old_start(), 1);
        assert_eq!(hunk.old_count(), 3);
        assert_eq!(hunk.new_start(), 1);
        assert_eq!(hunk.new_count(), 4);
        assert_eq!(hunk.lines().len(), 5);

        // Context line
        assert_eq!(hunk.lines()[0].kind(), DiffLineKind::Context);
        assert_eq!(hunk.lines()[0].old_line_no(), Some(1));
        assert_eq!(hunk.lines()[0].new_line_no(), Some(1));
        assert_eq!(hunk.lines()[0].content(), "fn main() {");

        // Deletion
        assert_eq!(hunk.lines()[1].kind(), DiffLineKind::Deletion);
        assert_eq!(hunk.lines()[1].old_line_no(), Some(2));
        assert_eq!(hunk.lines()[1].new_line_no(), None);

        // Additions
        assert_eq!(hunk.lines()[2].kind(), DiffLineKind::Addition);
        assert_eq!(hunk.lines()[2].old_line_no(), None);
        assert_eq!(hunk.lines()[2].new_line_no(), Some(2));

        assert_eq!(hunk.lines()[3].kind(), DiffLineKind::Addition);
        assert_eq!(hunk.lines()[3].old_line_no(), None);
        assert_eq!(hunk.lines()[3].new_line_no(), Some(3));

        // Closing context line
        assert_eq!(hunk.lines()[4].kind(), DiffLineKind::Context);
        assert_eq!(hunk.lines()[4].old_line_no(), Some(3));
        assert_eq!(hunk.lines()[4].new_line_no(), Some(4));
    }

    #[test]
    fn diff_parse_multi_hunk() {
        let input = "\
diff --git a/file.rs b/file.rs
index abc..def 100644
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,4 @@
 line1
+inserted
 line2
 line3
@@ -10,3 +11,2 @@
 line10
-removed
 line12
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].hunks().len(), 2);

        let h1 = &result[0].hunks()[0];
        assert_eq!(h1.old_start(), 1);
        assert_eq!(h1.new_start(), 1);
        assert_eq!(h1.lines().len(), 4);

        let h2 = &result[0].hunks()[1];
        assert_eq!(h2.old_start(), 10);
        assert_eq!(h2.new_start(), 11);
        assert_eq!(h2.lines().len(), 3);
    }

    #[test]
    fn diff_parse_binary_file() {
        let input = "\
diff --git a/image.png b/image.png
index abc1234..def5678 100644
Binary files a/image.png and b/image.png differ
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_binary());
        assert_eq!(result[0].file_path(), "image.png");
        assert!(result[0].hunks().is_empty());
    }

    #[test]
    fn diff_parse_new_file_all_additions() {
        let input = "\
diff --git a/new_file.rs b/new_file.rs
new file mode 100644
index 0000000..abc1234
--- /dev/null
+++ b/new_file.rs
@@ -0,0 +1,3 @@
+fn hello() {
+    println!(\"hi\");
+}
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);

        let diff = &result[0];
        assert_eq!(diff.file_path(), "new_file.rs");
        assert!(!diff.is_binary());
        assert_eq!(diff.hunks().len(), 1);

        let hunk = &diff.hunks()[0];
        assert_eq!(hunk.old_start(), 0);
        assert_eq!(hunk.old_count(), 0);
        assert_eq!(hunk.new_start(), 1);
        assert_eq!(hunk.new_count(), 3);

        // All lines should be additions
        for line in hunk.lines() {
            assert_eq!(line.kind(), DiffLineKind::Addition);
            assert_eq!(line.old_line_no(), None);
        }
        assert_eq!(hunk.lines()[0].new_line_no(), Some(1));
        assert_eq!(hunk.lines()[1].new_line_no(), Some(2));
        assert_eq!(hunk.lines()[2].new_line_no(), Some(3));
    }

    #[test]
    fn diff_parse_deleted_file_all_deletions() {
        let input = "\
diff --git a/removed.rs b/removed.rs
deleted file mode 100644
index abc1234..0000000
--- a/removed.rs
+++ /dev/null
@@ -1,2 +0,0 @@
-fn old() {}
-fn also_old() {}
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);

        let diff = &result[0];
        assert_eq!(diff.file_path(), "removed.rs");
        assert_eq!(diff.hunks().len(), 1);

        let hunk = &diff.hunks()[0];
        assert_eq!(hunk.old_start(), 1);
        assert_eq!(hunk.old_count(), 2);
        assert_eq!(hunk.new_start(), 0);
        assert_eq!(hunk.new_count(), 0);

        for line in hunk.lines() {
            assert_eq!(line.kind(), DiffLineKind::Deletion);
            assert_eq!(line.new_line_no(), None);
        }
        assert_eq!(hunk.lines()[0].old_line_no(), Some(1));
        assert_eq!(hunk.lines()[1].old_line_no(), Some(2));
    }

    #[test]
    fn diff_parse_rename_detection() {
        let input = "\
diff --git a/old_name.rs b/new_name.rs
similarity index 95%
rename from old_name.rs
rename to new_name.rs
index abc1234..def5678 100644
--- a/old_name.rs
+++ b/new_name.rs
@@ -1,3 +1,3 @@
 fn example() {
-    let x = 1;
+    let x = 2;
 }
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);

        let diff = &result[0];
        assert_eq!(diff.file_path(), "new_name.rs");
        assert_eq!(diff.old_path(), Some("old_name.rs"));
        assert_eq!(diff.hunks().len(), 1);
    }

    #[test]
    fn diff_parse_multiple_files() {
        let input = "\
diff --git a/a.rs b/a.rs
index abc..def 100644
--- a/a.rs
+++ b/a.rs
@@ -1,1 +1,2 @@
 line1
+added_in_a
diff --git a/b.rs b/b.rs
index ghi..jkl 100644
--- a/b.rs
+++ b/b.rs
@@ -1,2 +1,1 @@
 line1
-removed_in_b
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].file_path(), "a.rs");
        assert_eq!(result[1].file_path(), "b.rs");
        assert_eq!(result[0].hunks().len(), 1);
        assert_eq!(result[1].hunks().len(), 1);
    }

    #[test]
    fn diff_parse_hunk_header_without_count() {
        // When count is omitted it defaults to 1: "@@ -1 +1 @@"
        let input = "\
diff --git a/single.rs b/single.rs
index abc..def 100644
--- a/single.rs
+++ b/single.rs
@@ -1 +1 @@
-old
+new
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);

        let hunk = &result[0].hunks()[0];
        assert_eq!(hunk.old_start(), 1);
        assert_eq!(hunk.old_count(), 1);
        assert_eq!(hunk.new_start(), 1);
        assert_eq!(hunk.new_count(), 1);
    }

    #[test]
    fn diff_parse_hunk_with_context_text() {
        // Hunk header may include function context after "@@"
        let input = "\
diff --git a/lib.rs b/lib.rs
index abc..def 100644
--- a/lib.rs
+++ b/lib.rs
@@ -10,3 +10,4 @@ fn some_function()
 context
+addition
 context2
 context3
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);

        let hunk = &result[0].hunks()[0];
        assert!(hunk.header().contains("fn some_function()"));
        assert_eq!(hunk.old_start(), 10);
        assert_eq!(hunk.new_start(), 10);
    }

    #[test]
    fn diff_parse_no_newline_at_end_of_file_marker() {
        let input = "\
diff --git a/file.txt b/file.txt
index abc..def 100644
--- a/file.txt
+++ b/file.txt
@@ -1 +1 @@
-old content
\\ No newline at end of file
+new content
\\ No newline at end of file
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);

        let hunk = &result[0].hunks()[0];
        // Should have 2 lines (deletion + addition), the "no newline" markers are skipped
        assert_eq!(hunk.lines().len(), 2);
        assert_eq!(hunk.lines()[0].kind(), DiffLineKind::Deletion);
        assert_eq!(hunk.lines()[1].kind(), DiffLineKind::Addition);
    }

    #[test]
    fn diff_parse_file_mode_change() {
        let input = "\
diff --git a/script.sh b/script.sh
old mode 100644
new mode 100755
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_mode_change(), Some("100644 -> 100755"));
        assert!(result[0].hunks().is_empty());
    }

    // -----------------------------------------------------------------------
    // stage / unstage / commit / discard_changes argument construction tests (GIT-017)
    // -----------------------------------------------------------------------

    #[test]
    fn stage_rejects_empty_paths() {
        let adapter = CliGitAdapter::new();
        let result = adapter.stage(Path::new("/tmp"), &[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ShellError::Validation(msg) => {
                assert!(msg.contains("at least one path"), "msg: {msg}");
            }
            other => panic!("expected Validation error, got: {other:?}"),
        }
    }

    #[test]
    fn unstage_rejects_empty_paths() {
        let adapter = CliGitAdapter::new();
        let result = adapter.unstage(Path::new("/tmp"), &[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ShellError::Validation(msg) => {
                assert!(msg.contains("at least one path"), "msg: {msg}");
            }
            other => panic!("expected Validation error, got: {other:?}"),
        }
    }

    #[test]
    fn commit_rejects_empty_message() {
        let adapter = CliGitAdapter::new();
        let result = adapter.commit(Path::new("/tmp"), "", false);
        assert!(result.is_err());
        match result.unwrap_err() {
            ShellError::Validation(msg) => {
                assert!(msg.contains("must not be empty"), "msg: {msg}");
            }
            other => panic!("expected Validation error, got: {other:?}"),
        }
    }

    #[test]
    fn commit_rejects_whitespace_only_message() {
        let adapter = CliGitAdapter::new();
        let result = adapter.commit(Path::new("/tmp"), "   \t  ", false);
        assert!(result.is_err());
        match result.unwrap_err() {
            ShellError::Validation(msg) => {
                assert!(msg.contains("must not be empty"), "msg: {msg}");
            }
            other => panic!("expected Validation error, got: {other:?}"),
        }
    }

    #[test]
    fn discard_changes_rejects_empty_paths() {
        let adapter = CliGitAdapter::new();
        let result = adapter.discard_changes(Path::new("/tmp"), &[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ShellError::Validation(msg) => {
                assert!(msg.contains("at least one path"), "msg: {msg}");
            }
            other => panic!("expected Validation error, got: {other:?}"),
        }
    }

    #[test]
    fn stage_lines_rejects_empty_line_ranges() {
        let adapter = CliGitAdapter::new();
        let result = adapter.stage_lines(Path::new("/tmp"), "file.rs", &[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ShellError::Validation(msg) => {
                assert!(msg.contains("at least one line range"), "msg: {msg}");
            }
            other => panic!("expected Validation error, got: {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // parse_commit_show_output tests (GIT-017)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_commit_show_output_basic() {
        let show_output = "\
abc123def456abc123def456abc123def456abc1
abc123d
Alice
alice@example.com
2026-03-10T01:00:00+00:00
1111111111111111111111111111111111111111
feat: add new feature
";
        let result = parse_commit_show_output(show_output, "").expect("should parse");
        assert_eq!(
            result.hash().as_ref(),
            "abc123def456abc123def456abc123def456abc1"
        );
        assert_eq!(result.short_hash(), "abc123d");
        assert_eq!(result.author_name(), "Alice");
        assert_eq!(result.author_email(), "alice@example.com");
        assert_eq!(result.date(), "2026-03-10T01:00:00+00:00");
        assert_eq!(result.message(), "feat: add new feature");
        assert_eq!(result.parent_hashes().len(), 1);
    }

    #[test]
    fn parse_commit_show_output_no_parents() {
        let show_output = "\
abc123def456abc123def456abc123def456abc1
abc123d
Alice
alice@example.com
2026-03-10T01:00:00+00:00

initial commit
";
        let result = parse_commit_show_output(show_output, "").expect("should parse");
        assert!(result.parent_hashes().is_empty());
        assert_eq!(result.message(), "initial commit");
    }

    #[test]
    fn parse_commit_show_output_multiple_parents() {
        let show_output = "\
abc123def456abc123def456abc123def456abc1
abc123d
Alice
alice@example.com
2026-03-10T01:00:00+00:00
1111111111111111111111111111111111111111 2222222222222222222222222222222222222222
Merge branch 'feature'
";
        let result = parse_commit_show_output(show_output, "").expect("should parse");
        assert_eq!(result.parent_hashes().len(), 2);
    }

    #[test]
    fn parse_commit_show_output_rejects_insufficient_lines() {
        let show_output = "abc123\nshort\n";
        let result = parse_commit_show_output(show_output, "");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // filter_diff_to_line_ranges tests (GIT-017)
    // -----------------------------------------------------------------------

    #[test]
    fn filter_diff_keeps_additions_in_range() {
        let diff = "\
diff --git a/file.rs b/file.rs
index abc..def 100644
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,5 @@
 line1
+added_line2
+added_line3
 line2
 line3
";
        let filtered = filter_diff_to_line_ranges(diff, &[(2, 2)]);
        assert!(filtered.contains("+added_line2"));
        // added_line3 (new line 3) should be excluded
        assert!(!filtered.contains("+added_line3"));
    }

    #[test]
    fn filter_diff_returns_empty_when_no_lines_match() {
        let diff = "\
diff --git a/file.rs b/file.rs
index abc..def 100644
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
";
        // Line 2 is the addition, but range 100-200 won't match
        let filtered = filter_diff_to_line_ranges(diff, &[(100, 200)]);
        // No hunk should be emitted since no changes match
        assert!(!filtered.contains("@@"));
    }

    #[test]
    fn filter_diff_keeps_deletions_in_range() {
        let diff = "\
diff --git a/file.rs b/file.rs
index abc..def 100644
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,2 @@
 line1
-removed_line
 line3
";
        let filtered = filter_diff_to_line_ranges(diff, &[(2, 2)]);
        assert!(filtered.contains("-removed_line"));
    }

    // -----------------------------------------------------------------------
    // parse_branch_list tests (GIT-018)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_branch_list_empty_output() {
        let result = parse_branch_list("").expect("should parse empty output");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_branch_list_single_current_branch() {
        let output = "*main\torigin/main\t\n";
        let result = parse_branch_list(output).expect("should parse single branch");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name().as_ref(), "main");
        assert!(result[0].is_current());
        assert_eq!(result[0].upstream(), Some("origin/main"));
        assert_eq!(result[0].ahead(), 0);
        assert_eq!(result[0].behind(), 0);
    }

    #[test]
    fn parse_branch_list_non_current_branch() {
        let output = " feature/login\torigin/feature/login\t\n";
        let result = parse_branch_list(output).expect("should parse non-current branch");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name().as_ref(), "feature/login");
        assert!(!result[0].is_current());
        assert_eq!(result[0].upstream(), Some("origin/feature/login"));
    }

    #[test]
    fn parse_branch_list_with_ahead_behind() {
        let output = "*main\torigin/main\tahead 2, behind 1\n";
        let result = parse_branch_list(output).expect("should parse ahead/behind");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].ahead(), 2);
        assert_eq!(result[0].behind(), 1);
    }

    #[test]
    fn parse_branch_list_ahead_only() {
        let output = "*develop\torigin/develop\tahead 5\n";
        let result = parse_branch_list(output).expect("should parse ahead only");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].ahead(), 5);
        assert_eq!(result[0].behind(), 0);
    }

    #[test]
    fn parse_branch_list_behind_only() {
        let output = " staging\torigin/staging\tbehind 3\n";
        let result = parse_branch_list(output).expect("should parse behind only");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].ahead(), 0);
        assert_eq!(result[0].behind(), 3);
    }

    #[test]
    fn parse_branch_list_no_upstream() {
        let output = " local-only\t\t\n";
        let result = parse_branch_list(output).expect("should parse branch with no upstream");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name().as_ref(), "local-only");
        assert_eq!(result[0].upstream(), None);
        assert_eq!(result[0].ahead(), 0);
        assert_eq!(result[0].behind(), 0);
    }

    #[test]
    fn parse_branch_list_multiple_branches() {
        let output = "\
*main\torigin/main\tahead 1
 develop\torigin/develop\tbehind 2
 feature/auth\t\t
 release-v1.0\torigin/release-v1.0\tahead 3, behind 1
";
        let result = parse_branch_list(output).expect("should parse multiple branches");
        assert_eq!(result.len(), 4);

        assert_eq!(result[0].name().as_ref(), "main");
        assert!(result[0].is_current());
        assert_eq!(result[0].ahead(), 1);

        assert_eq!(result[1].name().as_ref(), "develop");
        assert!(!result[1].is_current());
        assert_eq!(result[1].behind(), 2);

        assert_eq!(result[2].name().as_ref(), "feature/auth");
        assert_eq!(result[2].upstream(), None);

        assert_eq!(result[3].name().as_ref(), "release-v1.0");
        assert_eq!(result[3].ahead(), 3);
        assert_eq!(result[3].behind(), 1);
    }

    #[test]
    fn parse_branch_list_skips_detached_head() {
        let output = "*(HEAD detached at abc1234)\t\t\n main\torigin/main\t\n";
        let result = parse_branch_list(output).expect("should skip detached HEAD");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name().as_ref(), "main");
    }

    #[test]
    fn parse_branch_list_gone_upstream() {
        let output = " stale-branch\torigin/stale-branch\tgone\n";
        let result = parse_branch_list(output).expect("should handle gone upstream");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].upstream(), Some("origin/stale-branch"));
        assert_eq!(result[0].ahead(), 0);
        assert_eq!(result[0].behind(), 0);
    }

    #[test]
    fn parse_branch_list_skips_empty_lines() {
        let output = "\n*main\torigin/main\t\n\n develop\t\t\n\n";
        let result = parse_branch_list(output).expect("should skip empty lines");
        assert_eq!(result.len(), 2);
    }

    // -----------------------------------------------------------------------
    // parse_tracking_info tests (GIT-018)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_tracking_info_empty() {
        assert_eq!(parse_tracking_info(""), (0, 0));
    }

    #[test]
    fn parse_tracking_info_gone() {
        assert_eq!(parse_tracking_info("gone"), (0, 0));
    }

    #[test]
    fn parse_tracking_info_ahead_only() {
        assert_eq!(parse_tracking_info("ahead 7"), (7, 0));
    }

    #[test]
    fn parse_tracking_info_behind_only() {
        assert_eq!(parse_tracking_info("behind 4"), (0, 4));
    }

    #[test]
    fn parse_tracking_info_ahead_and_behind() {
        assert_eq!(parse_tracking_info("ahead 2, behind 1"), (2, 1));
    }

    #[test]
    fn parse_tracking_info_large_numbers() {
        assert_eq!(parse_tracking_info("ahead 999, behind 500"), (999, 500));
    }

    // -----------------------------------------------------------------------
    // parse_log_output tests (GIT-019)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_log_output_single_commit() {
        let output = "abc123def456abc123def456abc123def456abc1\x1eabc123d\x1eAlice\x1ealice@example.com\x1e2026-03-10T01:00:00+00:00\x1efeat: initial commit\x1e1111111111111111111111111111111111111111\x1d";
        let result = parse_log_output(output).expect("should parse single commit");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].short_hash(), "abc123d");
        assert_eq!(result[0].author_name(), "Alice");
        assert_eq!(result[0].author_email(), "alice@example.com");
        assert_eq!(result[0].message(), "feat: initial commit");
        assert_eq!(result[0].parent_hashes().len(), 1);
    }

    #[test]
    fn parse_log_output_multiple_commits() {
        let output = format!(
            "{hash1}\x1eabc1\x1eAlice\x1ea@b.com\x1e2026-03-10\x1efirst\x1e{parent}\x1d\
             {hash2}\x1edef2\x1eBob\x1eb@c.com\x1e2026-03-09\x1esecond\x1e{hash1}\x1d",
            hash1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            hash2 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            parent = "cccccccccccccccccccccccccccccccccccccccc",
        );
        let result = parse_log_output(&output).expect("should parse multiple commits");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].author_name(), "Alice");
        assert_eq!(result[1].author_name(), "Bob");
    }

    #[test]
    fn parse_log_output_root_commit_no_parents() {
        let output = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x1eaaaa\x1eAlice\x1ea@b.com\x1e2026-01-01\x1einit\x1e\x1d";
        let result = parse_log_output(output).expect("should parse root commit");
        assert_eq!(result.len(), 1);
        assert!(result[0].parent_hashes().is_empty());
    }

    #[test]
    fn parse_log_output_merge_commit_two_parents() {
        let parent1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let parent2 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let output = format!(
            "cccccccccccccccccccccccccccccccccccccccc\x1ecccc\x1eAlice\x1ea@b.com\x1e2026-01-01\x1emerge\x1e{parent1} {parent2}\x1d"
        );
        let result = parse_log_output(&output).expect("should parse merge commit");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].parent_hashes().len(), 2);
    }

    #[test]
    fn parse_log_output_empty() {
        let result = parse_log_output("").expect("should handle empty");
        assert!(result.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse_blame_porcelain tests (GIT-019)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_blame_porcelain_single_block() {
        let output = "\
deadbeefdeadbeefdeadbeefdeadbeefdeadbeef 1 1 3
author Alice
author-mail <alice@example.com>
author-time 1709856000
author-tz +0000
committer Alice
committer-mail <alice@example.com>
committer-time 1709856000
committer-tz +0000
summary initial commit
filename src/main.rs
\tfn main() {
deadbeefdeadbeefdeadbeefdeadbeefdeadbeef 2 2
\t    println!(\"hello\");
deadbeefdeadbeefdeadbeefdeadbeefdeadbeef 3 3
\t}";
        let result = parse_blame_porcelain(output).expect("should parse blame");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].author(), "Alice");
        assert_eq!(result[0].line_start(), 1);
        assert_eq!(result[0].line_count(), 3);
        assert!(result[0].content().contains("fn main()"));
        assert!(result[0].content().contains("println!"));
    }

    #[test]
    fn parse_blame_porcelain_two_commits() {
        let hash_a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let hash_b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let output = format!(
            "\
{hash_a} 1 1 1
author Alice
author-time 1709856000
filename file.rs
\tline one
{hash_b} 2 2 1
author Bob
author-time 1709856100
filename file.rs
\tline two"
        );
        let result = parse_blame_porcelain(&output).expect("should parse two commits");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].author(), "Alice");
        assert_eq!(result[0].line_start(), 1);
        assert_eq!(result[0].line_count(), 1);
        assert_eq!(result[1].author(), "Bob");
        assert_eq!(result[1].line_start(), 2);
        assert_eq!(result[1].line_count(), 1);
    }

    #[test]
    fn parse_blame_porcelain_empty() {
        let result = parse_blame_porcelain("").expect("should handle empty");
        assert!(result.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse_stash_list_output tests (GIT-019)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_stash_list_single_entry() {
        let output =
            "stash@{0}\x1eWIP on main: abc1234 feat: something\x1e2026-03-10T01:00:00+00:00\n";
        let result = parse_stash_list_output(output).expect("should parse single stash");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index(), StashId::new(0));
        assert_eq!(result[0].message(), "WIP on main: abc1234 feat: something");
    }

    #[test]
    fn parse_stash_list_multiple_entries() {
        let output = "\
stash@{0}\x1eWIP on main\x1e2026-03-10T01:00:00+00:00
stash@{1}\x1efix: save work\x1e2026-03-09T12:00:00+00:00
stash@{2}\x1erefactor\x1e2026-03-08T08:00:00+00:00
";
        let result = parse_stash_list_output(output).expect("should parse multiple stashes");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].index(), StashId::new(0));
        assert_eq!(result[1].index(), StashId::new(1));
        assert_eq!(result[2].index(), StashId::new(2));
        assert_eq!(result[2].message(), "refactor");
    }

    #[test]
    fn parse_stash_list_empty() {
        let result = parse_stash_list_output("").expect("should handle empty");
        assert!(result.is_empty());
    }
}
