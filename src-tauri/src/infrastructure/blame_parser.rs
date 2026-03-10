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
