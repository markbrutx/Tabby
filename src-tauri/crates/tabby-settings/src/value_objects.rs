use crate::SettingsError;

// Re-export WorkingDirectory from the shared kernel (tabby-kernel).
pub use tabby_kernel::WorkingDirectory;

/// Font size in points, validated to be within 8–72.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontSize(u16);

impl FontSize {
    pub const MIN: u16 = 8;
    pub const MAX: u16 = 72;

    pub fn new(value: u16) -> Result<Self, SettingsError> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(SettingsError::Validation(format!(
                "Font size must be between {} and {}, got {}",
                Self::MIN,
                Self::MAX,
                value
            )));
        }
        Ok(Self(value))
    }

    pub fn value(self) -> u16 {
        self.0
    }
}

impl Default for FontSize {
    fn default() -> Self {
        Self(13)
    }
}

impl std::fmt::Display for FontSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifies a terminal launch profile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileId(String);

impl ProfileId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProfileId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for ProfileId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ProfileId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl std::fmt::Display for ProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- FontSize ---------------------------------------------------------

    #[test]
    fn font_size_accepts_minimum() {
        let fs = FontSize::new(8).expect("min should be valid");
        assert_eq!(fs.value(), 8);
    }

    #[test]
    fn font_size_accepts_maximum() {
        let fs = FontSize::new(72).expect("max should be valid");
        assert_eq!(fs.value(), 72);
    }

    #[test]
    fn font_size_accepts_typical_value() {
        let fs = FontSize::new(14).expect("14 should be valid");
        assert_eq!(fs.value(), 14);
    }

    #[test]
    fn font_size_rejects_below_minimum() {
        let err = FontSize::new(6).expect_err("6 should be rejected");
        assert!(err.to_string().contains("between 8 and 72"));
        assert!(err.to_string().contains("got 6"));
    }

    #[test]
    fn font_size_rejects_zero() {
        assert!(FontSize::new(0).is_err());
    }

    #[test]
    fn font_size_rejects_above_maximum() {
        let err = FontSize::new(73).expect_err("73 should be rejected");
        assert!(err.to_string().contains("between 8 and 72"));
    }

    #[test]
    fn font_size_default_is_13() {
        assert_eq!(FontSize::default().value(), 13);
    }

    #[test]
    fn font_size_display() {
        assert_eq!(FontSize::new(16).unwrap().to_string(), "16");
    }

    // -- WorkingDirectory (imported from tabby-kernel) ---------------------

    #[test]
    fn working_directory_accepts_valid_path() {
        let wd = WorkingDirectory::new("/home/user").expect("should be valid");
        assert_eq!(wd.as_str(), "/home/user");
    }

    #[test]
    fn working_directory_accepts_empty() {
        let wd = WorkingDirectory::empty();
        assert!(wd.is_empty());
        assert_eq!(wd.as_str(), "");
    }

    #[test]
    fn working_directory_new_accepts_empty_string() {
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

    // -- ProfileId --------------------------------------------------------

    #[test]
    fn profile_id_new_from_str() {
        let id = ProfileId::new("terminal");
        assert_eq!(id.as_str(), "terminal");
    }

    #[test]
    fn profile_id_new_from_string() {
        let id = ProfileId::new(String::from("claude"));
        assert_eq!(id.as_str(), "claude");
    }

    #[test]
    fn profile_id_equality_with_str() {
        let id = ProfileId::new("terminal");
        assert!(id == "terminal");
        assert!(id != "claude");
    }

    #[test]
    fn profile_id_equality_with_str_ref() {
        let id = ProfileId::new("terminal");
        let s: &str = "terminal";
        assert!(id == s);
    }

    #[test]
    fn profile_id_display() {
        assert_eq!(ProfileId::new("codex").to_string(), "codex");
    }

    #[test]
    fn profile_id_as_ref() {
        let id = ProfileId::new("terminal");
        let s: &str = id.as_ref();
        assert_eq!(s, "terminal");
    }

    #[test]
    fn profile_id_clone_is_equal() {
        let id = ProfileId::new("claude");
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn profile_id_debug_format() {
        let id = ProfileId::new("codex");
        let debug = format!("{id:?}");
        assert!(debug.contains("codex"));
    }

    #[test]
    fn profile_id_hash_equal_ids_have_equal_hashes() {
        use std::collections::HashSet;
        let id1 = ProfileId::new("terminal");
        let id2 = ProfileId::new("terminal");
        let mut set = HashSet::new();
        set.insert(id1);
        // Inserting an equal id should not increase the set size
        let inserted = set.insert(id2);
        assert!(!inserted, "duplicate id should not be inserted");
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn profile_id_used_as_hashmap_key() {
        use std::collections::HashMap;
        let mut map: HashMap<ProfileId, &str> = HashMap::new();
        let id = ProfileId::new("terminal");
        map.insert(id.clone(), "Terminal Profile");
        assert_eq!(map[&id], "Terminal Profile");
    }

    #[test]
    fn profile_id_empty_string_is_allowed() {
        // ProfileId has no validation — empty strings are permitted
        let id = ProfileId::new("");
        assert_eq!(id.as_str(), "");
    }

    #[test]
    fn profile_id_with_special_characters() {
        let id = ProfileId::new("my-profile/v2 (beta)");
        assert_eq!(id.as_str(), "my-profile/v2 (beta)");
    }

    // -- FontSize additional edge cases -----------------------------------

    #[test]
    fn font_size_copy_semantics() {
        let fs = FontSize::new(16).unwrap();
        let copied = fs; // Copy, not move
        assert_eq!(fs.value(), copied.value());
    }

    #[test]
    fn font_size_copy_is_equal() {
        let fs = FontSize::new(20).unwrap();
        let copied = fs;
        assert_eq!(fs, copied);
    }

    #[test]
    fn font_size_equality() {
        let a = FontSize::new(14).unwrap();
        let b = FontSize::new(14).unwrap();
        let c = FontSize::new(16).unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn font_size_debug_format() {
        let fs = FontSize::new(14).unwrap();
        let debug = format!("{fs:?}");
        assert!(debug.contains("14"));
    }

    #[test]
    fn font_size_min_constant_is_8() {
        assert_eq!(FontSize::MIN, 8);
    }

    #[test]
    fn font_size_max_constant_is_72() {
        assert_eq!(FontSize::MAX, 72);
    }

    #[test]
    fn font_size_boundary_below_min_is_7() {
        assert!(FontSize::new(7).is_err());
    }

    #[test]
    fn font_size_boundary_above_max_is_73() {
        assert!(FontSize::new(73).is_err());
    }

    #[test]
    fn font_size_error_message_contains_actual_value() {
        let err = FontSize::new(5).unwrap_err();
        assert!(err.to_string().contains("got 5"));
    }

    #[test]
    fn font_size_error_message_contains_bounds() {
        let err = FontSize::new(100).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("8"));
        assert!(msg.contains("72"));
    }

    // -- WorkingDirectory additional tests --------------------------------

    #[test]
    fn working_directory_clone_is_equal() {
        let wd = WorkingDirectory::new("/home/user").unwrap();
        let cloned = wd.clone();
        assert_eq!(wd, cloned);
    }

    #[test]
    fn working_directory_display() {
        let wd = WorkingDirectory::new("/tmp/test").unwrap();
        assert_eq!(wd.to_string(), "/tmp/test");
    }

    #[test]
    fn working_directory_default_is_empty() {
        let wd = WorkingDirectory::default();
        assert!(wd.is_empty());
        assert_eq!(wd.as_str(), "");
    }

    #[test]
    fn working_directory_with_unicode_path() {
        let wd = WorkingDirectory::new("/home/用户/文档").unwrap();
        assert_eq!(wd.as_str(), "/home/用户/文档");
    }

    #[test]
    fn working_directory_debug_format() {
        let wd = WorkingDirectory::new("/tmp").unwrap();
        let debug = format!("{wd:?}");
        assert!(debug.contains("tmp"));
    }

    #[test]
    fn working_directory_equality() {
        let a = WorkingDirectory::new("/home").unwrap();
        let b = WorkingDirectory::new("/home").unwrap();
        let c = WorkingDirectory::new("/tmp").unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn working_directory_empty_vs_non_empty() {
        let empty = WorkingDirectory::empty();
        let non_empty = WorkingDirectory::new("/home").unwrap();
        assert_ne!(empty, non_empty);
        assert!(empty.is_empty());
        assert!(!non_empty.is_empty());
    }
}
