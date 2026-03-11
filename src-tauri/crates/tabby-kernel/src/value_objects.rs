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
// CommandTemplate
// ---------------------------------------------------------------------------

/// A validated command template for terminal startup commands or overrides.
///
/// Wraps a non-empty command string. Use `Option<CommandTemplate>` when the
/// command may be absent (e.g. no startup command configured).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandTemplate(String);

impl CommandTemplate {
    /// Creates a new CommandTemplate without validation.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Validated constructor — rejects empty or whitespace-only commands.
    pub fn try_new(value: impl Into<String>) -> Result<Self, ValueObjectError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ValueObjectError::new(
                "CommandTemplate must not be empty or whitespace-only",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommandTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for CommandTemplate {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// LayoutPreset
// ---------------------------------------------------------------------------

/// Compile-time validated layout presets for workspace tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutPreset {
    #[default]
    OneByOne,
    OneByTwo,
    TwoByTwo,
    TwoByThree,
    ThreeByThree,
}

impl LayoutPreset {
    pub fn pane_count(self) -> usize {
        match self {
            Self::OneByOne => 1,
            Self::OneByTwo => 2,
            Self::TwoByTwo => 4,
            Self::TwoByThree => 6,
            Self::ThreeByThree => 9,
        }
    }

    pub fn parse(value: &str) -> Result<Self, ValueObjectError> {
        match value {
            "1x1" => Ok(Self::OneByOne),
            "1x2" => Ok(Self::OneByTwo),
            "2x2" => Ok(Self::TwoByTwo),
            "2x3" => Ok(Self::TwoByThree),
            "3x3" => Ok(Self::ThreeByThree),
            other => Err(ValueObjectError::new(format!(
                "unsupported layout preset: {other}"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneByOne => "1x1",
            Self::OneByTwo => "1x2",
            Self::TwoByTwo => "2x2",
            Self::TwoByThree => "2x3",
            Self::ThreeByThree => "3x3",
        }
    }
}

impl fmt::Display for LayoutPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    // =========================================================================
    // PaneId
    // =========================================================================

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
    fn pane_id_try_new_error_names_type() {
        let err = PaneId::try_new("").unwrap_err();
        assert!(err.to_string().contains("PaneId"));
    }

    #[test]
    fn pane_id_from_string() {
        let id = PaneId::from(String::from("pane-1"));
        assert_eq!(id.to_string(), "pane-1");
    }

    #[test]
    fn pane_id_from_string_allows_empty() {
        // From<String> bypasses validation
        let id = PaneId::from(String::from(""));
        assert_eq!(id.to_string(), "");
    }

    #[test]
    fn pane_id_display_shows_inner_value() {
        let id = PaneId::from(String::from("pane-abc"));
        assert_eq!(id.to_string(), "pane-abc");
    }

    #[test]
    fn pane_id_equality_same_value() {
        let a = PaneId::from(String::from("p1"));
        let b = PaneId::from(String::from("p1"));
        assert_eq!(a, b);
    }

    #[test]
    fn pane_id_inequality_different_value() {
        let a = PaneId::from(String::from("p1"));
        let c = PaneId::from(String::from("p2"));
        assert_ne!(a, c);
    }

    #[test]
    fn pane_id_ordering_less() {
        let a = PaneId::from(String::from("a"));
        let b = PaneId::from(String::from("b"));
        assert!(a < b);
    }

    #[test]
    fn pane_id_ordering_greater() {
        let a = PaneId::from(String::from("z"));
        let b = PaneId::from(String::from("a"));
        assert!(a > b);
    }

    #[test]
    fn pane_id_ordering_equal() {
        let a = PaneId::from(String::from("x"));
        let b = PaneId::from(String::from("x"));
        assert!(a <= b);
        assert!(a >= b);
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

    #[test]
    fn pane_id_clone_is_independent() {
        let id = PaneId::try_new("p1").unwrap();
        let cloned = id.clone();
        // They are equal but independent values
        assert_eq!(id.as_ref(), cloned.as_ref());
    }

    #[test]
    fn pane_id_hash_in_hashset() {
        let mut set = HashSet::new();
        let a = PaneId::from(String::from("p1"));
        let b = PaneId::from(String::from("p1"));
        let c = PaneId::from(String::from("p2"));
        set.insert(a);
        set.insert(b); // duplicate, should not increase size
        set.insert(c);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn pane_id_as_hashmap_key() {
        let mut map: HashMap<PaneId, &str> = HashMap::new();
        let id = PaneId::try_new("pane-key").unwrap();
        map.insert(id.clone(), "value");
        assert_eq!(map[&id], "value");
    }

    #[test]
    fn pane_id_try_new_whitespace_is_valid() {
        // Whitespace is not blocked by id_newtype! (only empty is blocked)
        let id = PaneId::try_new("  ").expect("whitespace is non-empty");
        assert_eq!(id.as_ref(), "  ");
    }

    #[test]
    fn pane_id_try_new_special_chars() {
        let id = PaneId::try_new("pane_✨_🚀").expect("special chars allowed");
        assert_eq!(id.as_ref(), "pane_✨_🚀");
    }

    // =========================================================================
    // TabId
    // =========================================================================

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
    fn tab_id_try_new_error_names_type() {
        let err = TabId::try_new("").unwrap_err();
        assert!(err.to_string().contains("TabId"));
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
    fn tab_id_equality_same_value() {
        let a = TabId::from(String::from("t1"));
        let b = TabId::from(String::from("t1"));
        assert_eq!(a, b);
    }

    #[test]
    fn tab_id_inequality_different_value() {
        let a = TabId::from(String::from("t1"));
        let c = TabId::from(String::from("t2"));
        assert_ne!(a, c);
    }

    #[test]
    fn tab_id_ordering() {
        let a = TabId::from(String::from("alpha"));
        let b = TabId::from(String::from("beta"));
        assert!(a < b);
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

    #[test]
    fn tab_id_as_ref_returns_inner_str() {
        let id = TabId::from(String::from("tab-xyz"));
        let s: &str = id.as_ref();
        assert_eq!(s, "tab-xyz");
    }

    #[test]
    fn tab_id_hash_in_hashset() {
        let mut set = HashSet::new();
        let a = TabId::try_new("t1").unwrap();
        let b = TabId::try_new("t1").unwrap();
        let c = TabId::try_new("t2").unwrap();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn tab_id_as_hashmap_key() {
        let mut map: HashMap<TabId, i32> = HashMap::new();
        let id = TabId::try_new("tab-key").unwrap();
        map.insert(id.clone(), 42);
        assert_eq!(map[&id], 42);
    }

    #[test]
    fn tab_id_from_string_allows_empty() {
        let id = TabId::from(String::from(""));
        assert_eq!(id.to_string(), "");
    }

    // =========================================================================
    // id_newtype! macro — cross-cutting checks
    // =========================================================================

    #[test]
    fn id_newtype_pane_and_tab_are_distinct_types() {
        // Compile-time check: PaneId and TabId are different types.
        // This test simply exercises both to confirm no type confusion.
        let pane = PaneId::try_new("same-value").unwrap();
        let tab = TabId::try_new("same-value").unwrap();
        assert_eq!(pane.as_ref(), tab.as_ref());
        // The fact that this compiles without coercion proves they are separate types.
    }

    #[test]
    fn id_newtype_try_new_returns_ok_for_single_char() {
        assert!(PaneId::try_new("x").is_ok());
        assert!(TabId::try_new("x").is_ok());
    }

    #[test]
    fn id_newtype_try_new_returns_err_for_empty() {
        assert!(PaneId::try_new("").is_err());
        assert!(TabId::try_new("").is_err());
    }

    #[test]
    fn id_newtype_sorted_order_matches_lexicographic() {
        let mut ids = vec![
            PaneId::try_new("c").unwrap(),
            PaneId::try_new("a").unwrap(),
            PaneId::try_new("b").unwrap(),
        ];
        ids.sort();
        let strs: Vec<&str> = ids.iter().map(|id| id.as_ref()).collect();
        assert_eq!(strs, vec!["a", "b", "c"]);
    }

    // =========================================================================
    // BrowserUrl
    // =========================================================================

    #[test]
    fn browser_url_new_accepts_any_value() {
        let url = BrowserUrl::new("https://example.com");
        assert_eq!(url.as_str(), "https://example.com");
    }

    #[test]
    fn browser_url_new_accepts_empty() {
        // new() has no validation
        let url = BrowserUrl::new("");
        assert_eq!(url.as_str(), "");
    }

    #[test]
    fn browser_url_try_new_accepts_valid_http() {
        let url = BrowserUrl::try_new("http://example.com").expect("http should be valid");
        assert_eq!(url.as_str(), "http://example.com");
    }

    #[test]
    fn browser_url_try_new_accepts_valid_https() {
        let url = BrowserUrl::try_new("https://example.com").expect("https should be valid");
        assert_eq!(url.as_str(), "https://example.com");
    }

    #[test]
    fn browser_url_try_new_accepts_localhost() {
        let url = BrowserUrl::try_new("http://localhost:3000").unwrap();
        assert_eq!(url.as_str(), "http://localhost:3000");
    }

    #[test]
    fn browser_url_try_new_accepts_file_protocol() {
        let url = BrowserUrl::try_new("file:///home/user/index.html").unwrap();
        assert_eq!(url.as_str(), "file:///home/user/index.html");
    }

    #[test]
    fn browser_url_try_new_accepts_special_characters_in_path() {
        let url = BrowserUrl::try_new("https://example.com/path?q=hello%20world&lang=en").unwrap();
        assert_eq!(url.as_str(), "https://example.com/path?q=hello%20world&lang=en");
    }

    #[test]
    fn browser_url_try_new_accepts_unicode_domain() {
        let url = BrowserUrl::try_new("https://münchen.de").unwrap();
        assert_eq!(url.as_str(), "https://münchen.de");
    }

    #[test]
    fn browser_url_try_new_accepts_very_long_url() {
        let long = format!("https://example.com/{}", "a".repeat(2000));
        let url = BrowserUrl::try_new(&long).unwrap();
        assert_eq!(url.as_str(), long);
    }

    #[test]
    fn browser_url_try_new_rejects_empty() {
        let err = BrowserUrl::try_new("").expect_err("should reject empty");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn browser_url_try_new_rejects_single_space() {
        let err = BrowserUrl::try_new(" ").expect_err("should reject single space");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn browser_url_try_new_rejects_tabs_and_newlines() {
        let err = BrowserUrl::try_new("\t\n").expect_err("should reject whitespace-only");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn browser_url_try_new_rejects_whitespace_only() {
        let err = BrowserUrl::try_new("   ").expect_err("should reject whitespace-only");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn browser_url_display_matches_as_str() {
        let url = BrowserUrl::new("https://tabby.dev");
        assert_eq!(url.to_string(), "https://tabby.dev");
        assert_eq!(url.to_string(), url.as_str());
    }

    #[test]
    fn browser_url_as_ref_matches_as_str() {
        let url = BrowserUrl::new("https://tabby.dev");
        assert_eq!(url.as_ref(), url.as_str());
    }

    #[test]
    fn browser_url_equality_same_value() {
        let a = BrowserUrl::new("https://example.com");
        let b = BrowserUrl::new("https://example.com");
        assert_eq!(a, b);
    }

    #[test]
    fn browser_url_inequality_different_value() {
        let a = BrowserUrl::new("https://example.com");
        let b = BrowserUrl::new("https://other.com");
        assert_ne!(a, b);
    }

    #[test]
    fn browser_url_clone_is_equal() {
        let url = BrowserUrl::new("https://clone-test.com");
        let cloned = url.clone();
        assert_eq!(url, cloned);
    }

    #[test]
    fn browser_url_debug_contains_value() {
        let url = BrowserUrl::new("https://debug-test.com");
        let debug = format!("{url:?}");
        assert!(debug.contains("debug-test.com"));
    }

    // =========================================================================
    // WorkingDirectory
    // =========================================================================

    #[test]
    fn working_directory_accepts_absolute_unix_path() {
        let wd = WorkingDirectory::new("/home/user/projects").expect("should be valid");
        assert_eq!(wd.as_str(), "/home/user/projects");
    }

    #[test]
    fn working_directory_accepts_relative_path() {
        let wd = WorkingDirectory::new("relative/path").expect("relative paths are allowed");
        assert_eq!(wd.as_str(), "relative/path");
    }

    #[test]
    fn working_directory_accepts_trailing_slash() {
        let wd = WorkingDirectory::new("/usr/local/").expect("trailing slash allowed");
        assert_eq!(wd.as_str(), "/usr/local/");
    }

    #[test]
    fn working_directory_accepts_empty_string() {
        let wd = WorkingDirectory::new("").expect("empty should be valid");
        assert!(wd.is_empty());
        assert_eq!(wd.as_str(), "");
    }

    #[test]
    fn working_directory_rejects_null_bytes() {
        let err = WorkingDirectory::new("/home/\0bad").expect_err("null bytes should be rejected");
        assert!(err.to_string().contains("null bytes"));
    }

    #[test]
    fn working_directory_rejects_null_byte_at_start() {
        let err = WorkingDirectory::new("\0/bad").unwrap_err();
        assert!(err.to_string().contains("null bytes"));
    }

    #[test]
    fn working_directory_rejects_null_byte_at_end() {
        let err = WorkingDirectory::new("/path/to/dir\0").unwrap_err();
        assert!(err.to_string().contains("null bytes"));
    }

    #[test]
    fn working_directory_accepts_tilde() {
        let wd = WorkingDirectory::new("~").expect("tilde should be valid");
        assert_eq!(wd.as_str(), "~");
    }

    #[test]
    fn working_directory_accepts_tilde_expansion_style() {
        let wd = WorkingDirectory::new("~/projects").expect("~/path should be valid");
        assert_eq!(wd.as_str(), "~/projects");
    }

    #[test]
    fn working_directory_accepts_spaces_in_path() {
        let wd = WorkingDirectory::new("/home/my projects/code").expect("spaces should be valid");
        assert_eq!(wd.as_str(), "/home/my projects/code");
    }

    #[test]
    fn working_directory_accepts_unicode_chars() {
        let wd = WorkingDirectory::new("/home/ユーザー/プロジェクト").unwrap();
        assert_eq!(wd.as_str(), "/home/ユーザー/プロジェクト");
    }

    #[test]
    fn working_directory_accepts_dot_paths() {
        let wd = WorkingDirectory::new("./relative/to/cwd").unwrap();
        assert_eq!(wd.as_str(), "./relative/to/cwd");
    }

    #[test]
    fn working_directory_accepts_parent_dir_notation() {
        let wd = WorkingDirectory::new("../../parent").unwrap();
        assert_eq!(wd.as_str(), "../../parent");
    }

    #[test]
    fn working_directory_empty_constructor() {
        let wd = WorkingDirectory::empty();
        assert!(wd.is_empty());
        assert_eq!(wd.as_str(), "");
    }

    #[test]
    fn working_directory_non_empty_is_not_empty() {
        let wd = WorkingDirectory::new("/tmp").unwrap();
        assert!(!wd.is_empty());
    }

    #[test]
    fn working_directory_empty_equals_default() {
        let wd = WorkingDirectory::empty();
        assert_eq!(wd, WorkingDirectory::default());
    }

    #[test]
    fn working_directory_default_is_empty() {
        let wd = WorkingDirectory::default();
        assert!(wd.is_empty());
    }

    #[test]
    fn working_directory_equality_same_path() {
        let a = WorkingDirectory::new("/usr/local").unwrap();
        let b = WorkingDirectory::new("/usr/local").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn working_directory_inequality_different_path() {
        let a = WorkingDirectory::new("/usr").unwrap();
        let b = WorkingDirectory::new("/tmp").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn working_directory_display() {
        let wd = WorkingDirectory::new("/tmp").expect("valid");
        assert_eq!(wd.to_string(), "/tmp");
    }

    #[test]
    fn working_directory_display_empty() {
        let wd = WorkingDirectory::empty();
        assert_eq!(wd.to_string(), "");
    }

    #[test]
    fn working_directory_clone_is_equal() {
        let wd = WorkingDirectory::new("/clone/me").unwrap();
        let cloned = wd.clone();
        assert_eq!(wd, cloned);
    }

    #[test]
    fn working_directory_debug_contains_path() {
        let wd = WorkingDirectory::new("/debug/path").unwrap();
        let debug = format!("{wd:?}");
        assert!(debug.contains("/debug/path"));
    }

    // =========================================================================
    // CommandTemplate
    // =========================================================================

    #[test]
    fn command_template_new_accepts_any_value() {
        let cmd = CommandTemplate::new("claude");
        assert_eq!(cmd.as_str(), "claude");
    }

    #[test]
    fn command_template_new_accepts_empty() {
        // new() has no validation
        let cmd = CommandTemplate::new("");
        assert_eq!(cmd.as_str(), "");
    }

    #[test]
    fn command_template_try_new_accepts_valid_simple() {
        let cmd = CommandTemplate::try_new("bash").expect("should accept non-empty");
        assert_eq!(cmd.as_str(), "bash");
    }

    #[test]
    fn command_template_try_new_accepts_command_with_args() {
        let cmd = CommandTemplate::try_new("vim --noplugin -u NONE").unwrap();
        assert_eq!(cmd.as_str(), "vim --noplugin -u NONE");
    }

    #[test]
    fn command_template_try_new_accepts_leading_whitespace_around_content() {
        // Leading/trailing whitespace is acceptable as long as the trimmed value is non-empty
        let cmd = CommandTemplate::try_new("  bash  ").unwrap();
        assert_eq!(cmd.as_str(), "  bash  ");
    }

    #[test]
    fn command_template_try_new_accepts_path_with_args() {
        let cmd = CommandTemplate::try_new("/usr/local/bin/fish --login").unwrap();
        assert_eq!(cmd.as_str(), "/usr/local/bin/fish --login");
    }

    #[test]
    fn command_template_try_new_accepts_unicode_command() {
        let cmd = CommandTemplate::try_new("echo 'こんにちは'").unwrap();
        assert_eq!(cmd.as_str(), "echo 'こんにちは'");
    }

    #[test]
    fn command_template_try_new_rejects_empty() {
        let err = CommandTemplate::try_new("").expect_err("should reject empty");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn command_template_try_new_rejects_single_space() {
        let err = CommandTemplate::try_new(" ").expect_err("single space should be rejected");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn command_template_try_new_rejects_tabs_only() {
        let err = CommandTemplate::try_new("\t\t").expect_err("tabs-only should be rejected");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn command_template_try_new_rejects_whitespace_only() {
        let err = CommandTemplate::try_new("   ").expect_err("should reject whitespace-only");
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn command_template_display_matches_as_str() {
        let cmd = CommandTemplate::new("vim --noplugin");
        assert_eq!(cmd.to_string(), cmd.as_str());
    }

    #[test]
    fn command_template_as_ref_matches_as_str() {
        let cmd = CommandTemplate::new("vim --noplugin");
        assert_eq!(cmd.as_ref(), cmd.as_str());
    }

    #[test]
    fn command_template_equality_same_value() {
        let a = CommandTemplate::new("bash");
        let b = CommandTemplate::new("bash");
        assert_eq!(a, b);
    }

    #[test]
    fn command_template_inequality_different_value() {
        let a = CommandTemplate::new("bash");
        let b = CommandTemplate::new("zsh");
        assert_ne!(a, b);
    }

    #[test]
    fn command_template_clone_is_equal() {
        let cmd = CommandTemplate::new("fish --login");
        let cloned = cmd.clone();
        assert_eq!(cmd, cloned);
    }

    #[test]
    fn command_template_debug_contains_value() {
        let cmd = CommandTemplate::new("debug-cmd");
        let debug = format!("{cmd:?}");
        assert!(debug.contains("debug-cmd"));
    }

    // =========================================================================
    // LayoutPreset
    // =========================================================================

    #[test]
    fn layout_preset_parse_one_by_one() {
        assert_eq!(LayoutPreset::parse("1x1").unwrap(), LayoutPreset::OneByOne);
    }

    #[test]
    fn layout_preset_parse_one_by_two() {
        assert_eq!(LayoutPreset::parse("1x2").unwrap(), LayoutPreset::OneByTwo);
    }

    #[test]
    fn layout_preset_parse_two_by_two() {
        assert_eq!(LayoutPreset::parse("2x2").unwrap(), LayoutPreset::TwoByTwo);
    }

    #[test]
    fn layout_preset_parse_two_by_three() {
        assert_eq!(
            LayoutPreset::parse("2x3").unwrap(),
            LayoutPreset::TwoByThree
        );
    }

    #[test]
    fn layout_preset_parse_three_by_three() {
        assert_eq!(
            LayoutPreset::parse("3x3").unwrap(),
            LayoutPreset::ThreeByThree
        );
    }

    #[test]
    fn layout_preset_parse_rejects_unknown() {
        let err = LayoutPreset::parse("4x4").expect_err("should reject unknown preset");
        assert!(err.to_string().contains("unsupported layout preset"));
    }

    #[test]
    fn layout_preset_parse_rejects_empty_string() {
        let err = LayoutPreset::parse("").unwrap_err();
        assert!(err.to_string().contains("unsupported layout preset"));
    }

    #[test]
    fn layout_preset_parse_rejects_uppercase() {
        let err = LayoutPreset::parse("1X1").unwrap_err();
        assert!(err.to_string().contains("unsupported layout preset"));
    }

    #[test]
    fn layout_preset_parse_rejects_partial_match() {
        let err = LayoutPreset::parse("1x").unwrap_err();
        assert!(err.to_string().contains("unsupported layout preset"));
    }

    #[test]
    fn layout_preset_parse_error_contains_input() {
        let err = LayoutPreset::parse("bogus").unwrap_err();
        assert!(err.to_string().contains("bogus"));
    }

    #[test]
    fn layout_preset_as_str_round_trips_all_variants() {
        let presets = [
            LayoutPreset::OneByOne,
            LayoutPreset::OneByTwo,
            LayoutPreset::TwoByTwo,
            LayoutPreset::TwoByThree,
            LayoutPreset::ThreeByThree,
        ];
        for preset in presets {
            let parsed = LayoutPreset::parse(preset.as_str()).unwrap();
            assert_eq!(parsed, preset);
        }
    }

    #[test]
    fn layout_preset_pane_count_one_by_one() {
        assert_eq!(LayoutPreset::OneByOne.pane_count(), 1);
    }

    #[test]
    fn layout_preset_pane_count_one_by_two() {
        assert_eq!(LayoutPreset::OneByTwo.pane_count(), 2);
    }

    #[test]
    fn layout_preset_pane_count_two_by_two() {
        assert_eq!(LayoutPreset::TwoByTwo.pane_count(), 4);
    }

    #[test]
    fn layout_preset_pane_count_two_by_three() {
        assert_eq!(LayoutPreset::TwoByThree.pane_count(), 6);
    }

    #[test]
    fn layout_preset_pane_count_three_by_three() {
        assert_eq!(LayoutPreset::ThreeByThree.pane_count(), 9);
    }

    #[test]
    fn layout_preset_display_all_variants() {
        assert_eq!(LayoutPreset::OneByOne.to_string(), "1x1");
        assert_eq!(LayoutPreset::OneByTwo.to_string(), "1x2");
        assert_eq!(LayoutPreset::TwoByTwo.to_string(), "2x2");
        assert_eq!(LayoutPreset::TwoByThree.to_string(), "2x3");
        assert_eq!(LayoutPreset::ThreeByThree.to_string(), "3x3");
    }

    #[test]
    fn layout_preset_as_str_matches_display() {
        for preset in [
            LayoutPreset::OneByOne,
            LayoutPreset::OneByTwo,
            LayoutPreset::TwoByTwo,
            LayoutPreset::TwoByThree,
            LayoutPreset::ThreeByThree,
        ] {
            assert_eq!(preset.as_str(), preset.to_string());
        }
    }

    #[test]
    fn layout_preset_default_is_one_by_one() {
        assert_eq!(LayoutPreset::default(), LayoutPreset::OneByOne);
    }

    #[test]
    fn layout_preset_equality_same_variant() {
        assert_eq!(LayoutPreset::TwoByTwo, LayoutPreset::TwoByTwo);
    }

    #[test]
    fn layout_preset_inequality_different_variants() {
        assert_ne!(LayoutPreset::OneByOne, LayoutPreset::ThreeByThree);
    }

    #[test]
    fn layout_preset_clone_is_equal() {
        let preset = LayoutPreset::TwoByThree;
        let cloned = preset; // Copy type, no explicit clone needed
        assert_eq!(preset, cloned);
    }

    #[test]
    fn layout_preset_copy_semantics() {
        let preset = LayoutPreset::ThreeByThree;
        let copied = preset; // LayoutPreset: Copy
        assert_eq!(preset, copied);
    }

    #[test]
    fn layout_preset_debug_format() {
        let debug = format!("{:?}", LayoutPreset::TwoByTwo);
        assert!(!debug.is_empty());
    }

    // =========================================================================
    // ValueObjectError
    // =========================================================================

    #[test]
    fn value_object_error_new_and_display() {
        let err = ValueObjectError::new("test error message");
        assert_eq!(err.to_string(), "test error message");
    }

    #[test]
    fn value_object_error_from_string_type() {
        let err = ValueObjectError::new(String::from("dynamic error"));
        assert_eq!(err.to_string(), "dynamic error");
    }

    #[test]
    fn value_object_error_equality_same_message() {
        let a = ValueObjectError::new("same");
        let b = ValueObjectError::new("same");
        assert_eq!(a, b);
    }

    #[test]
    fn value_object_error_inequality_different_message() {
        let a = ValueObjectError::new("foo");
        let b = ValueObjectError::new("bar");
        assert_ne!(a, b);
    }

    #[test]
    fn value_object_error_clone_is_equal() {
        let err = ValueObjectError::new("clone me");
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn value_object_error_debug_format() {
        let err = ValueObjectError::new("debug error");
        let debug = format!("{err:?}");
        assert!(debug.contains("debug error"));
    }

    #[test]
    fn value_object_error_implements_std_error() {
        let err: &dyn std::error::Error = &ValueObjectError::new("std error");
        assert_eq!(err.to_string(), "std error");
    }

    #[test]
    fn value_object_error_empty_message() {
        let err = ValueObjectError::new("");
        assert_eq!(err.to_string(), "");
    }

    #[test]
    fn value_object_error_long_message() {
        let msg = "x".repeat(1000);
        let err = ValueObjectError::new(&msg);
        assert_eq!(err.to_string(), msg);
    }
}
