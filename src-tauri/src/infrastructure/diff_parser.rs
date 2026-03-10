use tabby_git::{DiffContent, DiffHunk, DiffLine, DiffLineKind};

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
pub(super) fn parse_unified_diff(output: &str) -> Vec<DiffContent> {
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
pub(super) fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32)> {
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
pub(super) fn filter_diff_to_line_ranges(diff_output: &str, line_ranges: &[(u32, u32)]) -> String {
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
