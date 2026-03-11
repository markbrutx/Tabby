use std::fmt;

use tabby_kernel::ValueObjectError;

// ---------------------------------------------------------------------------
// BranchName
// ---------------------------------------------------------------------------

/// A validated Git branch name.
///
/// Must be non-empty and must not contain spaces.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchName(String);

impl BranchName {
    /// Validated constructor — rejects empty strings and strings containing spaces.
    pub fn try_new(value: impl Into<String>) -> Result<Self, ValueObjectError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ValueObjectError::new("BranchName must not be empty"));
        }
        if value.contains(' ') {
            return Err(ValueObjectError::new("BranchName must not contain spaces"));
        }
        Ok(Self(value))
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for BranchName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// CommitHash
// ---------------------------------------------------------------------------

/// A validated Git commit hash (short or full).
///
/// Must be 4–40 lowercase hexadecimal characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitHash(String);

impl CommitHash {
    /// Validated constructor — accepts 4–40 hex character strings.
    pub fn try_new(value: impl Into<String>) -> Result<Self, ValueObjectError> {
        let value = value.into();
        let len = value.len();
        if !(4..=40).contains(&len) {
            return Err(ValueObjectError::new(format!(
                "CommitHash must be 4–40 hex characters, got {len}"
            )));
        }
        if !value.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValueObjectError::new(
                "CommitHash must contain only hexadecimal characters (0-9, a-f, A-F)",
            ));
        }
        // Normalize to lowercase for consistent comparison.
        Ok(Self(value.to_ascii_lowercase()))
    }
}

impl fmt::Display for CommitHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for CommitHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// RemoteName
// ---------------------------------------------------------------------------

/// A validated Git remote name (e.g. "origin", "upstream").
///
/// Must be non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemoteName(String);

impl RemoteName {
    /// Validated constructor — rejects empty strings.
    pub fn try_new(value: impl Into<String>) -> Result<Self, ValueObjectError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ValueObjectError::new("RemoteName must not be empty"));
        }
        Ok(Self(value))
    }
}

impl fmt::Display for RemoteName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for RemoteName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// StashId
// ---------------------------------------------------------------------------

/// A Git stash entry identifier (index into the stash list).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StashId(usize);

impl StashId {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn index(self) -> usize {
        self.0
    }
}

