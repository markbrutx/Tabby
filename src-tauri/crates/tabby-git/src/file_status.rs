// ---------------------------------------------------------------------------
// FileStatusKind
// ---------------------------------------------------------------------------

/// The kind of change a file has undergone in the index or worktree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileStatusKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Conflicted,
}

// ---------------------------------------------------------------------------
// FileStatus
// ---------------------------------------------------------------------------

/// Status of a single file in a Git repository, combining index and worktree state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStatus {
    path: String,
    old_path: Option<String>,
    index_status: FileStatusKind,
    worktree_status: FileStatusKind,
}

impl FileStatus {
    pub fn new(
        path: impl Into<String>,
        old_path: Option<String>,
        index_status: FileStatusKind,
        worktree_status: FileStatusKind,
    ) -> Self {
        Self {
            path: path.into(),
            old_path,
            index_status,
            worktree_status,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn old_path(&self) -> Option<&str> {
        self.old_path.as_deref()
    }

    pub fn index_status(&self) -> FileStatusKind {
        self.index_status
    }

    pub fn worktree_status(&self) -> FileStatusKind {
        self.worktree_status
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_status_kind_debug_and_clone() {
        let kind = FileStatusKind::Modified;
        let cloned = kind;
        assert_eq!(kind, cloned);
        assert_eq!(format!("{kind:?}"), "Modified");
    }

    #[test]
    fn file_status_kind_all_variants_are_distinct() {
        let variants = [
            FileStatusKind::Modified,
            FileStatusKind::Added,
            FileStatusKind::Deleted,
            FileStatusKind::Renamed,
            FileStatusKind::Copied,
            FileStatusKind::Untracked,
            FileStatusKind::Ignored,
            FileStatusKind::Conflicted,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn file_status_new_without_old_path() {
        let status = FileStatus::new(
            "src/main.rs",
            None,
            FileStatusKind::Modified,
            FileStatusKind::Modified,
        );
        assert_eq!(status.path(), "src/main.rs");
        assert_eq!(status.old_path(), None);
        assert_eq!(status.index_status(), FileStatusKind::Modified);
        assert_eq!(status.worktree_status(), FileStatusKind::Modified);
    }

    #[test]
    fn file_status_renamed_with_old_path() {
        let status = FileStatus::new(
            "src/new_name.rs",
            Some("src/old_name.rs".to_string()),
            FileStatusKind::Renamed,
            FileStatusKind::Renamed,
        );
        assert_eq!(status.path(), "src/new_name.rs");
        assert_eq!(status.old_path(), Some("src/old_name.rs"));
        assert_eq!(status.index_status(), FileStatusKind::Renamed);
    }

    #[test]
    fn file_status_untracked_in_worktree() {
        let status = FileStatus::new(
            "new_file.txt",
            None,
            FileStatusKind::Untracked,
            FileStatusKind::Untracked,
        );
        assert_eq!(status.worktree_status(), FileStatusKind::Untracked);
    }

    #[test]
    fn file_status_mixed_index_and_worktree() {
        let status = FileStatus::new(
            "lib.rs",
            None,
            FileStatusKind::Added,
            FileStatusKind::Modified,
        );
        assert_eq!(status.index_status(), FileStatusKind::Added);
        assert_eq!(status.worktree_status(), FileStatusKind::Modified);
    }

    #[test]
    fn file_status_equality() {
        let a = FileStatus::new("a.rs", None, FileStatusKind::Added, FileStatusKind::Added);
        let b = FileStatus::new("a.rs", None, FileStatusKind::Added, FileStatusKind::Added);
        let c = FileStatus::new(
            "b.rs",
            None,
            FileStatusKind::Deleted,
            FileStatusKind::Deleted,
        );
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn file_status_clone() {
        let original = FileStatus::new(
            "test.rs",
            Some("old_test.rs".to_string()),
            FileStatusKind::Renamed,
            FileStatusKind::Renamed,
        );
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
