use std::path::Path;
use std::process::Command;

use tabby_git::value_objects::{BranchName, RemoteName, StashId};
use tabby_git::{
    BlameEntry, BranchInfo, CommitInfo, DiffContent, DiffHunk, DiffLine, DiffLineKind, FileStatus,
    FileStatusKind, GitRepositoryState, StashEntry,
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

/// Parse the unified diff output from `git diff` into a list of `DiffContent` entries.
///
/// Handles:
/// - `diff --git a/file b/file` headers
/// - `--- a/file` / `+++ b/file` headers
/// - `@@ -old_start,old_count +new_start,new_count @@ optional context` hunk headers
/// - Context (space prefix), addition (`+` prefix), deletion (`-` prefix) lines
/// - Binary files (`Binary files ... differ`)
/// - Renames via `rename from` / `rename to` in extended headers
/// - New files (all additions, `--- /dev/null`)
/// - Deleted files (all deletions, `+++ /dev/null`)
/// - Empty diff (no output) returns empty vec
fn parse_unified_diff(output: &str) -> Vec<DiffContent> {
    if output.trim().is_empty() {
        return Vec::new();
    }

    let mut results: Vec<DiffContent> = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Look for "diff --git a/... b/..."
        if !line.starts_with("diff --git ") {
            i += 1;
            continue;
        }

        // Extract file path from "diff --git a/path b/path"
        let after_prefix = &line["diff --git ".len()..];
        let file_path = extract_diff_git_path(after_prefix);

        let mut old_path: Option<String> = None;
        let mut is_binary = false;
        let mut file_mode_change: Option<String> = None;
        let mut hunks: Vec<DiffHunk> = Vec::new();
        let mut new_file_path = file_path.clone();

        i += 1;

        // Parse extended headers (old mode, new mode, rename from/to, index, etc.)
        while i < lines.len() && !lines[i].starts_with("diff --git ") {
            let eline = lines[i];

            if let Some(rest) = eline.strip_prefix("rename from ") {
                old_path = Some(rest.to_string());
            } else if let Some(rest) = eline.strip_prefix("rename to ") {
                new_file_path = rest.to_string();
            } else if let Some(rest) = eline.strip_prefix("old mode ") {
                let old_mode = rest.to_string();
                // Check for "new mode" on next line
                if i + 1 < lines.len() {
                    if let Some(new_mode) = lines[i + 1].strip_prefix("new mode ") {
                        file_mode_change = Some(format!("{old_mode} -> {new_mode}"));
                        i += 1;
                    }
                }
            } else if eline.starts_with("Binary files ") && eline.ends_with(" differ") {
                is_binary = true;
            } else if eline.starts_with("--- ") || eline.starts_with("+++ ") {
                // --- a/file or --- /dev/null
                // +++ b/file or +++ /dev/null
                // Skip these, we already have the file path from the diff header
            } else if eline.starts_with("@@ ") {
                // Start of a hunk — parse it
                if let Some(hunk) = parse_hunk_at(&lines, &mut i) {
                    hunks.push(hunk);
                    continue; // parse_hunk_at already advanced i
                }
            }

            i += 1;
        }

        results.push(DiffContent::new(
            new_file_path,
            old_path,
            hunks,
            is_binary,
            file_mode_change,
        ));
    }

    results
}

/// Extract the new file path from the "diff --git a/path b/path" line content
/// (after stripping the "diff --git " prefix).
///
/// The format is "a/<path> b/<path>". We take the b/ side.
fn extract_diff_git_path(after_prefix: &str) -> String {
    // Split on " b/" — the last occurrence handles paths with spaces
    if let Some(pos) = after_prefix.rfind(" b/") {
        after_prefix[pos + 3..].to_string()
    } else {
        // Fallback: try splitting on space and taking the second half
        let parts: Vec<&str> = after_prefix.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let b_part = parts[1];
            b_part.strip_prefix("b/").unwrap_or(b_part).to_string()
        } else {
            after_prefix.to_string()
        }
    }
}

/// Parse a single hunk starting at `lines[*i]` which must begin with "@@".
/// Advances `*i` past all lines belonging to this hunk.
/// Returns the parsed `DiffHunk`, or `None` if the header couldn't be parsed.
fn parse_hunk_at(lines: &[&str], i: &mut usize) -> Option<DiffHunk> {
    let header_line = lines[*i];

    // Parse "@@ -old_start,old_count +new_start,new_count @@ context"
    let (old_start, old_count, new_start, new_count) = parse_hunk_header(header_line)?;

    let header = header_line.to_string();
    let mut hunk_lines: Vec<DiffLine> = Vec::new();
    let mut old_line = old_start;
    let mut new_line = new_start;

    *i += 1;

    while *i < lines.len() {
        let line = lines[*i];

        // Stop at next hunk header or next diff header
        if line.starts_with("diff --git ") || line.starts_with("@@ ") {
            break;
        }

        if let Some(content) = line.strip_prefix('+') {
            hunk_lines.push(DiffLine::new(
                DiffLineKind::Addition,
                None,
                Some(new_line),
                content,
            ));
            new_line += 1;
        } else if let Some(content) = line.strip_prefix('-') {
            hunk_lines.push(DiffLine::new(
                DiffLineKind::Deletion,
                Some(old_line),
                None,
                content,
            ));
            old_line += 1;
        } else if let Some(content) = line.strip_prefix(' ') {
            hunk_lines.push(DiffLine::new(
                DiffLineKind::Context,
                Some(old_line),
                Some(new_line),
                content,
            ));
            old_line += 1;
            new_line += 1;
        } else if line == "\\ No newline at end of file" {
            // Skip this marker
        } else {
            // Unknown line — could be end of diff body
            break;
        }

        *i += 1;
    }

    Some(DiffHunk::new(
        old_start, old_count, new_start, new_count, header, hunk_lines,
    ))
}

/// Parse the hunk header line "@@ -start,count +start,count @@ ..."
/// Returns (old_start, old_count, new_start, new_count).
fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32)> {
    // Format: "@@ -old_start[,old_count] +new_start[,new_count] @@[ context]"
    let after_at = line.strip_prefix("@@ ")?;
    let end_at = after_at.find(" @@")?;
    let range_part = &after_at[..end_at];

    let parts: Vec<&str> = range_part.split(' ').collect();
    if parts.len() != 2 {
        return None;
    }

    let old_range = parts[0].strip_prefix('-')?;
    let new_range = parts[1].strip_prefix('+')?;

    let (old_start, old_count) = parse_range(old_range)?;
    let (new_start, new_count) = parse_range(new_range)?;

    Some((old_start, old_count, new_start, new_count))
}

/// Parse a range like "10,5" or "10" (count defaults to 1) into (start, count).
fn parse_range(range: &str) -> Option<(u32, u32)> {
    if let Some((start_s, count_s)) = range.split_once(',') {
        let start = start_s.parse::<u32>().ok()?;
        let count = count_s.parse::<u32>().ok()?;
        Some((start, count))
    } else {
        let start = range.parse::<u32>().ok()?;
        Some((start, 1))
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
}
