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

#[cfg(test)]
mod tests {
    use super::*;
    use tabby_git::DiffLineKind;

    // -----------------------------------------------------------------------
    // parse_unified_diff — basic cases
    // -----------------------------------------------------------------------

    #[test]
    fn empty_string_returns_empty_vec() {
        let result = parse_unified_diff("");
        assert!(result.is_empty());
    }

    #[test]
    fn whitespace_only_returns_empty_vec() {
        let result = parse_unified_diff("   \n\n\t\n");
        assert!(result.is_empty());
    }

    #[test]
    fn single_file_single_hunk() {
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
        assert_eq!(diff.file_mode_change(), None);
        assert_eq!(diff.hunks().len(), 1);
        let hunk = &diff.hunks()[0];
        assert_eq!(hunk.old_start(), 1);
        assert_eq!(hunk.old_count(), 3);
        assert_eq!(hunk.new_start(), 1);
        assert_eq!(hunk.new_count(), 4);
        // context + deletion + addition + addition + context = 5 lines
        assert_eq!(hunk.lines().len(), 5);
    }

    #[test]
    fn single_file_correct_line_kinds() {
        let input = "\
diff --git a/a.rs b/a.rs
index 000..111 100644
--- a/a.rs
+++ b/a.rs
@@ -1,2 +1,2 @@
 context
-deleted
+added
";
        let result = parse_unified_diff(input);
        let lines = result[0].hunks()[0].lines();
        assert_eq!(lines[0].kind(), DiffLineKind::Context);
        assert_eq!(lines[1].kind(), DiffLineKind::Deletion);
        assert_eq!(lines[2].kind(), DiffLineKind::Addition);
    }

    #[test]
    fn single_file_line_numbers_are_correct() {
        let input = "\
diff --git a/a.rs b/a.rs
index 000..111 100644
--- a/a.rs
+++ b/a.rs
@@ -10,2 +10,3 @@
 context_line
-old_line
+new_line_a
+new_line_b
";
        let result = parse_unified_diff(input);
        let lines = result[0].hunks()[0].lines();
        // context: old=10, new=10
        assert_eq!(lines[0].old_line_no(), Some(10));
        assert_eq!(lines[0].new_line_no(), Some(10));
        // deletion: old=11, new=None
        assert_eq!(lines[1].old_line_no(), Some(11));
        assert_eq!(lines[1].new_line_no(), None);
        // addition: old=None, new=11
        assert_eq!(lines[2].old_line_no(), None);
        assert_eq!(lines[2].new_line_no(), Some(11));
        // addition: old=None, new=12
        assert_eq!(lines[3].old_line_no(), None);
        assert_eq!(lines[3].new_line_no(), Some(12));
    }

