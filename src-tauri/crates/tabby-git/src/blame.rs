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
}
