use tabby_git::value_objects::{BranchName, CommitHash};
use tabby_git::{BranchInfo, CommitInfo, FileStatus, FileStatusKind};

use crate::shell::error::ShellError;

/// Map a single porcelain v2 XY status character to a `FileStatusKind`.
///
/// Git porcelain v2 uses these codes for the index (X) and worktree (Y) columns:
///   . = unmodified, M = modified, T = type-changed (treated as Modified),
///   A = added, D = deleted, R = renamed, C = copied, U = unmerged
pub(super) fn status_char_to_kind(ch: char) -> FileStatusKind {
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
pub(super) fn parse_porcelain_v2(output: &str) -> Result<Vec<FileStatus>, ShellError> {
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

/// Parse the output of `git show -s --format=%H%n%h%n%an%n%ae%n%aI%n%P%n%s HEAD`
/// into a `CommitInfo`.
pub(super) fn parse_commit_show_output(
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
pub(super) fn parse_branch_list(output: &str) -> Result<Vec<BranchInfo>, ShellError> {
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
pub(super) fn parse_tracking_info(info: &str) -> (u32, u32) {
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

/// Parse git log output produced with `--format=%H%x1e%h%x1e%an%x1e%ae%x1e%aI%x1e%s%x1e%P%x1d`.
///
/// Commits are separated by group-separator (0x1d), fields within a commit by
/// record-separator (0x1e).
pub(super) fn parse_log_output(output: &str) -> Result<Vec<CommitInfo>, ShellError> {
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
