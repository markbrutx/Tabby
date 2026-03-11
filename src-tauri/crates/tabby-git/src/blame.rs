use crate::value_objects::CommitHash;

/// A single blame entry mapping a range of lines to a commit.
#[derive(Debug, Clone, PartialEq)]
pub struct BlameEntry {
    hash: CommitHash,
    author: String,
    date: String,
    line_start: u32,
    line_count: u32,
    content: String,
}

impl BlameEntry {
    pub fn new(
        hash: CommitHash,
        author: String,
        date: String,
        line_start: u32,
        line_count: u32,
        content: String,
    ) -> Self {
        Self {
            hash,
            author,
            date,
            line_start,
            line_count,
            content,
        }
    }

    pub fn hash(&self) -> &CommitHash {
        &self.hash
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn line_start(&self) -> u32 {
        self.line_start
    }

    pub fn line_count(&self) -> u32 {
        self.line_count
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hash() -> CommitHash {
        CommitHash::try_new("deadbeef").expect("valid")
    }

    fn sample_entry() -> BlameEntry {
        BlameEntry::new(
            sample_hash(),
            "Alice".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
            1,
            5,
            "fn main() {}".to_string(),
        )
    }

    #[test]
    fn blame_entry_field_access() {
        let entry = sample_entry();
        assert_eq!(entry.hash(), &sample_hash());
        assert_eq!(entry.author(), "Alice");
        assert_eq!(entry.date(), "2026-03-10T01:00:00Z");
        assert_eq!(entry.line_start(), 1);
        assert_eq!(entry.line_count(), 5);
        assert_eq!(entry.content(), "fn main() {}");
    }

    #[test]
    fn blame_entry_equality() {
        let a = sample_entry();
        let b = sample_entry();
        assert_eq!(a, b);
    }

    #[test]
    fn blame_entry_inequality() {
        let a = sample_entry();
        let b = BlameEntry::new(
            CommitHash::try_new("cafebabe").expect("valid"),
            "Bob".to_string(),
            "2026-03-10T02:00:00Z".to_string(),
            10,
            3,
            "let x = 1;".to_string(),
        );
        assert_ne!(a, b);
    }

    #[test]
    fn blame_entry_clone() {
        let a = sample_entry();
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn blame_entry_line_start_zero() {
        let entry = BlameEntry::new(
            sample_hash(),
            "Alice".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
            0,
            1,
            "fn init() {}".to_string(),
        );
        assert_eq!(entry.line_start(), 0);
        assert_eq!(entry.line_count(), 1);
    }

    #[test]
    fn blame_entry_large_line_range() {
        let entry = BlameEntry::new(
            sample_hash(),
            "Bob".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            100,
            500,
            "big block".to_string(),
        );
        assert_eq!(entry.line_start(), 100);
        assert_eq!(entry.line_count(), 500);
    }

    #[test]
    fn blame_entry_empty_content() {
        let entry = BlameEntry::new(
            sample_hash(),
            "Alice".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
            1,
            1,
            "".to_string(),
        );
        assert_eq!(entry.content(), "");
    }

    #[test]
    fn blame_entry_multiline_content() {
        let content = "line1\nline2\nline3";
        let entry = BlameEntry::new(
            sample_hash(),
            "Alice".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
            1,
            3,
            content.to_string(),
        );
        assert_eq!(entry.content(), content);
    }

    #[test]
    fn blame_entry_debug() {
        let entry = sample_entry();
        let debug = format!("{entry:?}");
        assert!(debug.contains("BlameEntry"));
        assert!(debug.contains("Alice"));
    }

    #[test]
    fn blame_entry_hash_accessor_returns_ref() {
        let entry = sample_entry();
        let hash_ref: &CommitHash = entry.hash();
        assert_eq!(hash_ref, &sample_hash());
    }

    #[test]
    fn blame_entries_can_be_collected_into_vec() {
        let entries: Vec<BlameEntry> = (0..5)
            .map(|i| {
                BlameEntry::new(
                    CommitHash::try_new(format!("deadbeef{i:08}")).unwrap_or_else(|_| {
                        CommitHash::try_new("deadbeef").expect("fallback valid")
                    }),
                    format!("Author {i}"),
                    "2026-03-10T01:00:00Z".to_string(),
                    i as u32 * 10 + 1,
                    10,
                    format!("block {i}"),
                )
            })
            .collect();
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].line_start(), 1);
        assert_eq!(entries[4].line_start(), 41);
    }
}
