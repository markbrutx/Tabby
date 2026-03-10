use std::path::Path;
use std::process::Command;

use tabby_git::value_objects::{BranchName, CommitHash, RemoteName, StashId};
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

/// Filter a unified diff to only include changes within the specified new-file line ranges.
///
/// Keeps context lines and only includes additions/deletions whose new-file line numbers
/// fall within one of the given `(start, end)` ranges (inclusive).
/// Returns a valid unified diff patch suitable for `git apply --cached`.
fn filter_diff_to_line_ranges(diff_output: &str, line_ranges: &[(u32, u32)]) -> String {
    let lines: Vec<&str> = diff_output.lines().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Pass through diff header lines
        if line.starts_with("diff --git ")
            || line.starts_with("index ")
            || line.starts_with("--- ")
            || line.starts_with("+++ ")
            || line.starts_with("new file ")
            || line.starts_with("deleted file ")
            || line.starts_with("old mode ")
            || line.starts_with("new mode ")
            || line.starts_with("similarity index ")
            || line.starts_with("rename from ")
            || line.starts_with("rename to ")
        {
            result.push_str(line);
            result.push('\n');
            i += 1;
            continue;
        }

        // Parse hunk headers and filter their content
        if line.starts_with("@@ ") {
            if let Some((old_start, _old_count, new_start, _new_count)) = parse_hunk_header(line) {
                // Collect all lines in this hunk
                let mut hunk_lines: Vec<&str> = Vec::new();
                let mut old_line = old_start;
                let mut new_line = new_start;

                // Track which hunk lines to keep (true = keep as-is, false = convert to context)
                let mut keep_flags: Vec<bool> = Vec::new();
                let mut line_numbers: Vec<(u32, u32)> = Vec::new(); // (old_line, new_line) at each position

                i += 1;
                while i < lines.len() {
                    let hline = lines[i];
                    if hline.starts_with("diff --git ") || hline.starts_with("@@ ") {
                        break;
                    }

                    if hline.starts_with('+') {
                        let in_range = line_ranges
                            .iter()
                            .any(|&(start, end)| new_line >= start && new_line <= end);
                        keep_flags.push(in_range);
                        line_numbers.push((old_line, new_line));
                        hunk_lines.push(hline);
                        new_line += 1;
                    } else if hline.starts_with('-') {
                        // For deletions, check if the old line number's corresponding new
                        // position falls in range
                        let in_range = line_ranges
                            .iter()
                            .any(|&(start, end)| old_line >= start && old_line <= end);
                        keep_flags.push(in_range);
                        line_numbers.push((old_line, new_line));
                        hunk_lines.push(hline);
                        old_line += 1;
                    } else if hline.starts_with(' ') {
                        keep_flags.push(true); // context always kept
                        line_numbers.push((old_line, new_line));
                        hunk_lines.push(hline);
                        old_line += 1;
                        new_line += 1;
                    } else if hline == "\\ No newline at end of file" {
                        keep_flags.push(true);
                        line_numbers.push((old_line, new_line));
                        hunk_lines.push(hline);
                    } else {
                        break;
                    }
                    i += 1;
                }

                // Check if any non-context lines are kept
                let has_changes = hunk_lines
                    .iter()
                    .zip(keep_flags.iter())
                    .any(|(l, &keep)| keep && (l.starts_with('+') || l.starts_with('-')));

                if !has_changes {
                    continue;
                }

                // Build filtered hunk: convert excluded changes to context lines
                let mut filtered: Vec<String> = Vec::new();
                let mut new_old_count: u32 = 0;
                let mut new_new_count: u32 = 0;

                for (hline, &keep) in hunk_lines.iter().zip(keep_flags.iter()) {
                    if hline.starts_with('+') {
                        if keep {
                            filtered.push(hline.to_string());
                            new_new_count += 1;
                        }
                        // Excluded additions are simply dropped
                    } else if let Some(content) = hline.strip_prefix('-') {
                        if keep {
                            filtered.push(hline.to_string());
                            new_old_count += 1;
                        } else {
                            // Convert excluded deletion to context
                            filtered.push(format!(" {content}"));
                            new_old_count += 1;
                            new_new_count += 1;
                        }
                    } else if hline.starts_with(' ') {
                        filtered.push(hline.to_string());
                        new_old_count += 1;
                        new_new_count += 1;
                    } else if *hline == "\\ No newline at end of file" {
                        filtered.push(hline.to_string());
                    }
                }

                result.push_str(&format!(
                    "@@ -{},{} +{},{} @@\n",
                    old_start, new_old_count, new_start, new_new_count
                ));
                for fl in &filtered {
                    result.push_str(fl);
                    result.push('\n');
                }
            } else {
                i += 1;
            }
            continue;
        }

        i += 1;
    }

    result
}

