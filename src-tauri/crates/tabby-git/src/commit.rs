use crate::value_objects::CommitHash;

/// Information about a single Git commit.
#[derive(Debug, Clone, PartialEq)]
pub struct CommitInfo {
    hash: CommitHash,
    short_hash: String,
    author_name: String,
    author_email: String,
    date: String,
    message: String,
    parent_hashes: Vec<CommitHash>,
}

impl CommitInfo {
    pub fn new(
        hash: CommitHash,
        short_hash: String,
        author_name: String,
        author_email: String,
        date: String,
        message: String,
        parent_hashes: Vec<CommitHash>,
    ) -> Self {
        Self {
            hash,
            short_hash,
            author_name,
            author_email,
            date,
            message,
            parent_hashes,
        }
    }

    pub fn hash(&self) -> &CommitHash {
        &self.hash
    }

    pub fn short_hash(&self) -> &str {
        &self.short_hash
    }

    pub fn author_name(&self) -> &str {
        &self.author_name
    }

    pub fn author_email(&self) -> &str {
        &self.author_email
    }

    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn parent_hashes(&self) -> &[CommitHash] {
        &self.parent_hashes
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hash() -> CommitHash {
        CommitHash::try_new("abc123def456abc123def456abc123def456abc1").expect("valid hash")
    }

    fn sample_parent() -> CommitHash {
        CommitHash::try_new("1111111111111111111111111111111111111111").expect("valid hash")
    }

    fn sample_commit() -> CommitInfo {
        CommitInfo::new(
            sample_hash(),
            "abc123d".to_string(),
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
            "feat: initial commit".to_string(),
            vec![sample_parent()],
        )
    }

    #[test]
    fn commit_info_field_access() {
        let commit = sample_commit();
        assert_eq!(commit.hash(), &sample_hash());
        assert_eq!(commit.short_hash(), "abc123d");
        assert_eq!(commit.author_name(), "Alice");
        assert_eq!(commit.author_email(), "alice@example.com");
        assert_eq!(commit.date(), "2026-03-10T01:00:00Z");
        assert_eq!(commit.message(), "feat: initial commit");
        assert_eq!(commit.parent_hashes().len(), 1);
        assert_eq!(commit.parent_hashes()[0], sample_parent());
    }

    #[test]
    fn commit_info_no_parents() {
        let commit = CommitInfo::new(
            sample_hash(),
            "abc123d".to_string(),
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
            "initial commit".to_string(),
            vec![],
        );
        assert!(commit.parent_hashes().is_empty());
    }

    #[test]
    fn commit_info_multiple_parents() {
        let parent2 =
            CommitHash::try_new("2222222222222222222222222222222222222222").expect("valid");
        let commit = CommitInfo::new(
            sample_hash(),
            "abc123d".to_string(),
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
            "merge commit".to_string(),
            vec![sample_parent(), parent2.clone()],
        );
        assert_eq!(commit.parent_hashes().len(), 2);
        assert_eq!(commit.parent_hashes()[1], parent2);
    }

    #[test]
    fn commit_info_equality() {
        let a = sample_commit();
        let b = sample_commit();
        assert_eq!(a, b);
    }

    #[test]
    fn commit_info_inequality() {
        let a = sample_commit();
        let b = CommitInfo::new(
            sample_hash(),
            "abc123d".to_string(),
            "Bob".to_string(),
            "bob@example.com".to_string(),
            "2026-03-10T02:00:00Z".to_string(),
            "different commit".to_string(),
            vec![],
        );
        assert_ne!(a, b);
    }

    #[test]
    fn commit_info_clone() {
        let a = sample_commit();
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn commit_info_empty_message() {
        let commit = CommitInfo::new(
            sample_hash(),
            "abc123d".to_string(),
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
            "".to_string(),
            vec![],
        );
        assert_eq!(commit.message(), "");
    }

    #[test]
    fn commit_info_multiline_message() {
        let msg = "feat: add feature\n\nThis adds a new feature\nwith multiple lines.";
        let commit = CommitInfo::new(
            sample_hash(),
            "abc123d".to_string(),
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
            msg.to_string(),
            vec![],
        );
        assert_eq!(commit.message(), msg);
    }

    #[test]
    fn commit_info_short_hash_different_from_full() {
        let commit = sample_commit();
        assert_ne!(commit.short_hash(), commit.hash().as_ref());
    }

    #[test]
    fn commit_info_author_email_format() {
        let commit = sample_commit();
        assert!(commit.author_email().contains('@'));
    }

    #[test]
    fn commit_info_debug() {
        let commit = sample_commit();
        let debug = format!("{commit:?}");
        assert!(debug.contains("CommitInfo"));
        assert!(debug.contains("Alice"));
    }

    #[test]
    fn commit_info_parent_hashes_returns_slice() {
        let commit = sample_commit();
        let parents: &[CommitHash] = commit.parent_hashes();
        assert_eq!(parents.len(), 1);
    }
}
