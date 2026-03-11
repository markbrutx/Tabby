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

#[cfg(test)]
mod tests {
    use super::*;
    use tabby_git::FileStatusKind;

    // -----------------------------------------------------------------------
    // status_char_to_kind
    // -----------------------------------------------------------------------

    #[test]
    fn status_char_m_is_modified() {
        assert_eq!(status_char_to_kind('M'), FileStatusKind::Modified);
    }

    #[test]
    fn status_char_t_is_modified() {
        // T = type-changed, treated as Modified
        assert_eq!(status_char_to_kind('T'), FileStatusKind::Modified);
    }

    #[test]
    fn status_char_a_is_added() {
        assert_eq!(status_char_to_kind('A'), FileStatusKind::Added);
    }

    #[test]
    fn status_char_d_is_deleted() {
        assert_eq!(status_char_to_kind('D'), FileStatusKind::Deleted);
    }

    #[test]
    fn status_char_r_is_renamed() {
        assert_eq!(status_char_to_kind('R'), FileStatusKind::Renamed);
    }

    #[test]
    fn status_char_c_is_copied() {
        assert_eq!(status_char_to_kind('C'), FileStatusKind::Copied);
    }

    #[test]
    fn status_char_u_is_conflicted() {
        assert_eq!(status_char_to_kind('U'), FileStatusKind::Conflicted);
    }

    #[test]
    fn status_char_dot_is_modified_fallback() {
        // '.' = unmodified slot, but when present means the other column has a change
        assert_eq!(status_char_to_kind('.'), FileStatusKind::Modified);
    }

    #[test]
    fn status_char_unknown_is_modified_fallback() {
        assert_eq!(status_char_to_kind('X'), FileStatusKind::Modified);
        assert_eq!(status_char_to_kind('?'), FileStatusKind::Modified);
    }

    // -----------------------------------------------------------------------
    // parse_porcelain_v2 — ordinary changed entries (prefix "1")
    // -----------------------------------------------------------------------