/// Parse the output of `git show -s --format=%H%n%h%n%an%n%ae%n%aI%n%P%n%s HEAD`
/// into a `CommitInfo`.
fn parse_commit_show_output(
    show_output: &str,
    _commit_output: &str,
) -> Result<CommitInfo, ShellError> {
    let lines: Vec<&str> = show_output.lines().collect();
    if lines.len() < 7 {
        return Err(ShellError::Io(format!(
            "unexpected git show output (expected 7 lines, got {}): {}",
            lines.len(),
            show_output
        )));
    }

    let full_hash = lines[0].trim();
    let short_hash = lines[1].trim();
    let author_name = lines[2].trim();
    let author_email = lines[3].trim();
    let date = lines[4].trim();
    let parent_line = lines[5].trim();
    let subject = lines[6].trim();

    let hash = CommitHash::try_new(full_hash)
        .map_err(|e| ShellError::Io(format!("failed to parse commit hash '{full_hash}': {e}")))?;

    let parent_hashes: Vec<CommitHash> = if parent_line.is_empty() {
        Vec::new()
    } else {
        parent_line
            .split(' ')
            .map(|h| {
                CommitHash::try_new(h.trim())
                    .map_err(|e| ShellError::Io(format!("failed to parse parent hash '{h}': {e}")))
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(CommitInfo::new(
        hash,
        short_hash.to_string(),
        author_name.to_string(),
        author_email.to_string(),
        date.to_string(),
        subject.to_string(),
        parent_hashes,
    ))
}

/// Parse output of `git branch -vv --format=%(HEAD)%(refname:short)\t%(upstream:short)\t%(upstream:track,nobracket)`
///
/// Each line has the format:
///   `*main\torigin/main\tahead 2, behind 1`   (current branch with upstream + tracking)
///   ` feature\torigin/feature\t`               (non-current, upstream but no divergence)
///   ` local-only\t\t`                          (no upstream)
///
/// The leading `*` means the branch is the current HEAD branch, space otherwise.
fn parse_branch_list(output: &str) -> Result<Vec<BranchInfo>, ShellError> {
    let mut branches = Vec::new();

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        let is_current = line.starts_with('*');
        // Strip the HEAD indicator character
        let rest = &line[1..];

        let parts: Vec<&str> = rest.splitn(3, '\t').collect();
        let name_str = parts.first().copied().unwrap_or("").trim();
        if name_str.is_empty() {
            continue;
        }

        // Skip detached HEAD entries like "(HEAD detached at ...)"
        if name_str.starts_with('(') {
            continue;
        }

        let upstream_str = parts.get(1).copied().unwrap_or("").trim();
        let tracking_str = parts.get(2).copied().unwrap_or("").trim();

        let upstream = if upstream_str.is_empty() {
            None
        } else {
            Some(upstream_str.to_string())
        };

        let (ahead, behind) = parse_tracking_info(tracking_str);

        let name = BranchName::try_new(name_str).map_err(|e| {
            ShellError::Io(format!("failed to parse branch name '{name_str}': {e}"))
        })?;

        branches.push(BranchInfo::new(name, is_current, upstream, ahead, behind));
    }

    Ok(branches)
}

/// Parse the tracking info string from `%(upstream:track,nobracket)`.
///
/// Examples: `"ahead 2, behind 1"`, `"ahead 3"`, `"behind 5"`, `"gone"`, `""`.
fn parse_tracking_info(info: &str) -> (u32, u32) {
    if info.is_empty() || info == "gone" {
        return (0, 0);
    }

    let mut ahead: u32 = 0;
    let mut behind: u32 = 0;

    for part in info.split(", ") {
        let part = part.trim();
        if let Some(n) = part.strip_prefix("ahead ") {
            ahead = n.trim().parse().unwrap_or(0);
        } else if let Some(n) = part.strip_prefix("behind ") {
            behind = n.trim().parse().unwrap_or(0);
        }
    }

    (ahead, behind)
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

    fn log(&self, repo_path: &Path, max_count: u32) -> Result<Vec<CommitInfo>, ShellError> {
        // Custom format: fields separated by record-separator (0x1e), commits by group-separator (0x1d)
        let format = "%H%x1e%h%x1e%an%x1e%ae%x1e%aI%x1e%s%x1e%P%x1d";
        let output = self.run_git(
            repo_path,
            &[
                "log",
                &format!("--format={format}"),
                &format!("-n{max_count}"),
            ],
        )?;
        parse_log_output(&output)
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

// ---------------------------------------------------------------------------
// Log output parser
// ---------------------------------------------------------------------------

/// Parse git log output produced with `--format=%H%x1e%h%x1e%an%x1e%ae%x1e%aI%x1e%s%x1e%P%x1d`.
///
/// Commits are separated by group-separator (0x1d), fields within a commit by
/// record-separator (0x1e).
fn parse_log_output(output: &str) -> Result<Vec<CommitInfo>, ShellError> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut commits = Vec::new();
    for record in trimmed.split('\x1d') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }
        let fields: Vec<&str> = record.split('\x1e').collect();
        if fields.len() < 6 {
            return Err(ShellError::Io(format!(
                "unexpected log record (expected 7 fields, got {}): {record}",
                fields.len()
            )));
        }

        let hash = CommitHash::try_new(fields[0].trim())
            .map_err(|e| ShellError::Io(format!("invalid commit hash in log: {e}")))?;

        let parent_hashes_str = if fields.len() > 6 {
            fields[6].trim()
        } else {
            ""
        };
        let parent_hashes: Vec<CommitHash> = if parent_hashes_str.is_empty() {
            Vec::new()
        } else {
            parent_hashes_str
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(|h| {
                    CommitHash::try_new(h)
                        .map_err(|e| ShellError::Io(format!("invalid parent hash in log: {e}")))
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        commits.push(CommitInfo::new(
            hash,
            fields[1].trim().to_string(),
            fields[2].trim().to_string(),
            fields[3].trim().to_string(),
            fields[4].trim().to_string(),
            fields[5].trim().to_string(),
            parent_hashes,
        ));
    }

    Ok(commits)
}

// ---------------------------------------------------------------------------
// Blame porcelain parser
// ---------------------------------------------------------------------------

/// Parse `git blame --porcelain` output into `BlameEntry` blocks.
///
/// Porcelain format:
/// ```text
/// <hash> <orig_line> <final_line> <num_lines>
/// author <name>
/// author-time <epoch>
/// ...
/// \t<content line>
/// ```
/// Lines for the same commit are grouped into contiguous `BlameEntry` blocks.
fn parse_blame_porcelain(output: &str) -> Result<Vec<BlameEntry>, ShellError> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // First pass: collect per-line blame info
    struct LineBlame {
        hash: String,
        author: String,
        date: String,
        final_line: u32,
        content: String,
    }

    let mut lines_info: Vec<LineBlame> = Vec::new();
    let mut current_hash = String::new();
    let mut current_author = String::new();
    let mut current_date = String::new();
    let mut current_final_line: u32 = 0;

    for line in trimmed.lines() {
        if let Some(stripped) = line.strip_prefix('\t') {
            // Content line — finalize this blamed line
            lines_info.push(LineBlame {
                hash: current_hash.clone(),
                author: current_author.clone(),
                date: current_date.clone(),
                final_line: current_final_line,
                content: stripped.to_string(),
            });
        } else if let Some(rest) = line.strip_prefix("author ") {
            current_author = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("author-time ") {
            current_date = rest.to_string();
        } else {
            // Could be a header line: <hash> <orig_line> <final_line> [<num_lines>]
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                // Check if first token looks like a hex hash (at least 4 hex chars)
                let maybe_hash = parts[0];
                if maybe_hash.len() >= 4 && maybe_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    current_hash = maybe_hash.to_lowercase();
                    current_final_line = parts[2].parse::<u32>().unwrap_or(0);
                }
            }
        }
    }

    // Second pass: group contiguous lines by commit hash
    let mut entries: Vec<BlameEntry> = Vec::new();
    for line_info in &lines_info {
        let can_merge = entries.last().is_some_and(|last: &BlameEntry| {
            last.hash().as_ref() == line_info.hash
                && last.line_start() + last.line_count() == line_info.final_line
        });

        if can_merge {
            // Merge into last entry: increment line_count, append content
            let last = entries.last().expect("checked above");
            let merged_content = if last.content().is_empty() {
                line_info.content.clone()
            } else {
                format!("{}\n{}", last.content(), line_info.content)
            };
            let merged = BlameEntry::new(
                CommitHash::try_new(&line_info.hash)
                    .map_err(|e| ShellError::Io(format!("invalid blame hash: {e}")))?,
                last.author().to_string(),
                last.date().to_string(),
                last.line_start(),
                last.line_count() + 1,
                merged_content,
            );
            let len = entries.len();
            entries[len - 1] = merged;
        } else {
            let hash = CommitHash::try_new(&line_info.hash)
                .map_err(|e| ShellError::Io(format!("invalid blame hash: {e}")))?;
            entries.push(BlameEntry::new(
                hash,
                line_info.author.clone(),
                line_info.date.clone(),
                line_info.final_line,
                1,
                line_info.content.clone(),
            ));
        }
    }

    Ok(entries)
}

// ---------------------------------------------------------------------------
// Stash list parser
// ---------------------------------------------------------------------------

/// Parse `git stash list --format=%gd%x1e%gs%x1e%aI` output.
///
/// Each line: `stash@{N}<RS>message<RS>date`
fn parse_stash_list_output(output: &str) -> Result<Vec<StashEntry>, ShellError> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\x1e').collect();
        if fields.len() < 3 {
            return Err(ShellError::Io(format!(
                "unexpected stash list format (expected 3 fields, got {}): {line}",
                fields.len()
            )));
        }

        // Parse index from "stash@{N}"
        let gd = fields[0].trim();
        let index: usize = gd
            .strip_prefix("stash@{")
            .and_then(|s| s.strip_suffix('}'))
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| ShellError::Io(format!("invalid stash ref format: {gd}")))?;

        entries.push(StashEntry::new(
            StashId::new(index),
            fields[1].trim().to_string(),
            fields[2].trim().to_string(),
        ));
    }

    Ok(entries)
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
