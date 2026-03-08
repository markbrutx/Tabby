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
    use super::RuntimeSessionId;

    #[test]
    fn display_shows_inner_value() {
        let id = RuntimeSessionId::from(String::from("pty-abc"));
        assert_eq!(id.to_string(), "pty-abc");
    }

    #[test]
    fn from_string() {
        let id = RuntimeSessionId::from(String::from("session-1"));
        assert_eq!(id.as_ref(), "session-1");
    }

    #[test]
    fn equality() {
        let a = RuntimeSessionId::from(String::from("s1"));
        let b = RuntimeSessionId::from(String::from("s1"));
        let c = RuntimeSessionId::from(String::from("s2"));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn clone_is_equal() {
        let id = RuntimeSessionId::from(String::from("s1"));
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn as_ref_returns_inner_str() {
        let id = RuntimeSessionId::from(String::from("browser-xyz"));
        let s: &str = id.as_ref();
        assert_eq!(s, "browser-xyz");
    }

    #[test]
    fn debug_format() {
        let id = RuntimeSessionId::from(String::from("pty-1"));
        let debug = format!("{id:?}");
        assert!(debug.contains("pty-1"));
    }
}
