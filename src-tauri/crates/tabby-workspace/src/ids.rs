use std::fmt;

// Re-export shared value objects from tabby-contracts (shared kernel).
pub use tabby_contracts::{BrowserUrl, PaneId, TabId};

/// Type-safe identifier for a pane's content definition.
/// Each PaneContentId belongs to exactly one PaneSlot — never shared, never reused after destruction.
///
/// This is workspace-specific and does not belong in the shared kernel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneContentId(String);

impl fmt::Display for PaneContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for PaneContentId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for PaneContentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{PaneContentId, PaneId, TabId};

    #[test]
    fn tab_id_display_shows_inner_value() {
        let id = TabId::from(String::from("tab-abc"));
        assert_eq!(id.to_string(), "tab-abc");
    }

    #[test]
    fn pane_id_display_shows_inner_value() {
        let id = PaneId::from(String::from("pane-123"));
        assert_eq!(id.to_string(), "pane-123");
    }

    #[test]
    fn tab_id_from_string() {
        let id = TabId::from(String::from("t1"));
        assert_eq!(id.as_ref(), "t1");
    }

    #[test]
    fn pane_id_from_string() {
        let id = PaneId::from(String::from("p1"));
        assert_eq!(id.as_ref(), "p1");
    }

    #[test]
    fn tab_id_equality() {
        let a = TabId::from(String::from("t1"));
        let b = TabId::from(String::from("t1"));
        let c = TabId::from(String::from("t2"));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn pane_id_equality() {
        let a = PaneId::from(String::from("p1"));
        let b = PaneId::from(String::from("p1"));
        let c = PaneId::from(String::from("p2"));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn tab_id_clone_is_equal() {
        let id = TabId::from(String::from("t1"));
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn pane_id_as_ref_returns_inner_str() {
        let id = PaneId::from(String::from("pane-xyz"));
        let s: &str = id.as_ref();
        assert_eq!(s, "pane-xyz");
    }

    #[test]
    fn tab_id_debug_format() {
        let id = TabId::from(String::from("tab-1"));
        let debug = format!("{id:?}");
        assert!(debug.contains("tab-1"));
    }

    #[test]
    fn pane_id_ordering() {
        let a = PaneId::from(String::from("a"));
        let b = PaneId::from(String::from("b"));
        assert!(a < b);
    }

    #[test]
    fn pane_content_id_display() {
        let id = PaneContentId::from(String::from("content-1"));
        assert_eq!(id.to_string(), "content-1");
    }

    #[test]
    fn pane_content_id_equality() {
        let a = PaneContentId::from(String::from("c1"));
        let b = PaneContentId::from(String::from("c1"));
        assert_eq!(a, b);
    }
}
