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

    #[test]
    fn stash_entry_index_returns_stash_id() {
        let stash = sample_stash();
        let id: StashId = stash.index();
        assert_eq!(id.index(), 0);
        assert_eq!(id.to_string(), "stash@{0}");
    }

    #[test]
    fn stash_entry_message_empty() {
        let stash = StashEntry::new(
            StashId::new(0),
            "".to_string(),
            "2026-03-10T01:00:00Z".to_string(),
        );
        assert_eq!(stash.message(), "");
    }

    #[test]
    fn stash_entry_large_index() {
        let stash = StashEntry::new(
            StashId::new(50),
            "old stash".to_string(),
            "2025-01-01T00:00:00Z".to_string(),
        );
        assert_eq!(stash.index().index(), 50);
    }

    #[test]
    fn stash_entry_debug() {
        let stash = sample_stash();
        let debug = format!("{stash:?}");
        assert!(debug.contains("StashEntry"));
        assert!(debug.contains("WIP"));
    }

    #[test]
    fn stash_entries_vec_ordered() {
        let entries: Vec<StashEntry> = (0..3)
            .map(|i| {
                StashEntry::new(
                    StashId::new(i),
                    format!("WIP stash {i}"),
                    "2026-03-10T01:00:00Z".to_string(),
                )
            })
            .collect();
        assert_eq!(entries[0].index().index(), 0);
        assert_eq!(entries[1].index().index(), 1);
        assert_eq!(entries[2].index().index(), 2);
    }

    #[test]
    fn stash_entry_date_field() {
        let stash = StashEntry::new(
            StashId::new(0),
            "some stash".to_string(),
            "2099-12-31T23:59:59Z".to_string(),
        );
        assert_eq!(stash.date(), "2099-12-31T23:59:59Z");
    }
}
