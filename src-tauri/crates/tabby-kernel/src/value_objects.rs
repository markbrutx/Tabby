use std::fmt;

/// Error type for value object construction validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueObjectError(String);

impl ValueObjectError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ValueObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ValueObjectError {}

// ---------------------------------------------------------------------------
// ID newtypes macro
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// Validated constructor — rejects empty identifiers.
            pub fn try_new(value: impl Into<String>) -> Result<Self, $crate::ValueObjectError> {
                let value = value.into();
                if value.is_empty() {
                    return Err($crate::ValueObjectError::new(concat!(
                        stringify!($name),
                        " must not be empty"
                    )));
                }
                Ok(Self(value))
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
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

// ---------------------------------------------------------------------------
// BrowserUrl
// ---------------------------------------------------------------------------

/// A URL for browser pane content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserUrl(String);

impl BrowserUrl {
    /// Creates a new BrowserUrl without validation.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Validated constructor — rejects empty URLs.
    pub fn try_new(value: impl Into<String>) -> Result<Self, ValueObjectError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ValueObjectError::new("BrowserUrl must not be empty"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BrowserUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for BrowserUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// WorkingDirectory
// ---------------------------------------------------------------------------

/// A working directory path. Empty means "not configured".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingDirectory(String);

impl WorkingDirectory {
    /// Validated constructor — rejects null bytes.
    pub fn new(value: impl Into<String>) -> Result<Self, ValueObjectError> {
        let value = value.into();
        if value.contains('\0') {
            return Err(ValueObjectError::new(
                "Working directory must not contain null bytes",
            ));
        }
        Ok(Self(value))
    }

    pub fn empty() -> Self {
        Self(String::new())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for WorkingDirectory {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for WorkingDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- PaneId ------------------------------------------------------------

    #[test]
    fn pane_id_try_new_accepts_valid_value() {
        let id = PaneId::try_new("pane-123").expect("should accept non-empty");
        assert_eq!(id.as_ref(), "pane-123");
    }

    #[test]
    fn pane_id_try_new_rejects_empty() {
        let err = PaneId::try_new("").expect_err("should reject empty");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn pane_id_from_string() {
        let id = PaneId::from(String::from("pane-1"));
        assert_eq!(id.to_string(), "pane-1");
    }

    #[test]
    fn pane_id_display_shows_inner_value() {
        let id = PaneId::from(String::from("pane-abc"));
        assert_eq!(id.to_string(), "pane-abc");
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
    fn pane_id_ordering() {
        let a = PaneId::from(String::from("a"));
        let b = PaneId::from(String::from("b"));
        assert!(a < b);
    }

    #[test]
    fn pane_id_as_ref_returns_inner_str() {
        let id = PaneId::from(String::from("pane-xyz"));
        let s: &str = id.as_ref();
        assert_eq!(s, "pane-xyz");
    }

    #[test]
    fn pane_id_debug_format() {
        let id = PaneId::from(String::from("pane-1"));
        let debug = format!("{id:?}");
        assert!(debug.contains("pane-1"));
    }

    #[test]
    fn pane_id_clone_is_equal() {
        let id = PaneId::from(String::from("p1"));
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    // -- TabId -------------------------------------------------------------

    #[test]
    fn tab_id_try_new_accepts_valid_value() {
        let id = TabId::try_new("tab-1").expect("should accept non-empty");
        assert_eq!(id.as_ref(), "tab-1");
    }

    #[test]
    fn tab_id_try_new_rejects_empty() {
        let err = TabId::try_new("").expect_err("should reject empty");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn tab_id_from_string() {
        let id = TabId::from(String::from("tab-abc"));
        assert_eq!(id.to_string(), "tab-abc");
    }

    #[test]
    fn tab_id_display_shows_inner_value() {
        let id = TabId::from(String::from("tab-abc"));
        assert_eq!(id.to_string(), "tab-abc");
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
    fn tab_id_clone_is_equal() {
        let id = TabId::from(String::from("t1"));
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn tab_id_debug_format() {
        let id = TabId::from(String::from("tab-1"));
        let debug = format!("{id:?}");
        assert!(debug.contains("tab-1"));
    }

    // -- BrowserUrl --------------------------------------------------------

    #[test]
    fn browser_url_new_accepts_any_value() {
        let url = BrowserUrl::new("https://example.com");
        assert_eq!(url.as_str(), "https://example.com");
    }

    #[test]
    fn browser_url_try_new_accepts_valid() {
        let url = BrowserUrl::try_new("https://example.com").expect("should accept non-empty");
        assert_eq!(url.as_str(), "https://example.com");
    }

    #[test]
    fn browser_url_try_new_rejects_empty() {
        let err = BrowserUrl::try_new("").expect_err("should reject empty");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn browser_url_try_new_rejects_whitespace_only() {
        let err = BrowserUrl::try_new("   ").expect_err("should reject whitespace-only");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn browser_url_display_and_as_ref() {
        let url = BrowserUrl::new("https://tabby.dev");
        assert_eq!(url.to_string(), "https://tabby.dev");
        assert_eq!(url.as_ref(), "https://tabby.dev");
        assert_eq!(url.as_str(), "https://tabby.dev");
    }

    // -- WorkingDirectory --------------------------------------------------

    #[test]
    fn working_directory_accepts_valid_path() {
        let wd = WorkingDirectory::new("/home/user").expect("should be valid");
        assert_eq!(wd.as_str(), "/home/user");
    }

    #[test]
    fn working_directory_accepts_empty_string() {
        let wd = WorkingDirectory::new("").expect("empty should be valid");
        assert!(wd.is_empty());
    }

    #[test]
    fn working_directory_rejects_null_bytes() {
        let err = WorkingDirectory::new("/home/\0bad").expect_err("null bytes should be rejected");
        assert!(err.to_string().contains("null bytes"));
    }

    #[test]
    fn working_directory_accepts_tilde() {
        let wd = WorkingDirectory::new("~").expect("tilde should be valid");
        assert_eq!(wd.as_str(), "~");
    }

    #[test]
    fn working_directory_accepts_spaces_in_path() {
        let wd = WorkingDirectory::new("/home/my projects").expect("spaces should be valid");
        assert_eq!(wd.as_str(), "/home/my projects");
    }

    #[test]
    fn working_directory_empty_is_default() {
        let wd = WorkingDirectory::empty();
        assert!(wd.is_empty());
        assert_eq!(wd, WorkingDirectory::default());
    }

    #[test]
    fn working_directory_display() {
        let wd = WorkingDirectory::new("/tmp").expect("valid");
        assert_eq!(wd.to_string(), "/tmp");
    }

    // -- ValueObjectError --------------------------------------------------

    #[test]
    fn value_object_error_display() {
        let err = ValueObjectError::new("test error");
        assert_eq!(err.to_string(), "test error");
    }
}
