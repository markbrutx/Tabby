use std::fmt;

/// Type-safe identifier for a runtime session (PTY or browser instance).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeSessionId(String);

impl fmt::Display for RuntimeSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for RuntimeSessionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for RuntimeSessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::RuntimeSessionId;

    fn sid(s: &str) -> RuntimeSessionId {
        RuntimeSessionId::from(String::from(s))
    }

    // ---------------------------------------------------------------------------
    // Construction and basic accessors
    // ---------------------------------------------------------------------------

    #[test]
    fn display_shows_inner_value() {
        let id = sid("pty-abc");
        assert_eq!(id.to_string(), "pty-abc");
    }

    #[test]
    fn from_string() {
        let id = sid("session-1");
        assert_eq!(id.as_ref(), "session-1");
    }

    #[test]
    fn as_ref_returns_inner_str() {
        let id = sid("browser-xyz");
        let s: &str = id.as_ref();
        assert_eq!(s, "browser-xyz");
    }

    #[test]
    fn debug_format() {
        let id = sid("pty-1");
        let debug = format!("{id:?}");
        assert!(debug.contains("pty-1"));
    }

    #[test]
    fn display_format_matches_inner_string() {
        let id = sid("session-display");
        assert_eq!(format!("{id}"), "session-display");
    }

    #[test]
    fn from_empty_string_is_allowed() {
        // RuntimeSessionId has no validation; empty strings are accepted
        let id = sid("");
        assert_eq!(id.as_ref(), "");
        assert_eq!(id.to_string(), "");
    }

    #[test]
    fn from_string_with_special_characters() {
        let id = sid("pty-\u{1F680}-session");
        assert_eq!(id.as_ref(), "pty-\u{1F680}-session");
    }

    #[test]
    fn from_string_with_uuid_like_value() {
        let id = sid("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    // ---------------------------------------------------------------------------
    // Equality and identity
    // ---------------------------------------------------------------------------

    #[test]
    fn equality() {
        let a = sid("s1");
        let b = sid("s1");
        let c = sid("s2");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn equality_is_reflexive() {
        let id = sid("reflexive");
        assert_eq!(id, id.clone());
    }

    #[test]
    fn equality_is_symmetric() {
        let a = sid("sym");
        let b = sid("sym");
        assert_eq!(a, b);
        assert_eq!(b, a);
    }

    #[test]
    fn equality_is_transitive() {
        let a = sid("trans");
        let b = sid("trans");
        let c = sid("trans");
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a, c);
    }

    #[test]
    fn inequality_different_values() {
        let a = sid("alpha");
        let b = sid("beta");
        assert_ne!(a, b);
    }

    #[test]
    fn case_sensitive_equality() {
        let lower = sid("session");
        let upper = sid("Session");
        assert_ne!(lower, upper);
    }

    // ---------------------------------------------------------------------------
    // Clone
    // ---------------------------------------------------------------------------

    #[test]
    fn clone_is_equal() {
        let id = sid("s1");
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn clone_is_independent_value() {
        let id = sid("original");
        let cloned = id.clone();
        // Both produce the same string independently
        assert_eq!(id.as_ref(), cloned.as_ref());
    }

    #[test]
    fn clone_produces_equal_display() {
        let id = sid("clone-display");
        let cloned = id.clone();
        assert_eq!(id.to_string(), cloned.to_string());
    }

    // ---------------------------------------------------------------------------
    // Hash — use in collections
    // ---------------------------------------------------------------------------

    #[test]
    fn usable_as_hashmap_key() {
        let mut map: HashMap<RuntimeSessionId, u32> = HashMap::new();
        let id = sid("map-key");
        map.insert(id.clone(), 42);
        assert_eq!(map.get(&id), Some(&42));
    }

    #[test]
    fn equal_ids_share_hashmap_slot() {
        let mut map: HashMap<RuntimeSessionId, &str> = HashMap::new();
        map.insert(sid("shared"), "first");
        map.insert(sid("shared"), "second");
        assert_eq!(map.len(), 1);
        assert_eq!(map[&sid("shared")], "second");
    }

    #[test]
    fn usable_in_hashset_deduplication() {
        let mut set: HashSet<RuntimeSessionId> = HashSet::new();
        set.insert(sid("dup"));
        set.insert(sid("dup"));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn distinct_values_in_hashset() {
        let mut set: HashSet<RuntimeSessionId> = HashSet::new();
        set.insert(sid("a"));
        set.insert(sid("b"));
        set.insert(sid("c"));
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn hashset_contains_after_insert() {
        let mut set: HashSet<RuntimeSessionId> = HashSet::new();
        let id = sid("present");
        set.insert(id.clone());
        assert!(set.contains(&id));
        assert!(!set.contains(&sid("absent")));
    }

    #[test]
    fn multiple_ids_as_hashmap_keys_are_independent() {
        let mut map: HashMap<RuntimeSessionId, i32> = HashMap::new();
        for i in 0..100 {
            map.insert(sid(&format!("session-{i}")), i);
        }
        assert_eq!(map.len(), 100);
        for i in 0..100 {
            assert_eq!(map[&sid(&format!("session-{i}"))], i);
        }
    }
}