impl fmt::Display for StashId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stash@{{{}}}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- BranchName --------------------------------------------------------

    #[test]
    fn branch_name_accepts_simple_name() {
        let name = BranchName::try_new("main").expect("should accept simple name");
        assert_eq!(name.as_ref(), "main");
        assert_eq!(name.to_string(), "main");
    }

    #[test]
    fn branch_name_accepts_slashes() {
        let name = BranchName::try_new("feature/GIT-002").expect("should accept slashes");
        assert_eq!(name.as_ref(), "feature/GIT-002");
    }

    #[test]
    fn branch_name_accepts_hyphens_and_dots() {
        let name = BranchName::try_new("release-v1.0.0").expect("should accept hyphens and dots");
        assert_eq!(name.as_ref(), "release-v1.0.0");
    }

    #[test]
    fn branch_name_rejects_empty() {
        let err = BranchName::try_new("").expect_err("should reject empty");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn branch_name_rejects_spaces() {
        let err = BranchName::try_new("my branch").expect_err("should reject spaces");
        assert!(err.to_string().contains("must not contain spaces"));
    }

    #[test]
    fn branch_name_equality() {
        let a = BranchName::try_new("main").expect("valid");
        let b = BranchName::try_new("main").expect("valid");
        let c = BranchName::try_new("develop").expect("valid");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn branch_name_clone() {
        let a = BranchName::try_new("main").expect("valid");
        let b = a.clone();
        assert_eq!(a, b);
    }

    // -- CommitHash --------------------------------------------------------

    #[test]
    fn commit_hash_accepts_full_hash() {
        let hash = CommitHash::try_new("abc123def456abc123def456abc123def456abc1")
            .expect("should accept 40-char hash");
        assert_eq!(hash.as_ref(), "abc123def456abc123def456abc123def456abc1");
    }

    #[test]
    fn commit_hash_accepts_short_hash() {
        let hash = CommitHash::try_new("abcd").expect("should accept 4-char hash");
        assert_eq!(hash.as_ref(), "abcd");
    }

    #[test]
    fn commit_hash_accepts_7_char_hash() {
        let hash = CommitHash::try_new("abc1234").expect("should accept 7-char hash");
        assert_eq!(hash.to_string(), "abc1234");
    }

    #[test]
    fn commit_hash_normalizes_to_lowercase() {
        let hash = CommitHash::try_new("ABCDEF12").expect("should accept uppercase hex");
        assert_eq!(hash.as_ref(), "abcdef12");
    }

    #[test]
    fn commit_hash_rejects_too_short() {
        let err = CommitHash::try_new("abc").expect_err("should reject <4 chars");
        assert!(err.to_string().contains("4–40 hex characters"));
    }

    #[test]
    fn commit_hash_rejects_too_long() {
        let long = "a".repeat(41);
        let err = CommitHash::try_new(long).expect_err("should reject >40 chars");
        assert!(err.to_string().contains("4–40 hex characters"));
    }

    #[test]
    fn commit_hash_rejects_non_hex() {
        let err = CommitHash::try_new("ghij1234").expect_err("should reject non-hex chars");
        assert!(err.to_string().contains("hexadecimal"));
    }

    #[test]
    fn commit_hash_rejects_empty() {
        let err = CommitHash::try_new("").expect_err("should reject empty");
        assert!(err.to_string().contains("4–40 hex characters"));
    }

    #[test]
    fn commit_hash_equality_after_normalization() {
        let a = CommitHash::try_new("ABCD1234").expect("valid");
        let b = CommitHash::try_new("abcd1234").expect("valid");
        assert_eq!(a, b);
    }

    #[test]
    fn commit_hash_clone() {
        let a = CommitHash::try_new("deadbeef").expect("valid");
        let b = a.clone();
        assert_eq!(a, b);
    }

    // -- RemoteName --------------------------------------------------------

    #[test]
    fn remote_name_accepts_origin() {
        let name = RemoteName::try_new("origin").expect("should accept 'origin'");
        assert_eq!(name.as_ref(), "origin");
        assert_eq!(name.to_string(), "origin");
    }

    #[test]
    fn remote_name_accepts_upstream() {
        let name = RemoteName::try_new("upstream").expect("should accept 'upstream'");
        assert_eq!(name.as_ref(), "upstream");
    }

    #[test]
    fn remote_name_accepts_hyphenated() {
        let name = RemoteName::try_new("my-fork").expect("should accept hyphenated");
        assert_eq!(name.as_ref(), "my-fork");
    }

    #[test]
    fn remote_name_rejects_empty() {
        let err = RemoteName::try_new("").expect_err("should reject empty");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn remote_name_equality() {
        let a = RemoteName::try_new("origin").expect("valid");
        let b = RemoteName::try_new("origin").expect("valid");
        let c = RemoteName::try_new("upstream").expect("valid");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn remote_name_clone() {
        let a = RemoteName::try_new("origin").expect("valid");
        let b = a.clone();
        assert_eq!(a, b);
    }

    // -- StashId -----------------------------------------------------------

    #[test]
    fn stash_id_new_and_index() {
        let id = StashId::new(0);
        assert_eq!(id.index(), 0);
    }

    #[test]
    fn stash_id_display() {
        let id = StashId::new(0);
        assert_eq!(id.to_string(), "stash@{0}");

        let id = StashId::new(3);
        assert_eq!(id.to_string(), "stash@{3}");
    }

    #[test]
    fn stash_id_equality() {
        let a = StashId::new(0);
        let b = StashId::new(0);
        let c = StashId::new(1);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn stash_id_copy() {
        let a = StashId::new(5);
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn stash_id_large_index() {
        let id = StashId::new(999);
        assert_eq!(id.index(), 999);
        assert_eq!(id.to_string(), "stash@{999}");
    }

    // -- BranchName edge cases ---------------------------------------------

    #[test]
    fn branch_name_single_char_accepted() {
        let name = BranchName::try_new("x").expect("single char should be valid");
        assert_eq!(name.as_ref(), "x");
    }

    #[test]
    fn branch_name_with_numbers() {
        let name = BranchName::try_new("branch-42").expect("numbers should be valid");
        assert_eq!(name.to_string(), "branch-42");
    }

    #[test]
    fn branch_name_unicode_accepted() {
        let name = BranchName::try_new("feature/café").expect("unicode should be valid");
        assert_eq!(name.as_ref(), "feature/café");
    }

    #[test]
    fn branch_name_rejects_leading_space() {
        let err = BranchName::try_new(" leading").expect_err("leading space should fail");
        assert!(err.to_string().contains("must not contain spaces"));
    }

    #[test]
    fn branch_name_rejects_trailing_space() {
        let err = BranchName::try_new("trailing ").expect_err("trailing space should fail");
        assert!(err.to_string().contains("must not contain spaces"));
    }

    #[test]
    fn branch_name_display_equals_as_ref() {
        let name = BranchName::try_new("feature/test").expect("valid");
        assert_eq!(name.to_string(), name.as_ref());
    }

    #[test]
    fn branch_name_hash_used_in_hashset() {
        use std::collections::HashSet;
        let a = BranchName::try_new("main").expect("valid");
        let b = BranchName::try_new("develop").expect("valid");
        let c = BranchName::try_new("main").expect("valid");
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c); // duplicate of a
        assert_eq!(set.len(), 2);
    }

    // -- CommitHash edge cases ---------------------------------------------

    #[test]
    fn commit_hash_exactly_4_chars_accepted() {
        let hash = CommitHash::try_new("abcd").expect("4 chars should be valid");
        assert_eq!(hash.as_ref(), "abcd");
    }

    #[test]
    fn commit_hash_exactly_40_chars_accepted() {
        let hash = CommitHash::try_new("abcdef1234567890abcdef1234567890abcdef12")
            .expect("40 chars should be valid");
        assert_eq!(hash.as_ref().len(), 40);
    }

    #[test]
    fn commit_hash_exactly_3_chars_rejected() {
        CommitHash::try_new("abc").expect_err("3 chars should be rejected");
    }

    #[test]
    fn commit_hash_exactly_41_chars_rejected() {
        let long = "a".repeat(41);
        CommitHash::try_new(long).expect_err("41 chars should be rejected");
    }

    #[test]
    fn commit_hash_mixed_case_normalizes() {
        let hash = CommitHash::try_new("DEADBEEF").expect("uppercase valid");
        assert_eq!(hash.as_ref(), "deadbeef");
        assert_eq!(hash.to_string(), "deadbeef");
    }

    #[test]
    fn commit_hash_display_equals_as_ref() {
        let hash = CommitHash::try_new("deadbeef").expect("valid");
        assert_eq!(hash.to_string(), hash.as_ref());
    }

    #[test]
    fn commit_hash_hash_in_hashmap() {
        use std::collections::HashMap;
        let h1 = CommitHash::try_new("deadbeef").expect("valid");
        let h2 = CommitHash::try_new("cafebabe").expect("valid");
        let mut map = HashMap::new();
        map.insert(h1.clone(), "first");
        map.insert(h2, "second");
        assert_eq!(*map.get(&h1).unwrap(), "first");
    }

    #[test]
    fn commit_hash_rejects_spaces() {
        CommitHash::try_new("dead beef").expect_err("space is not hex");
    }

    // -- RemoteName edge cases ---------------------------------------------

    #[test]
    fn remote_name_single_char_accepted() {
        let name = RemoteName::try_new("o").expect("single char should be valid");
        assert_eq!(name.as_ref(), "o");
    }

    #[test]
    fn remote_name_with_slashes_accepted() {
        // git does allow complex remote names
        let name = RemoteName::try_new("company/fork").expect("slashes valid");
        assert_eq!(name.to_string(), "company/fork");
    }

    #[test]
    fn remote_name_display_equals_as_ref() {
        let name = RemoteName::try_new("origin").expect("valid");
        assert_eq!(name.to_string(), name.as_ref());
    }

    #[test]
    fn remote_name_hash_in_hashset() {
        use std::collections::HashSet;
        let a = RemoteName::try_new("origin").expect("valid");
        let b = RemoteName::try_new("origin").expect("valid");
        let c = RemoteName::try_new("upstream").expect("valid");
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(set.len(), 2);
    }

    // -- StashId edge cases ------------------------------------------------

    #[test]
    fn stash_id_zero() {
        let id = StashId::new(0);
        assert_eq!(id.index(), 0);
        assert_eq!(id.to_string(), "stash@{0}");
    }

    #[test]
    fn stash_id_max_usize() {
        let id = StashId::new(usize::MAX);
        assert_eq!(id.index(), usize::MAX);
    }

    #[test]
    fn stash_id_hash_in_hashset() {
        use std::collections::HashSet;
        let a = StashId::new(0);
        let b = StashId::new(0);
        let c = StashId::new(1);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn stash_id_debug() {
        let id = StashId::new(5);
        let debug = format!("{id:?}");
        assert!(debug.contains("5"));
    }
}
