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

        let diff_output = self.run_git(repo_path, &["diff", "--", file_path])?;
        if diff_output.trim().is_empty() {
            return Err(ShellError::Validation(format!(
                "no unstaged changes found for {file_path}"
            )));
        }

        let filtered_patch = filter_diff_to_line_ranges(&diff_output, line_ranges);
        if filtered_patch.is_empty() {
            return Err(ShellError::Validation(
                "no matching lines found in diff for the given ranges".to_string(),
            ));
        }

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

        if !tracked_paths.is_empty() {
            let mut args = vec!["restore", "--"];
            args.extend(tracked_paths.iter());
            self.run_git(repo_path, &args)?;
        }

        if !untracked_paths.is_empty() {
            let mut args = vec!["clean", "-f", "--"];
            args.extend(untracked_paths.iter());
            self.run_git(repo_path, &args)?;
        }

        Ok(())
    }

    fn repo_state(&self, repo_path: &Path) -> Result<GitRepositoryState, ShellError> {
        let head_output = self.run_git(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        let head_ref = head_output.trim();
        let is_detached = head_ref == "HEAD";
        let head_branch = if is_detached {
            None
        } else {
            BranchName::try_new(head_ref).ok()
        };

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

    use super::super::porcelain_parser::{parse_tracking_info, status_char_to_kind};

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

    #[test]
    fn parse_clean_repo_returns_empty_vec() {
        let result = parse_porcelain_v2("").expect("should parse empty output");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_modified_file() {
        let output = "1 .M N... 100644 100644 100644 abc123 def456 src/main.rs\n";
        let result = parse_porcelain_v2(output).expect("should parse modified entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "src/main.rs");
        assert_eq!(result[0].worktree_status(), FileStatusKind::Modified);
    }

    #[test]
    fn parse_untracked_file() {
        let output = "? untracked_file.txt\n";
        let result = parse_porcelain_v2(output).expect("should parse untracked entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index_status(), FileStatusKind::Untracked);
    }

    #[test]
    fn parse_renamed_file() {
        let output =
            "2 R. N... 100644 100644 100644 abc1234 def5678 R100 new_name.rs\told_name.rs\n";
        let result = parse_porcelain_v2(output).expect("should parse renamed entry");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "new_name.rs");
        assert_eq!(result[0].old_path(), Some("old_name.rs"));
        assert_eq!(result[0].index_status(), FileStatusKind::Renamed);
    }

    #[test]
    fn parse_mixed_status_output() {
        let output = "# branch.oid abc123def456\n# branch.head main\n1 .M N... 100644 100644 100644 abc1234 def5678 src/lib.rs\n1 A. N... 000000 100644 100644 0000000 abc1234 src/new.rs\n2 R. N... 100644 100644 100644 abc1234 def5678 R100 renamed.rs\told.rs\n? untracked.txt\nu UU N... 100644 100644 100644 100644 abc1234 def5678 ghi9012 merge_conflict.rs\n! ignored.log\n";
        let result = parse_porcelain_v2(output).expect("should parse mixed output");
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn status_char_mapping_covers_all_codes() {
        assert_eq!(status_char_to_kind('M'), FileStatusKind::Modified);
        assert_eq!(status_char_to_kind('A'), FileStatusKind::Added);
        assert_eq!(status_char_to_kind('D'), FileStatusKind::Deleted);
        assert_eq!(status_char_to_kind('R'), FileStatusKind::Renamed);
        assert_eq!(status_char_to_kind('C'), FileStatusKind::Copied);
        assert_eq!(status_char_to_kind('U'), FileStatusKind::Conflicted);
        assert_eq!(status_char_to_kind('.'), FileStatusKind::Modified);
    }

    #[test]
    fn diff_parse_empty_output_returns_empty_vec() {
        let result = parse_unified_diff("");
        assert!(result.is_empty());
    }

    #[test]
    fn diff_parse_single_hunk_modification() {
        let input = "diff --git a/src/main.rs b/src/main.rs\nindex abc1234..def5678 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n-    println!(\"hello\");\n+    println!(\"hello world\");\n+    println!(\"goodbye\");\n }\n";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_path(), "src/main.rs");
        assert_eq!(result[0].hunks().len(), 1);
        assert_eq!(result[0].hunks()[0].lines().len(), 5);
    }

    #[test]
    fn diff_parse_binary_file() {
        let input = "diff --git a/image.png b/image.png\nindex abc1234..def5678 100644\nBinary files a/image.png and b/image.png differ\n";
        let result = parse_unified_diff(input);
        assert!(result[0].is_binary());
    }

    #[test]
    fn diff_parse_rename_detection() {
        let input = "diff --git a/old_name.rs b/new_name.rs\nsimilarity index 95%\nrename from old_name.rs\nrename to new_name.rs\nindex abc1234..def5678 100644\n--- a/old_name.rs\n+++ b/new_name.rs\n@@ -1,3 +1,3 @@\n fn example() {\n-    let x = 1;\n+    let x = 2;\n }\n";
        let result = parse_unified_diff(input);
        assert_eq!(result[0].file_path(), "new_name.rs");
        assert_eq!(result[0].old_path(), Some("old_name.rs"));
    }

    #[test]
    fn diff_parse_file_mode_change() {
        let input = "diff --git a/script.sh b/script.sh\nold mode 100644\nnew mode 100755\n";
        let result = parse_unified_diff(input);
        assert_eq!(result[0].file_mode_change(), Some("100644 -> 100755"));
    }

    #[test]
    fn diff_parse_no_newline_marker() {
        let input = "diff --git a/file.txt b/file.txt\nindex abc..def 100644\n--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old content\n\\ No newline at end of file\n+new content\n\\ No newline at end of file\n";
        let result = parse_unified_diff(input);
        assert_eq!(result[0].hunks()[0].lines().len(), 2);
        assert_eq!(
            result[0].hunks()[0].lines()[0].kind(),
            DiffLineKind::Deletion
        );
        assert_eq!(
            result[0].hunks()[0].lines()[1].kind(),
            DiffLineKind::Addition
        );
    }

    #[test]
    fn stage_rejects_empty_paths() {
        let adapter = CliGitAdapter::new();
        let result = adapter.stage(Path::new("/tmp"), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn unstage_rejects_empty_paths() {
        let adapter = CliGitAdapter::new();
        let result = adapter.unstage(Path::new("/tmp"), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn commit_rejects_empty_message() {
        let adapter = CliGitAdapter::new();
        let result = adapter.commit(Path::new("/tmp"), "", false);
        assert!(result.is_err());
    }

    #[test]
    fn discard_changes_rejects_empty_paths() {
        let adapter = CliGitAdapter::new();
        let result = adapter.discard_changes(Path::new("/tmp"), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn stage_lines_rejects_empty_line_ranges() {
        let adapter = CliGitAdapter::new();
        let result = adapter.stage_lines(Path::new("/tmp"), "file.rs", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_commit_show_output_basic() {
        let show_output = "abc123def456abc123def456abc123def456abc1\nabc123d\nAlice\nalice@example.com\n2026-03-10T01:00:00+00:00\n1111111111111111111111111111111111111111\nfeat: add new feature\n";
        let result = parse_commit_show_output(show_output, "").expect("should parse");
        assert_eq!(result.short_hash(), "abc123d");
        assert_eq!(result.parent_hashes().len(), 1);
    }

    #[test]
    fn parse_commit_show_output_no_parents() {
        let show_output = "abc123def456abc123def456abc123def456abc1\nabc123d\nAlice\nalice@example.com\n2026-03-10T01:00:00+00:00\n\ninitial commit\n";
        let result = parse_commit_show_output(show_output, "").expect("should parse");
        assert!(result.parent_hashes().is_empty());
    }

    #[test]
    fn parse_commit_show_output_rejects_insufficient_lines() {
        let result = parse_commit_show_output("abc123\nshort\n", "");
        assert!(result.is_err());
    }

    #[test]
    fn filter_diff_keeps_additions_in_range() {
        let diff = "diff --git a/file.rs b/file.rs\nindex abc..def 100644\n--- a/file.rs\n+++ b/file.rs\n@@ -1,3 +1,5 @@\n line1\n+added_line2\n+added_line3\n line2\n line3\n";
        let filtered = filter_diff_to_line_ranges(diff, &[(2, 2)]);
        assert!(filtered.contains("+added_line2"));
        assert!(!filtered.contains("+added_line3"));
    }

    #[test]
    fn filter_diff_returns_empty_when_no_lines_match() {
        let diff = "diff --git a/file.rs b/file.rs\nindex abc..def 100644\n--- a/file.rs\n+++ b/file.rs\n@@ -1,3 +1,4 @@\n line1\n+added\n line2\n line3\n";
        let filtered = filter_diff_to_line_ranges(diff, &[(100, 200)]);
        assert!(!filtered.contains("@@"));
    }

    #[test]
    fn parse_branch_list_single_current_branch() {
        let output = "*main\torigin/main\t\n";
        let result = parse_branch_list(output).expect("should parse");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name().as_ref(), "main");
        assert!(result[0].is_current());
    }

    #[test]
    fn parse_branch_list_with_ahead_behind() {
        let output = "*main\torigin/main\tahead 2, behind 1\n";
        let result = parse_branch_list(output).expect("should parse");
        assert_eq!(result[0].ahead(), 2);
        assert_eq!(result[0].behind(), 1);
    }

    #[test]
    fn parse_branch_list_no_upstream() {
        let output = " local-only\t\t\n";
        let result = parse_branch_list(output).expect("should parse");
        assert_eq!(result[0].upstream(), None);
    }

    #[test]
    fn parse_branch_list_skips_detached_head() {
        let output = "*(HEAD detached at abc1234)\t\t\n main\torigin/main\t\n";
        let result = parse_branch_list(output).expect("should skip detached HEAD");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn parse_tracking_info_empty() {
        assert_eq!(parse_tracking_info(""), (0, 0));
    }

    #[test]
    fn parse_tracking_info_ahead_and_behind() {
        assert_eq!(parse_tracking_info("ahead 2, behind 1"), (2, 1));
    }

    #[test]
    fn parse_tracking_info_gone() {
        assert_eq!(parse_tracking_info("gone"), (0, 0));
    }

    #[test]
    fn parse_log_output_single_commit() {
        let output = "abc123def456abc123def456abc123def456abc1\x1eabc123d\x1eAlice\x1ealice@example.com\x1e2026-03-10T01:00:00+00:00\x1efeat: initial commit\x1e1111111111111111111111111111111111111111\x1d";
        let result = parse_log_output(output).expect("should parse");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].author_name(), "Alice");
    }

    #[test]
    fn parse_log_output_empty() {
        let result = parse_log_output("").expect("should handle empty");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_blame_porcelain_single_block() {
        let output = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef 1 1 3\nauthor Alice\nauthor-mail <alice@example.com>\nauthor-time 1709856000\nauthor-tz +0000\ncommitter Alice\ncommitter-mail <alice@example.com>\ncommitter-time 1709856000\ncommitter-tz +0000\nsummary initial commit\nfilename src/main.rs\n\tfn main() {\ndeadbeefdeadbeefdeadbeefdeadbeefdeadbeef 2 2\n\t    println!(\"hello\");\ndeadbeefdeadbeefdeadbeefdeadbeefdeadbeef 3 3\n\t}";
        let result = parse_blame_porcelain(output).expect("should parse blame");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].author(), "Alice");
        assert_eq!(result[0].line_count(), 3);
    }

    #[test]
    fn parse_blame_porcelain_empty() {
        let result = parse_blame_porcelain("").expect("should handle empty");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_stash_list_single_entry() {
        let output =
            "stash@{0}\x1eWIP on main: abc1234 feat: something\x1e2026-03-10T01:00:00+00:00\n";
        let result = parse_stash_list_output(output).expect("should parse");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index(), StashId::new(0));
    }

    #[test]
    fn parse_stash_list_empty() {
        let result = parse_stash_list_output("").expect("should handle empty");
        assert!(result.is_empty());
    }
}
