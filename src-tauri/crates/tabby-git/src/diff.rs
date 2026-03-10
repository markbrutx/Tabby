// ---------------------------------------------------------------------------
// DiffLineKind
// ---------------------------------------------------------------------------

/// The kind of a single line within a diff hunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffLineKind {
    Context,
    Addition,
    Deletion,
    HunkHeader,
}

// ---------------------------------------------------------------------------
// DiffLine
// ---------------------------------------------------------------------------

/// A single line in a diff hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    kind: DiffLineKind,
    old_line_no: Option<u32>,
    new_line_no: Option<u32>,
    content: String,
}

impl DiffLine {
    pub fn new(
        kind: DiffLineKind,
        old_line_no: Option<u32>,
        new_line_no: Option<u32>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            old_line_no,
            new_line_no,
            content: content.into(),
        }
    }

    pub fn kind(&self) -> DiffLineKind {
        self.kind
    }

    pub fn old_line_no(&self) -> Option<u32> {
        self.old_line_no
    }

    pub fn new_line_no(&self) -> Option<u32> {
        self.new_line_no
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

// ---------------------------------------------------------------------------
// DiffHunk
// ---------------------------------------------------------------------------

/// A contiguous block of changes within a diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    old_start: u32,
    old_count: u32,
    new_start: u32,
    new_count: u32,
    header: String,
    lines: Vec<DiffLine>,
}

impl DiffHunk {
    pub fn new(
        old_start: u32,
        old_count: u32,
        new_start: u32,
        new_count: u32,
        header: impl Into<String>,
        lines: Vec<DiffLine>,
    ) -> Self {
        Self {
            old_start,
            old_count,
            new_start,
            new_count,
            header: header.into(),
            lines,
        }
    }

    pub fn old_start(&self) -> u32 {
        self.old_start
    }

    pub fn old_count(&self) -> u32 {
        self.old_count
    }

    pub fn new_start(&self) -> u32 {
        self.new_start
    }

    pub fn new_count(&self) -> u32 {
        self.new_count
    }

    pub fn header(&self) -> &str {
        &self.header
    }

    pub fn lines(&self) -> &[DiffLine] {
        &self.lines
    }
}

// ---------------------------------------------------------------------------
// DiffContent
// ---------------------------------------------------------------------------

/// The full diff for a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffContent {
    file_path: String,
    old_path: Option<String>,
    hunks: Vec<DiffHunk>,
    is_binary: bool,
    file_mode_change: Option<String>,
}

impl DiffContent {
    pub fn new(
        file_path: impl Into<String>,
        old_path: Option<String>,
        hunks: Vec<DiffHunk>,
        is_binary: bool,
        file_mode_change: Option<String>,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            old_path,
            hunks,
            is_binary,
            file_mode_change,
        }
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn old_path(&self) -> Option<&str> {
        self.old_path.as_deref()
    }

    pub fn hunks(&self) -> &[DiffHunk] {
        &self.hunks
    }

    pub fn is_binary(&self) -> bool {
        self.is_binary
    }