    #[test]
    fn empty_output_returns_empty_vec() {
        let result = parse_porcelain_v2("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn whitespace_only_returns_empty_vec() {
        let result = parse_porcelain_v2("   \n\n").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn header_lines_are_skipped() {
        let output = "# branch.oid abc123def456\n# branch.head main\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn ordinary_modified_worktree() {
        // "1 .M ..." means worktree modified, index unmodified
        let output = "1 .M N... 100644 100644 100644 abc1234 def5678 src/main.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "src/main.rs");
        assert_eq!(result[0].worktree_status(), FileStatusKind::Modified);
    }

    #[test]
    fn ordinary_modified_index() {
        // "1 M. ..." means index modified, worktree unmodified
        let output = "1 M. N... 100644 100644 100644 abc1234 def5678 src/lib.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result[0].index_status(), FileStatusKind::Modified);
    }

    #[test]
    fn ordinary_added_in_index() {
        let output = "1 A. N... 000000 100644 100644 0000000 abc1234 src/new.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result[0].index_status(), FileStatusKind::Added);
        assert_eq!(result[0].path(), "src/new.rs");
    }

    #[test]
    fn ordinary_deleted_in_index() {
        let output = "1 D. N... 100644 000000 000000 abc1234 0000000 src/old.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result[0].index_status(), FileStatusKind::Deleted);
    }

    #[test]
    fn ordinary_type_changed_treated_as_modified() {
        let output = "1 T. N... 100644 100644 100644 abc1234 def5678 symlink.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result[0].index_status(), FileStatusKind::Modified);
    }

    #[test]
    fn ordinary_entry_no_old_path() {
        let output = "1 MM N... 100644 100644 100644 abc1234 def5678 src/both.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result[0].old_path(), None);
    }

    #[test]
    fn ordinary_entry_malformed_too_few_fields_is_error() {
        // Missing the path field (only 8 fields)
        let output = "1 .M N... 100644 100644 100644 abc1234 def5678\n";
        let result = parse_porcelain_v2(output);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // parse_porcelain_v2 — renamed/copied entries (prefix "2")
    // -----------------------------------------------------------------------

    #[test]
    fn renamed_entry_parsed_correctly() {
        let output =
            "2 R. N... 100644 100644 100644 abc1234 def5678 R100 new_name.rs\told_name.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "new_name.rs");
        assert_eq!(result[0].old_path(), Some("old_name.rs"));
        assert_eq!(result[0].index_status(), FileStatusKind::Renamed);
    }

    #[test]
    fn copied_entry_parsed_correctly() {
        let output =
            "2 C. N... 100644 100644 100644 abc1234 def5678 C100 copy.rs\toriginal.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result[0].index_status(), FileStatusKind::Copied);
        assert_eq!(result[0].old_path(), Some("original.rs"));
    }

    #[test]
    fn renamed_entry_with_path_spaces() {
        let output = "2 R. N... 100644 100644 100644 abc1234 def5678 R100 new path/file.rs\told path/file.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result[0].path(), "new path/file.rs");
        assert_eq!(result[0].old_path(), Some("old path/file.rs"));
    }

    #[test]
    fn renamed_entry_malformed_too_few_fields_is_error() {
        // Only 9 fields (need 10)
        let output = "2 R. N... 100644 100644 100644 abc1234 def5678 R100\n";
        let result = parse_porcelain_v2(output);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // parse_porcelain_v2 — unmerged entries (prefix "u")
    // -----------------------------------------------------------------------

    #[test]
    fn unmerged_entry_is_conflicted() {
        let output = "u UU N... 100644 100644 100644 100644 abc1234 def5678 ghi9012 merge_conflict.rs\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index_status(), FileStatusKind::Conflicted);
        assert_eq!(result[0].worktree_status(), FileStatusKind::Conflicted);
        assert_eq!(result[0].path(), "merge_conflict.rs");
    }

    #[test]
    fn unmerged_entry_malformed_too_few_fields_is_error() {
        let output = "u UU N... 100644 100644\n";
        let result = parse_porcelain_v2(output);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // parse_porcelain_v2 — untracked and ignored
    // -----------------------------------------------------------------------

    #[test]
    fn untracked_entry() {
        let output = "? untracked_file.txt\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "untracked_file.txt");
        assert_eq!(result[0].index_status(), FileStatusKind::Untracked);
        assert_eq!(result[0].worktree_status(), FileStatusKind::Untracked);
    }

    #[test]
    fn untracked_entry_with_spaces_in_path() {
        let output = "? path with spaces/file.txt\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result[0].path(), "path with spaces/file.txt");
    }

    #[test]
    fn ignored_entry() {
        let output = "! ignored.log\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path(), "ignored.log");
        assert_eq!(result[0].index_status(), FileStatusKind::Ignored);
        assert_eq!(result[0].worktree_status(), FileStatusKind::Ignored);
    }

    // -----------------------------------------------------------------------
    // parse_porcelain_v2 — mixed status output
    // -----------------------------------------------------------------------

    #[test]
    fn mixed_status_output_counts_all_entries() {
        let output = "\
# branch.oid abc123def456
# branch.head main
1 .M N... 100644 100644 100644 abc1234 def5678 src/lib.rs
1 A. N... 000000 100644 100644 0000000 abc1234 src/new.rs
2 R. N... 100644 100644 100644 abc1234 def5678 R100 renamed.rs\told.rs
? untracked.txt
u UU N... 100644 100644 100644 100644 abc1234 def5678 ghi9012 conflict.rs
! ignored.log
";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn mixed_status_output_correct_kinds() {
        let output = "\
1 .M N... 100644 100644 100644 abc1234 def5678 modified.rs
1 A. N... 000000 100644 100644 0000000 abc1234 added.rs
1 D. N... 100644 000000 000000 abc1234 0000000 deleted.rs
";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result[0].worktree_status(), FileStatusKind::Modified);
        assert_eq!(result[1].index_status(), FileStatusKind::Added);
        assert_eq!(result[2].index_status(), FileStatusKind::Deleted);
    }

    #[test]
    fn empty_lines_in_output_are_skipped() {
        let output = "\n\n1 .M N... 100644 100644 100644 abc1234 def5678 src/file.rs\n\n";
        let result = parse_porcelain_v2(output).unwrap();
        assert_eq!(result.len(), 1);
    }

    // -----------------------------------------------------------------------
    // parse_commit_show_output
    // -----------------------------------------------------------------------

    #[test]
    fn commit_show_basic_parse() {
        let show_output = "\
abc123def456abc123def456abc123def456abc1
abc123d
Alice
alice@example.com
2026-03-10T01:00:00+00:00
1111111111111111111111111111111111111111
feat: add new feature
";
        let result = parse_commit_show_output(show_output, "").unwrap();
        assert_eq!(result.short_hash(), "abc123d");
        assert_eq!(result.author_name(), "Alice");
        assert_eq!(result.author_email(), "alice@example.com");
        assert_eq!(result.date(), "2026-03-10T01:00:00+00:00");
        assert_eq!(result.message(), "feat: add new feature");
        assert_eq!(result.parent_hashes().len(), 1);
    }

    #[test]
    fn commit_show_no_parents_is_root_commit() {
        let show_output = "\
abc123def456abc123def456abc123def456abc1
abc123d
Alice
alice@example.com
2026-03-10T01:00:00+00:00

initial commit
";
        let result = parse_commit_show_output(show_output, "").unwrap();
        assert!(result.parent_hashes().is_empty());
    }

    #[test]
    fn commit_show_merge_commit_has_two_parents() {
        let show_output = "\
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaa
Merger
merger@example.com
2026-03-10T01:00:00+00:00
bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb cccccccccccccccccccccccccccccccccccccccc
Merge branch 'feature'
";
        let result = parse_commit_show_output(show_output, "").unwrap();
        assert_eq!(result.parent_hashes().len(), 2);
    }

    #[test]
    fn commit_show_too_few_lines_is_error() {
        let result = parse_commit_show_output("abc123\nshort\n", "");
        assert!(result.is_err());
    }

    #[test]
    fn commit_show_invalid_hash_is_error() {
        let show_output = "\
not-a-valid-hash!!!
short
Author
email@e.com
2026-01-01T00:00:00+00:00

message
";
        let result = parse_commit_show_output(show_output, "");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // parse_branch_list
    // -----------------------------------------------------------------------

    #[test]
    fn branch_list_single_current_branch() {
        let output = "*main\torigin/main\t\n";
        let result = parse_branch_list(output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name().as_ref(), "main");
        assert!(result[0].is_current());
    }

    #[test]
    fn branch_list_non_current_branch() {
        let output = " feature-branch\torigin/feature-branch\t\n";
        let result = parse_branch_list(output).unwrap();
        assert!(!result[0].is_current());
        assert_eq!(result[0].name().as_ref(), "feature-branch");
    }

    #[test]
    fn branch_list_with_upstream() {
        let output = "*main\torigin/main\t\n";
        let result = parse_branch_list(output).unwrap();
        assert_eq!(result[0].upstream(), Some("origin/main"));
    }

    #[test]
    fn branch_list_no_upstream() {
        let output = " local-only\t\t\n";
        let result = parse_branch_list(output).unwrap();
        assert_eq!(result[0].upstream(), None);
    }

    #[test]
    fn branch_list_ahead_and_behind() {
        let output = "*main\torigin/main\tahead 2, behind 1\n";
        let result = parse_branch_list(output).unwrap();
        assert_eq!(result[0].ahead(), 2);
        assert_eq!(result[0].behind(), 1);
    }

    #[test]
    fn branch_list_skips_detached_head() {
        let output = "*(HEAD detached at abc1234)\t\t\n main\torigin/main\t\n";
        let result = parse_branch_list(output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name().as_ref(), "main");
    }

    #[test]
    fn branch_list_multiple_branches() {
        let output = "*main\torigin/main\t\n dev\torigin/dev\tahead 1\n feature\t\t\n";
        let result = parse_branch_list(output).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn branch_list_empty_output() {
        let result = parse_branch_list("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn branch_list_gone_upstream_has_zero_ahead_behind() {
        let output = " old-branch\torigin/old-branch\tgone\n";
        let result = parse_branch_list(output).unwrap();
        assert_eq!(result[0].ahead(), 0);
        assert_eq!(result[0].behind(), 0);
    }

    // -----------------------------------------------------------------------
    // parse_tracking_info
    // -----------------------------------------------------------------------

    #[test]
    fn tracking_info_empty_string() {
        assert_eq!(parse_tracking_info(""), (0, 0));
    }

    #[test]
    fn tracking_info_gone() {
        assert_eq!(parse_tracking_info("gone"), (0, 0));
    }

    #[test]
    fn tracking_info_ahead_only() {
        assert_eq!(parse_tracking_info("ahead 3"), (3, 0));
    }

    #[test]
    fn tracking_info_behind_only() {
        assert_eq!(parse_tracking_info("behind 5"), (0, 5));
    }

    #[test]
    fn tracking_info_ahead_and_behind() {
        assert_eq!(parse_tracking_info("ahead 2, behind 1"), (2, 1));
    }

    #[test]
    fn tracking_info_large_numbers() {
        assert_eq!(parse_tracking_info("ahead 100, behind 200"), (100, 200));
    }

    #[test]
    fn tracking_info_malformed_is_zero() {
        // Non-numeric values should produce zero
        assert_eq!(parse_tracking_info("ahead xyz"), (0, 0));
        assert_eq!(parse_tracking_info("behind abc"), (0, 0));
    }

    // -----------------------------------------------------------------------
    // parse_log_output
    // -----------------------------------------------------------------------

    #[test]
    fn log_output_empty_returns_empty() {
        let result = parse_log_output("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn log_output_single_commit() {
        // Format: hash\x1eshort\x1eauthor\x1eemail\x1edate\x1esubject\x1eparent\x1d
        let output = "abc123def456abc123def456abc123def456abc1\x1eabc123d\x1eAlice\x1ealice@example.com\x1e2026-03-10T01:00:00+00:00\x1efeat: initial commit\x1e\x1d";
        let result = parse_log_output(output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].author_name(), "Alice");
        assert_eq!(result[0].message(), "feat: initial commit");
        assert!(result[0].parent_hashes().is_empty());
    }

    #[test]
    fn log_output_single_commit_with_parent() {
        let parent = "1111111111111111111111111111111111111111";
        let output = format!("abc123def456abc123def456abc123def456abc1\x1eabc123d\x1eAlice\x1ealice@example.com\x1e2026-03-10T01:00:00+00:00\x1efeat: add stuff\x1e{parent}\x1d");
        let result = parse_log_output(&output).unwrap();
        assert_eq!(result[0].parent_hashes().len(), 1);
    }

    #[test]
    fn log_output_multiple_commits() {
        let output = "\
abc123def456abc123def456abc123def456abc1\x1eabc123d\x1eAlice\x1ealice@example.com\x1e2026-03-10T01:00:00+00:00\x1efeat: second\x1e1111111111111111111111111111111111111111\x1d\
1111111111111111111111111111111111111111\x1e1111111\x1eBob\x1ebob@example.com\x1e2026-03-09T01:00:00+00:00\x1efeat: first\x1e\x1d";
        let result = parse_log_output(output).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].author_name(), "Alice");
        assert_eq!(result[1].author_name(), "Bob");
    }

    #[test]
    fn log_output_too_few_fields_is_error() {
        // Only 3 fields, need at least 6
        let output = "abc1234\x1eshort\x1eauthor\x1d";
        let result = parse_log_output(output);
        assert!(result.is_err());
    }

    #[test]
    fn log_output_invalid_hash_is_error() {
        let output = "not-hex!\x1eshort\x1eAuthor\x1eemail\x1edate\x1esubject\x1d";
        let result = parse_log_output(output);
        assert!(result.is_err());
    }
}
