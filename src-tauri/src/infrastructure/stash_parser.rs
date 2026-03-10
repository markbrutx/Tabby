use tabby_git::value_objects::StashId;
use tabby_git::StashEntry;

use crate::shell::error::ShellError;

/// Parse `git stash list --format=%gd%x1e%gs%x1e%aI` output.
///
/// Each line: `stash@{N}<RS>message<RS>date`
pub(super) fn parse_stash_list_output(output: &str) -> Result<Vec<StashEntry>, ShellError> {
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
