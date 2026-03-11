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

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Empty / trivial input
    // -----------------------------------------------------------------------

    #[test]
    fn empty_input_returns_empty_vec() {
        let result = parse_stash_list_output("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn whitespace_only_returns_empty_vec() {
        let result = parse_stash_list_output("   \n\n\t").unwrap();
        assert!(result.is_empty());
    }

    // -----------------------------------------------------------------------
    // Single entry
    // -----------------------------------------------------------------------

    #[test]
    fn single_entry_parsed_correctly() {
        let output = "stash@{0}\x1eWIP on main: abc1234 feat: something\x1e2026-03-10T01:00:00+00:00\n";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index(), StashId::new(0));
        assert_eq!(result[0].message(), "WIP on main: abc1234 feat: something");
        assert_eq!(result[0].date(), "2026-03-10T01:00:00+00:00");
    }

    #[test]
    fn stash_index_zero() {
        let output = "stash@{0}\x1emessage\x1e2026-01-01T00:00:00+00:00\n";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result[0].index(), StashId::new(0));
    }

    #[test]
    fn stash_index_nonzero() {
        let output = "stash@{5}\x1emessage\x1e2026-01-01T00:00:00+00:00\n";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result[0].index(), StashId::new(5));
    }

    // -----------------------------------------------------------------------
    // Multiple entries
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_entries_parsed_in_order() {
        let output = "\
stash@{0}\x1eWIP on main: latest change\x1e2026-03-11T00:00:00+00:00
stash@{1}\x1eWIP on main: older change\x1e2026-03-10T00:00:00+00:00
stash@{2}\x1eOn feature: experimental\x1e2026-03-09T00:00:00+00:00
";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].index(), StashId::new(0));
        assert_eq!(result[1].index(), StashId::new(1));
        assert_eq!(result[2].index(), StashId::new(2));
    }

    #[test]
    fn multiple_entries_messages_are_correct() {
        let output = "\
stash@{0}\x1efirst message\x1e2026-03-11T00:00:00+00:00
stash@{1}\x1esecond message\x1e2026-03-10T00:00:00+00:00
";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result[0].message(), "first message");
        assert_eq!(result[1].message(), "second message");
    }

    #[test]
    fn multiple_entries_dates_are_correct() {
        let output = "\
stash@{0}\x1emsg1\x1e2026-03-11T12:00:00+00:00
stash@{1}\x1emsg2\x1e2026-03-10T08:00:00+00:00
";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result[0].date(), "2026-03-11T12:00:00+00:00");
        assert_eq!(result[1].date(), "2026-03-10T08:00:00+00:00");
    }

    // -----------------------------------------------------------------------
    // Custom messages
    // -----------------------------------------------------------------------

    #[test]
    fn custom_stash_message() {
        let output = "stash@{0}\x1eMy custom stash message\x1e2026-03-10T00:00:00+00:00\n";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result[0].message(), "My custom stash message");
    }

    #[test]
    fn stash_message_with_colons_and_spaces() {
        let output = "stash@{0}\x1eWIP on main: abc1234 feat: add new feature\x1e2026-03-10T00:00:00+00:00\n";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result[0].message(), "WIP on main: abc1234 feat: add new feature");
    }

    #[test]
    fn stash_message_with_unicode() {
        let output = "stash@{0}\x1eWIP: 作業中の変更\x1e2026-03-10T00:00:00+00:00\n";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result[0].message(), "WIP: 作業中の変更");
    }

    // -----------------------------------------------------------------------
    // Malformed input
    // -----------------------------------------------------------------------

    #[test]
    fn malformed_too_few_fields_is_error() {
        // Only 2 fields (need 3)
        let output = "stash@{0}\x1eonly message\n";
        let result = parse_stash_list_output(output);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_stash_ref_format_is_error() {
        // Not the "stash@{N}" format
        let output = "not-a-stash-ref\x1emessage\x1e2026-03-10T00:00:00+00:00\n";
        let result = parse_stash_list_output(output);
        assert!(result.is_err());
    }

    #[test]
    fn stash_ref_missing_closing_brace_is_error() {
        let output = "stash@{0\x1emessage\x1e2026-03-10T00:00:00+00:00\n";
        let result = parse_stash_list_output(output);
        assert!(result.is_err());
    }

    #[test]
    fn stash_ref_non_numeric_index_is_error() {
        let output = "stash@{abc}\x1emessage\x1e2026-03-10T00:00:00+00:00\n";
        let result = parse_stash_list_output(output);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Edge cases — blank lines between entries are skipped
    // -----------------------------------------------------------------------

    #[test]
    fn blank_lines_between_entries_are_skipped() {
        let output = "\
stash@{0}\x1emsg0\x1e2026-03-11T00:00:00+00:00

stash@{1}\x1emsg1\x1e2026-03-10T00:00:00+00:00
";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn leading_trailing_whitespace_in_fields_is_trimmed() {
        // The parser trims the fields, so extra whitespace around RS-separated values is ok
        let output = "stash@{0}\x1e  trimmed message  \x1e  2026-03-10T00:00:00+00:00  \n";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result[0].message(), "trimmed message");
        assert_eq!(result[0].date(), "2026-03-10T00:00:00+00:00");
    }

    #[test]
    fn large_stash_index() {
        let output = "stash@{99}\x1emany stashes\x1e2026-03-10T00:00:00+00:00\n";
        let result = parse_stash_list_output(output).unwrap();
        assert_eq!(result[0].index(), StashId::new(99));
    }
}