    pub fn file_mode_change(&self) -> Option<&str> {
        self.file_mode_change.as_deref()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- DiffLineKind -------------------------------------------------------

    #[test]
    fn diff_line_kind_all_variants_are_distinct() {
        let variants = [
            DiffLineKind::Context,
            DiffLineKind::Addition,
            DiffLineKind::Deletion,
            DiffLineKind::HunkHeader,
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
    fn diff_line_kind_debug() {
        assert_eq!(format!("{:?}", DiffLineKind::Addition), "Addition");
    }

    // -- DiffLine -----------------------------------------------------------

    #[test]
    fn diff_line_context() {
        let line = DiffLine::new(DiffLineKind::Context, Some(10), Some(10), "    let x = 1;");
        assert_eq!(line.kind(), DiffLineKind::Context);
        assert_eq!(line.old_line_no(), Some(10));
        assert_eq!(line.new_line_no(), Some(10));
        assert_eq!(line.content(), "    let x = 1;");
    }

    #[test]
    fn diff_line_addition() {
        let line = DiffLine::new(DiffLineKind::Addition, None, Some(11), "    let y = 2;");
        assert_eq!(line.kind(), DiffLineKind::Addition);
        assert_eq!(line.old_line_no(), None);
        assert_eq!(line.new_line_no(), Some(11));
    }

    #[test]
    fn diff_line_deletion() {
        let line = DiffLine::new(DiffLineKind::Deletion, Some(11), None, "    let z = 3;");
        assert_eq!(line.kind(), DiffLineKind::Deletion);
        assert_eq!(line.old_line_no(), Some(11));
        assert_eq!(line.new_line_no(), None);
    }

    #[test]
    fn diff_line_hunk_header() {
        let line = DiffLine::new(
            DiffLineKind::HunkHeader,
            None,
            None,
            "@@ -10,5 +10,7 @@ fn main()",
        );
        assert_eq!(line.kind(), DiffLineKind::HunkHeader);
        assert_eq!(line.old_line_no(), None);
        assert_eq!(line.new_line_no(), None);
    }

    #[test]
    fn diff_line_equality() {
        let a = DiffLine::new(DiffLineKind::Addition, None, Some(1), "hello");
        let b = DiffLine::new(DiffLineKind::Addition, None, Some(1), "hello");
        let c = DiffLine::new(DiffLineKind::Deletion, Some(1), None, "hello");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn diff_line_clone() {
        let original = DiffLine::new(DiffLineKind::Context, Some(5), Some(5), "code");
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // -- DiffHunk -----------------------------------------------------------

    #[test]
    fn diff_hunk_construction() {
        let lines = vec![
            DiffLine::new(DiffLineKind::Context, Some(10), Some(10), "    let x = 1;"),
            DiffLine::new(DiffLineKind::Addition, None, Some(11), "    let y = 2;"),
        ];
        let hunk = DiffHunk::new(10, 1, 10, 2, "@@ -10,1 +10,2 @@", lines);

        assert_eq!(hunk.old_start(), 10);
        assert_eq!(hunk.old_count(), 1);
        assert_eq!(hunk.new_start(), 10);
        assert_eq!(hunk.new_count(), 2);
        assert_eq!(hunk.header(), "@@ -10,1 +10,2 @@");
        assert_eq!(hunk.lines().len(), 2);
    }

    #[test]
    fn diff_hunk_empty_lines() {
        let hunk = DiffHunk::new(1, 0, 1, 0, "@@ -1,0 +1,0 @@", vec![]);
        assert!(hunk.lines().is_empty());
    }

    #[test]
    fn diff_hunk_equality() {
        let lines = vec![DiffLine::new(DiffLineKind::Addition, None, Some(1), "new")];
        let a = DiffHunk::new(1, 0, 1, 1, "@@ -1,0 +1,1 @@", lines.clone());
        let b = DiffHunk::new(1, 0, 1, 1, "@@ -1,0 +1,1 @@", lines);
        assert_eq!(a, b);
    }

    #[test]
    fn diff_hunk_clone() {
        let hunk = DiffHunk::new(
            5,
            3,
            5,
            4,
            "@@ -5,3 +5,4 @@",
            vec![DiffLine::new(DiffLineKind::Context, Some(5), Some(5), "x")],
        );
        let cloned = hunk.clone();
        assert_eq!(hunk, cloned);
    }

    // -- DiffContent --------------------------------------------------------

    #[test]
    fn diff_content_simple_file() {
        let lines = vec![DiffLine::new(
            DiffLineKind::Addition,
            None,
            Some(1),
            "hello",
        )];
        let hunk = DiffHunk::new(0, 0, 1, 1, "@@ -0,0 +1,1 @@", lines);
        let diff = DiffContent::new("src/main.rs", None, vec![hunk], false, None);

        assert_eq!(diff.file_path(), "src/main.rs");
        assert_eq!(diff.old_path(), None);
        assert_eq!(diff.hunks().len(), 1);
        assert!(!diff.is_binary());
        assert_eq!(diff.file_mode_change(), None);
    }

    #[test]
    fn diff_content_renamed_file() {
        let diff = DiffContent::new(
            "src/new.rs",
            Some("src/old.rs".to_string()),
            vec![],
            false,
            None,
        );
        assert_eq!(diff.file_path(), "src/new.rs");
        assert_eq!(diff.old_path(), Some("src/old.rs"));
    }

    #[test]
    fn diff_content_binary_file() {
        let diff = DiffContent::new("image.png", None, vec![], true, None);
        assert!(diff.is_binary());
        assert!(diff.hunks().is_empty());
    }

    #[test]
    fn diff_content_with_file_mode_change() {
        let diff = DiffContent::new(
            "script.sh",
            None,
            vec![],
            false,
            Some("100644 -> 100755".to_string()),
        );
        assert_eq!(diff.file_mode_change(), Some("100644 -> 100755"));
    }

    #[test]
    fn diff_content_equality() {
        let a = DiffContent::new("a.rs", None, vec![], false, None);
        let b = DiffContent::new("a.rs", None, vec![], false, None);
        let c = DiffContent::new("b.rs", None, vec![], false, None);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn diff_content_clone() {
        let lines = vec![DiffLine::new(
            DiffLineKind::Deletion,
            Some(1),
            None,
            "removed",
        )];
        let hunk = DiffHunk::new(1, 1, 1, 0, "@@ -1,1 +1,0 @@", lines);
        let original = DiffContent::new(
            "file.rs",
            Some("old_file.rs".to_string()),
            vec![hunk],
            false,
            Some("100644 -> 100755".to_string()),
        );
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn diff_content_multiple_hunks() {
        let hunk1 = DiffHunk::new(
            1,
            1,
            1,
            2,
            "@@ -1,1 +1,2 @@",
            vec![DiffLine::new(
                DiffLineKind::Addition,
                None,
                Some(2),
                "new line",
            )],
        );
        let hunk2 = DiffHunk::new(
            50,
            1,
            51,
            0,
            "@@ -50,1 +51,0 @@",
            vec![DiffLine::new(
                DiffLineKind::Deletion,
                Some(50),
                None,
                "old line",
            )],
        );
        let diff = DiffContent::new("large_file.rs", None, vec![hunk1, hunk2], false, None);
        assert_eq!(diff.hunks().len(), 2);
        assert_eq!(diff.hunks()[0].old_start(), 1);
        assert_eq!(diff.hunks()[1].old_start(), 50);
    }
}
