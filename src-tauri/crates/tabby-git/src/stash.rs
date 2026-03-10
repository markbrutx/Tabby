use crate::value_objects::StashId;

/// A single stash entry.
#[derive(Debug, Clone, PartialEq)]
pub struct StashEntry {
    index: StashId,
    message: String,
    date: String,
}

impl StashEntry {
    pub fn new(index: StashId, message: String, date: String) -> Self {
        Self {
            index,
            message,
            date,
        }
    }

    pub fn index(&self) -> StashId {
        self.index
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn date(&self) -> &str {
        &self.date
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stash() -> StashEntry {
        StashEntry::new(
            StashId::new(0),
            "WIP on main: abc1234 feat: something".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
        )
    }

    #[test]
    fn stash_entry_field_access() {
        let stash = sample_stash();
        assert_eq!(stash.index(), StashId::new(0));
        assert_eq!(stash.message(), "WIP on main: abc1234 feat: something");
        assert_eq!(stash.date(), "2026-03-10T01:00:00Z");
    }

    #[test]
    fn stash_entry_equality() {
        let a = sample_stash();
        let b = sample_stash();
        assert_eq!(a, b);
    }

    #[test]
    fn stash_entry_inequality() {
        let a = sample_stash();
        let b = StashEntry::new(
            StashId::new(1),
            "WIP on develop".to_string(),
            "2026-03-10T02:00:00Z".to_string(),
        );
        assert_ne!(a, b);
    }

    #[test]
    fn stash_entry_clone() {
        let a = sample_stash();
        let b = a.clone();
        assert_eq!(a, b);
    }
}
