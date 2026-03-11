use tabby_git::value_objects::CommitHash;
use tabby_git::BlameEntry;

use crate::shell::error::ShellError;

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
pub(super) fn parse_blame_porcelain(output: &str) -> Result<Vec<BlameEntry>, ShellError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal valid porcelain blame block helper
    fn make_blame_block(
        hash: &str,
        author: &str,
        author_time: &str,
        final_line: usize,
        content: &str,
    ) -> String {
        format!(
            "{hash} 1 {final_line} 1\nauthor {author}\nauthor-mail <{author}@example.com>\nauthor-time {author_time}\nauthor-tz +0000\ncommitter {author}\ncommitter-mail <{author}@example.com>\ncommitter-time {author_time}\ncommitter-tz +0000\nsummary test commit\nfilename src/lib.rs\n\t{content}\n",
        )
    }

    // -----------------------------------------------------------------------
    // Empty / trivial input
    // -----------------------------------------------------------------------

    #[test]
    fn empty_input_returns_empty_vec() {
        let result = parse_blame_porcelain("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn whitespace_only_returns_empty_vec() {
        let result = parse_blame_porcelain("   \n\n").unwrap();
        assert!(result.is_empty());
    }

    // -----------------------------------------------------------------------
    // Single commit, single line
    // -----------------------------------------------------------------------

    #[test]
    fn single_line_single_commit() {
        let block = make_blame_block(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "Alice",
            "1709856000",
            1,
            "fn main() {}",
        );
        let result = parse_blame_porcelain(&block).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].author(), "Alice");
        assert_eq!(result[0].date(), "1709856000");
        assert_eq!(result[0].line_start(), 1);
        assert_eq!(result[0].line_count(), 1);
        assert_eq!(result[0].content(), "fn main() {}");
    }

    #[test]
    fn single_commit_hash_is_lowercase() {
        let block = make_blame_block(
            "DEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF",
            "Alice",
            "1709856000",
            1,
            "code",
        );
        let result = parse_blame_porcelain(&block).unwrap();
        // Parser lowercases the hash
        assert_eq!(
            result[0].hash().as_ref(),
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        );
    }

    // -----------------------------------------------------------------------
    // Multiple lines from the same commit — should be merged into one entry
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_lines_same_commit_merged_into_one_entry() {
        let hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
        let output = format!(
            "{hash} 1 1 3\nauthor Alice\nauthor-mail <alice@example.com>\nauthor-time 1709856000\nauthor-tz +0000\ncommitter Alice\ncommitter-mail <alice@example.com>\ncommitter-time 1709856000\ncommitter-tz +0000\nsummary initial commit\nfilename src/main.rs\n\tfn main() {{\n{hash} 2 2\n\t    println!(\"hello\");\n{hash} 3 3\n\t}}\n"
        );
        let result = parse_blame_porcelain(&output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line_count(), 3);
        assert_eq!(result[0].author(), "Alice");
    }

    #[test]
    fn multiple_lines_same_commit_content_is_joined_with_newline() {
        let hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
        let output = format!(
            "{hash} 1 1 2\nauthor Alice\nauthor-mail <alice@example.com>\nauthor-time 1709856000\nauthor-tz +0000\ncommitter Alice\ncommitter-mail <alice@example.com>\ncommitter-time 1709856000\ncommitter-tz +0000\nsummary initial commit\nfilename src/main.rs\n\tline one\n{hash} 2 2\n\tline two\n"
        );
        let result = parse_blame_porcelain(&output).unwrap();
        assert!(result[0].content().contains("line one"));
        assert!(result[0].content().contains("line two"));
    }

    // -----------------------------------------------------------------------
    // Multiple commits — different entries
    // -----------------------------------------------------------------------

    #[test]
    fn two_commits_produce_two_entries() {
        let hash1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let hash2 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let block1 = make_blame_block(hash1, "Alice", "1709856000", 1, "first line");
        let block2 = make_blame_block(hash2, "Bob", "1709856001", 2, "second line");
        let output = format!("{block1}{block2}");
        let result = parse_blame_porcelain(&output).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].author(), "Alice");
        assert_eq!(result[1].author(), "Bob");
    }

    #[test]
    fn alternating_commits_produce_separate_entries() {
        // commit A, then B, then A again → 3 entries (not merged because not contiguous)
        let hash_a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let hash_b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let block_a1 = make_blame_block(hash_a, "Alice", "1709856000", 1, "line 1");
        let block_b = make_blame_block(hash_b, "Bob", "1709856001", 2, "line 2");
        let block_a2 = make_blame_block(hash_a, "Alice", "1709856000", 3, "line 3");
        let output = format!("{block_a1}{block_b}{block_a2}");
        let result = parse_blame_porcelain(&output).unwrap();
        assert_eq!(result.len(), 3);
    }

    // -----------------------------------------------------------------------
    // Special author name characters
    // -----------------------------------------------------------------------

    #[test]
    fn author_with_spaces_in_name() {
        let block = make_blame_block(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "Alice Smith",
            "1709856000",
            1,
            "code",
        );
        let result = parse_blame_porcelain(&block).unwrap();
        assert_eq!(result[0].author(), "Alice Smith");
    }

    #[test]
    fn author_with_unicode_name() {
        // The parser stores the author line verbatim after "author "
        let hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
        let output = format!(
            "{hash} 1 1 1\nauthor 田中太郎\nauthor-mail <tanaka@example.com>\nauthor-time 1709856000\nauthor-tz +0900\ncommitter 田中太郎\ncommitter-mail <tanaka@example.com>\ncommitter-time 1709856000\ncommitter-tz +0900\nsummary add stuff\nfilename main.rs\n\t日本語のコード\n"
        );
        let result = parse_blame_porcelain(&output).unwrap();
        assert_eq!(result[0].author(), "田中太郎");
        assert_eq!(result[0].content(), "日本語のコード");
    }

    // -----------------------------------------------------------------------
    // Boundary commit (hash starts with zeros)
    // -----------------------------------------------------------------------

    #[test]
    fn boundary_commit_hash_with_zeros() {
        let hash = "0000000000000000000000000000000000000000";
        let output = format!(
            "{hash} 1 1 1\nauthor Not Committed Yet\nauthor-mail <not.committed.yet>\nauthor-time 1709856000\nauthor-tz +0000\ncommitter Not Committed Yet\ncommitter-mail <not.committed.yet>\ncommitter-time 1709856000\ncommitter-tz +0000\nsummary Version of test\nfilename test.rs\n\tuncommitted line\n"
        );
        let result = parse_blame_porcelain(&output).unwrap();
        assert_eq!(result[0].author(), "Not Committed Yet");
        assert_eq!(result[0].content(), "uncommitted line");
    }

    // -----------------------------------------------------------------------
    // Short (4-char) hashes
    // -----------------------------------------------------------------------

    #[test]
    fn short_hash_minimum_four_chars() {
        // 4-char hex hash is valid for CommitHash
        let hash = "dead";
        let output = format!(
            "{hash} 1 1 1\nauthor Alice\nauthor-mail <a@b.com>\nauthor-time 100\nauthor-tz +0000\ncommitter Alice\ncommitter-mail <a@b.com>\ncommitter-time 100\ncommitter-tz +0000\nsummary msg\nfilename f.rs\n\tcode\n"
        );
        let result = parse_blame_porcelain(&output).unwrap();
        assert_eq!(result[0].hash().as_ref(), "dead");
    }

    // -----------------------------------------------------------------------
    // Content with special characters
    // -----------------------------------------------------------------------

    #[test]
    fn content_with_tab_prefix_stripped() {
        // The content line starts with \t; the parser strips that prefix
        let block = make_blame_block(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "Alice",
            "1709856000",
            1,
            "    let x = 1; // indented",
        );
        let result = parse_blame_porcelain(&block).unwrap();
        assert_eq!(result[0].content(), "    let x = 1; // indented");
    }

    #[test]
    fn content_with_special_characters() {
        let block = make_blame_block(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "Alice",
            "1709856000",
            1,
            "let s = \"hello \\\"world\\\"\";",
        );
        let result = parse_blame_porcelain(&block).unwrap();
        assert_eq!(result[0].content(), "let s = \"hello \\\"world\\\"\";");
    }

    // -----------------------------------------------------------------------
    // Blame with many lines from different commits
    // -----------------------------------------------------------------------

    #[test]
    fn five_lines_three_commits_merges_contiguous() {
        let hash_a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let hash_b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let hash_c = "cccccccccccccccccccccccccccccccccccccccc";

        // A: lines 1-2, B: line 3, C: lines 4-5
        let block_a1 = make_blame_block(hash_a, "Alice", "100", 1, "line1");
        let block_a2 = make_blame_block(hash_a, "Alice", "100", 2, "line2");
        let block_b = make_blame_block(hash_b, "Bob", "200", 3, "line3");
        let block_c1 = make_blame_block(hash_c, "Carol", "300", 4, "line4");
        let block_c2 = make_blame_block(hash_c, "Carol", "300", 5, "line5");

        let output = format!("{block_a1}{block_a2}{block_b}{block_c1}{block_c2}");
        let result = parse_blame_porcelain(&output).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].line_count(), 2); // A: 2 lines merged
        assert_eq!(result[1].line_count(), 1); // B: 1 line
        assert_eq!(result[2].line_count(), 2); // C: 2 lines merged
    }
}