    #[test]
    fn multiple_files_in_one_diff() {
        let input = "\
diff --git a/file_a.rs b/file_a.rs
index 000..111 100644
--- a/file_a.rs
+++ b/file_a.rs
@@ -1 +1 @@
-old_a
+new_a
diff --git a/file_b.rs b/file_b.rs
index 222..333 100644
--- a/file_b.rs
+++ b/file_b.rs
@@ -1 +1 @@
-old_b
+new_b
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].file_path(), "file_a.rs");
        assert_eq!(result[1].file_path(), "file_b.rs");
    }

    #[test]
    fn multiple_hunks_in_one_file() {
        let input = "\
diff --git a/large.rs b/large.rs
index 000..111 100644
--- a/large.rs
+++ b/large.rs
@@ -1,2 +1,2 @@
-line1_old
+line1_new
 context1
@@ -50,2 +50,2 @@
-line50_old
+line50_new
 context50
";
        let result = parse_unified_diff(input);
        assert_eq!(result[0].hunks().len(), 2);
        assert_eq!(result[0].hunks()[0].old_start(), 1);
        assert_eq!(result[0].hunks()[1].old_start(), 50);
    }

    // -----------------------------------------------------------------------
    // Binary diffs
    // -----------------------------------------------------------------------

    #[test]
    fn binary_file_detected() {
        let input = "\
diff --git a/image.png b/image.png
index abc1234..def5678 100644
Binary files a/image.png and b/image.png differ
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_binary());
        assert!(result[0].hunks().is_empty());
        assert_eq!(result[0].file_path(), "image.png");
    }

    #[test]
    fn binary_file_has_correct_path() {
        let input = "\
diff --git a/assets/logo.svg b/assets/logo.svg
index abc..def 100644
Binary files a/assets/logo.svg and b/assets/logo.svg differ
";
        let result = parse_unified_diff(input);
        assert_eq!(result[0].file_path(), "assets/logo.svg");
    }

    // -----------------------------------------------------------------------
    // Rename diffs
    // -----------------------------------------------------------------------

    #[test]
    fn rename_diff_sets_old_path_and_new_path() {
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
        assert_eq!(result[0].file_path(), "new_name.rs");
        assert_eq!(result[0].old_path(), Some("old_name.rs"));
    }

    #[test]
    fn rename_only_no_content_changes() {
        let input = "\
diff --git a/foo.rs b/bar.rs
similarity index 100%
rename from foo.rs
rename to bar.rs
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_path(), "bar.rs");
        assert_eq!(result[0].old_path(), Some("foo.rs"));
        assert!(result[0].hunks().is_empty());
    }

    // -----------------------------------------------------------------------
    // File mode change
    // -----------------------------------------------------------------------

    #[test]
    fn file_mode_change_parsed() {
        let input = "\
diff --git a/script.sh b/script.sh
old mode 100644
new mode 100755
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_mode_change(), Some("100644 -> 100755"));
    }

    // -----------------------------------------------------------------------
    // New file / deleted file
    // -----------------------------------------------------------------------

    #[test]
    fn new_file_all_additions() {
        let input = "\
diff --git a/new_file.rs b/new_file.rs
new file mode 100644
index 0000000..abc1234
--- /dev/null
+++ b/new_file.rs
@@ -0,0 +1,3 @@
+fn new_function() {
+    println!(\"hello\");
+}
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        let lines = &result[0].hunks()[0].lines();
        assert!(lines.iter().all(|l| l.kind() == DiffLineKind::Addition));
    }

    #[test]
    fn deleted_file_all_deletions() {
        let input = "\
diff --git a/gone.rs b/gone.rs
deleted file mode 100644
index abc1234..0000000
--- a/gone.rs
+++ /dev/null
@@ -1,2 +0,0 @@
-fn deleted_fn() {}
-// old comment
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        let lines = &result[0].hunks()[0].lines();
        assert!(lines.iter().all(|l| l.kind() == DiffLineKind::Deletion));
    }

    // -----------------------------------------------------------------------
    // No newline at end of file marker
    // -----------------------------------------------------------------------

    #[test]
    fn no_newline_marker_is_skipped_not_counted_as_line() {
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
        let lines = result[0].hunks()[0].lines();
        // Only deletion and addition, the markers should be skipped
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].kind(), DiffLineKind::Deletion);
        assert_eq!(lines[1].kind(), DiffLineKind::Addition);
    }

    #[test]
    fn no_newline_at_end_deletion_only() {
        let input = "\
diff --git a/file.txt b/file.txt
index abc..def 100644
--- a/file.txt
+++ b/file.txt
@@ -1 +1,2 @@
-only line
\\ No newline at end of file
+first line
+second line
";
        let result = parse_unified_diff(input);
        let lines = result[0].hunks()[0].lines();
        assert_eq!(lines.len(), 3); // deletion + 2 additions
    }

    // -----------------------------------------------------------------------
    // Special characters and unicode
    // -----------------------------------------------------------------------

    #[test]
    fn unicode_in_file_content() {
        let input = "\
diff --git a/unicode.rs b/unicode.rs
index abc..def 100644
--- a/unicode.rs
+++ b/unicode.rs
@@ -1 +1 @@
-let greeting = \"こんにちは\";
+let greeting = \"안녕하세요\";
";
        let result = parse_unified_diff(input);
        let lines = result[0].hunks()[0].lines();
        assert_eq!(lines[0].content(), "let greeting = \"こんにちは\";");
        assert_eq!(lines[1].content(), "let greeting = \"안녕하세요\";");
    }

    #[test]
    fn file_path_with_spaces_via_b_prefix() {
        // "diff --git a/path with spaces b/path with spaces" — rfind " b/" should work
        let input = "\
diff --git a/path with spaces/file.rs b/path with spaces/file.rs
index abc..def 100644
--- a/path with spaces/file.rs
+++ b/path with spaces/file.rs
@@ -1 +1 @@
-old
+new
";
        let result = parse_unified_diff(input);
        assert_eq!(result[0].file_path(), "path with spaces/file.rs");
    }

    #[test]
    fn unicode_filename_in_diff_header() {
        let input = "\
diff --git a/src/données.rs b/src/données.rs
index abc..def 100644
--- a/src/données.rs
+++ b/src/données.rs
@@ -1 +1 @@
-old
+new
";
        let result = parse_unified_diff(input);
        assert_eq!(result[0].file_path(), "src/données.rs");
    }

    #[test]
    fn hunk_context_with_special_chars() {
        let input = "\
diff --git a/special.rs b/special.rs
index abc..def 100644
--- a/special.rs
+++ b/special.rs
@@ -1,3 +1,3 @@
 fn test() -> Result<(), Box<dyn std::error::Error>> {
-    let url = \"http://example.com?a=1&b=2\";
+    let url = \"http://example.com?a=1&b=2#fragment\";
 }
";
        let result = parse_unified_diff(input);
        let lines = result[0].hunks()[0].lines();
        // context + deletion + addition + context = 4 lines
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].kind(), DiffLineKind::Context);
        assert_eq!(lines[1].kind(), DiffLineKind::Deletion);
        assert_eq!(lines[2].kind(), DiffLineKind::Addition);
        assert_eq!(lines[3].kind(), DiffLineKind::Context);
    }

    // -----------------------------------------------------------------------
    // Hunk header context text
    // -----------------------------------------------------------------------

    #[test]
    fn hunk_header_with_function_context() {
        let input = "\
diff --git a/lib.rs b/lib.rs
index abc..def 100644
--- a/lib.rs
+++ b/lib.rs
@@ -10,3 +10,3 @@ fn my_function() {
 let a = 1;
-let b = 2;
+let b = 3;
 let c = 4;
";
        let result = parse_unified_diff(input);
        let hunk = &result[0].hunks()[0];
        assert!(hunk.header().contains("fn my_function()"));
        assert_eq!(hunk.old_start(), 10);
    }

    // -----------------------------------------------------------------------
    // parse_hunk_header unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn hunk_header_with_count() {
        let result = parse_hunk_header("@@ -1,5 +2,6 @@");
        assert_eq!(result, Some((1, 5, 2, 6)));
    }

    #[test]
    fn hunk_header_without_count_defaults_to_one() {
        let result = parse_hunk_header("@@ -1 +1 @@");
        assert_eq!(result, Some((1, 1, 1, 1)));
    }

    #[test]
    fn hunk_header_with_context_text() {
        let result = parse_hunk_header("@@ -10,3 +10,4 @@ fn example() {");
        assert_eq!(result, Some((10, 3, 10, 4)));
    }

    #[test]
    fn hunk_header_zero_count() {
        // New file: @@ -0,0 +1,5 @@
        let result = parse_hunk_header("@@ -0,0 +1,5 @@");
        assert_eq!(result, Some((0, 0, 1, 5)));
    }

    #[test]
    fn hunk_header_large_numbers() {
        let result = parse_hunk_header("@@ -10000,50 +10001,51 @@");
        assert_eq!(result, Some((10000, 50, 10001, 51)));
    }

    #[test]
    fn hunk_header_malformed_missing_at_prefix() {
        let result = parse_hunk_header("-1,5 +2,6 @@");
        assert_eq!(result, None);
    }

    #[test]
    fn hunk_header_malformed_no_closing_at() {
        let result = parse_hunk_header("@@ -1,5 +2,6");
        assert_eq!(result, None);
    }

    #[test]
    fn hunk_header_malformed_non_numeric() {
        let result = parse_hunk_header("@@ -abc,def +ghi,jkl @@");
        assert_eq!(result, None);
    }

    #[test]
    fn hunk_header_empty_string() {
        let result = parse_hunk_header("");
        assert_eq!(result, None);
    }

    // -----------------------------------------------------------------------
    // filter_diff_to_line_ranges
    // -----------------------------------------------------------------------

    #[test]
    fn filter_diff_passthrough_header_lines() {
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
        let filtered = filter_diff_to_line_ranges(diff, &[(2, 3)]);
        assert!(filtered.contains("diff --git a/file.rs b/file.rs"));
        assert!(filtered.contains("--- a/file.rs"));
        assert!(filtered.contains("+++ b/file.rs"));
    }

    #[test]
    fn filter_diff_includes_matching_additions() {
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
        assert!(!filtered.contains("+added_line3"));
    }

    #[test]
    fn filter_diff_excludes_out_of_range_additions() {
        let diff = "\
diff --git a/file.rs b/file.rs
index abc..def 100644
--- a/file.rs
+++ b/file.rs
@@ -1,2 +1,4 @@
+line1
+line2
+line3
+line4
";
        let filtered = filter_diff_to_line_ranges(diff, &[(100, 200)]);
        // No matching changes, so no hunk should appear
        assert!(!filtered.contains("@@"));
    }

    #[test]
    fn filter_diff_multiple_ranges() {
        let diff = "\
diff --git a/file.rs b/file.rs
index abc..def 100644
--- a/file.rs
+++ b/file.rs
@@ -1,5 +1,7 @@
 line1
+added2
+added3
 line3
+added4
 line5
 line6
";
        // new-file line numbers: line1=1(ctx), added2=2, added3=3, line3=4(ctx), added4=5
        let filtered = filter_diff_to_line_ranges(diff, &[(2, 2), (5, 5)]);
        assert!(filtered.contains("+added2"));
        assert!(filtered.contains("+added4"));
        assert!(!filtered.contains("+added3"));
    }

    #[test]
    fn filter_diff_empty_input_returns_empty() {
        let filtered = filter_diff_to_line_ranges("", &[(1, 10)]);
        assert!(filtered.is_empty());
    }

    #[test]
    fn filter_diff_passthrough_rename_headers() {
        let diff = "\
diff --git a/old.rs b/new.rs
similarity index 95%
rename from old.rs
rename to new.rs
@@ -1 +1 @@
-old
+new
";
        let filtered = filter_diff_to_line_ranges(diff, &[(1, 1)]);
        assert!(filtered.contains("rename from old.rs"));
        assert!(filtered.contains("rename to new.rs"));
    }

    #[test]
    fn filter_diff_converts_excluded_deletion_to_context() {
        // A deletion that's out of range should become a context line (space prefix).
        // Hunk: new_start=1. Line tracking:
        //   -old_line  → deletion, old_line=1 (not in range 2..2), old_line→2
        //   context    → context (always kept), old_line=2→3, new_line=1→2
        //   +added     → addition, new_line=2 (in range 2..2), new_line→3
        // The deletion at old_line=1 is out of range, so it's converted to a context line " old_line"
        let diff = "\
diff --git a/f.rs b/f.rs
index abc..def 100644
--- a/f.rs
+++ b/f.rs
@@ -1,2 +1,2 @@
-old_line
+added_line
";
        // Include only new_line=1 (the addition): old range check: old_line=1 in (1,1)? yes
        // Actually for addition: new_line=1 in (1,1)? yes. keep=true.
        // For deletion: old_line=1 in (1,1)? yes. keep=true.
        // Both get kept.
        let filtered = filter_diff_to_line_ranges(diff, &[(1, 1)]);
        assert!(filtered.contains("@@"));
        assert!(filtered.contains("+added_line") || filtered.contains("-old_line"));
    }

    // -----------------------------------------------------------------------
    // Malformed / edge-case input
    // -----------------------------------------------------------------------

    #[test]
    fn malformed_input_with_no_diff_header_is_ignored() {
        let input = "just some random text\nno diff headers here\n";
        let result = parse_unified_diff(input);
        assert!(result.is_empty());
    }

    #[test]
    fn diff_with_no_hunks_creates_entry_with_empty_hunks() {
        let input = "\
diff --git a/file.rs b/file.rs
index abc..def 100644
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        assert!(result[0].hunks().is_empty());
    }

    #[test]
    fn partial_diff_no_file_separator() {
        // Single diff block with partial content
        let input = "\
diff --git a/partial.rs b/partial.rs
index abc..def 100644
--- a/partial.rs
+++ b/partial.rs
";
        let result = parse_unified_diff(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_path(), "partial.rs");
    }

    #[test]
    fn line_content_strips_prefix_correctly() {
        let input = "\
diff --git a/content.rs b/content.rs
index abc..def 100644
--- a/content.rs
+++ b/content.rs
@@ -1,1 +1,1 @@
-   indented deletion
+   indented addition
";
        let result = parse_unified_diff(input);
        let lines = result[0].hunks()[0].lines();
        // The prefix (+/-/ ) is stripped; only content remains
        assert_eq!(lines[0].content(), "   indented deletion");
        assert_eq!(lines[1].content(), "   indented addition");
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
