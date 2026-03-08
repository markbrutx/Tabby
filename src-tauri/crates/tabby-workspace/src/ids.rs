use std::fmt;

macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

id_newtype!(
    /// Type-safe identifier for a workspace tab.
    TabId
);

id_newtype!(
    /// Type-safe identifier for a pane within a tab.
    PaneId
);

id_newtype!(
    /// Type-safe identifier for a pane's content definition.
    /// Each PaneContentId belongs to exactly one PaneSlot — never shared, never reused after destruction.
    PaneContentId
);

#[cfg(test)]
mod tests {
    use super::{PaneId, TabId};

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
}
