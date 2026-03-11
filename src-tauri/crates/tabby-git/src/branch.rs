use crate::value_objects::BranchName;

/// Information about a Git branch.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchInfo {
    name: BranchName,
    is_current: bool,
    upstream: Option<String>,
    ahead: u32,
    behind: u32,
}

impl BranchInfo {
    pub fn new(
        name: BranchName,
        is_current: bool,
        upstream: Option<String>,
        ahead: u32,
        behind: u32,
    ) -> Self {
        Self {
            name,
            is_current,
            upstream,
            ahead,
            behind,
        }
    }

    pub fn name(&self) -> &BranchName {
        &self.name
    }

    pub fn is_current(&self) -> bool {
        self.is_current
    }

    pub fn upstream(&self) -> Option<&str> {
        self.upstream.as_deref()
    }

    pub fn ahead(&self) -> u32 {
        self.ahead
    }

    pub fn behind(&self) -> u32 {
        self.behind
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_branch() -> BranchInfo {
        BranchInfo::new(
            BranchName::try_new("main").expect("valid"),
            true,
            Some("origin/main".to_string()),
            2,
            1,
        )
    }

    #[test]
    fn branch_info_field_access() {
        let branch = sample_branch();
        assert_eq!(branch.name().as_ref(), "main");
        assert!(branch.is_current());
        assert_eq!(branch.upstream(), Some("origin/main"));
        assert_eq!(branch.ahead(), 2);
        assert_eq!(branch.behind(), 1);
    }

    #[test]
    fn branch_info_no_upstream() {
        let branch = BranchInfo::new(
            BranchName::try_new("feature/local").expect("valid"),
            false,
            None,
            0,
            0,
        );
        assert_eq!(branch.name().as_ref(), "feature/local");
        assert!(!branch.is_current());
        assert_eq!(branch.upstream(), None);
        assert_eq!(branch.ahead(), 0);
        assert_eq!(branch.behind(), 0);
    }

    #[test]
    fn branch_info_equality() {
        let a = sample_branch();
        let b = sample_branch();
        assert_eq!(a, b);
    }

    #[test]
    fn branch_info_inequality_name() {
        let a = sample_branch();
        let b = BranchInfo::new(
            BranchName::try_new("develop").expect("valid"),
            true,
            Some("origin/main".to_string()),
            2,
            1,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn branch_info_inequality_current() {
        let a = sample_branch();
        let b = BranchInfo::new(
            BranchName::try_new("main").expect("valid"),
            false,
            Some("origin/main".to_string()),
            2,
            1,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn branch_info_clone() {
        let a = sample_branch();
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn branch_info_ahead_behind_zero() {
        let branch = BranchInfo::new(
            BranchName::try_new("main").expect("valid"),
            true,
            Some("origin/main".to_string()),
            0,
            0,
        );
        assert_eq!(branch.ahead(), 0);
        assert_eq!(branch.behind(), 0);
    }

    #[test]
    fn branch_info_large_ahead_count() {
        let branch = BranchInfo::new(
            BranchName::try_new("feature/lots-of-commits").expect("valid"),
            false,
            Some("origin/feature/lots-of-commits".to_string()),
            1000,
            0,
        );
        assert_eq!(branch.ahead(), 1000);
    }

    #[test]
    fn branch_info_large_behind_count() {
        let branch = BranchInfo::new(
            BranchName::try_new("stale").expect("valid"),
            false,
            Some("origin/main".to_string()),
            0,
            500,
        );
        assert_eq!(branch.behind(), 500);
    }

    #[test]
    fn branch_info_not_current() {
        let branch = BranchInfo::new(
            BranchName::try_new("feature/other").expect("valid"),
            false,
            None,
            0,
            0,
        );
        assert!(!branch.is_current());
    }

    #[test]
    fn branch_info_upstream_none_returns_none() {
        let branch = BranchInfo::new(
            BranchName::try_new("orphan").expect("valid"),
            true,
            None,
            0,
            0,
        );
        assert!(branch.upstream().is_none());
    }

    #[test]
    fn branch_info_name_returns_branch_name_ref() {
        let branch = sample_branch();
        let name: &BranchName = branch.name();
        assert_eq!(name.as_ref(), "main");
    }

    #[test]
    fn branch_info_debug() {
        let branch = sample_branch();
        let debug = format!("{branch:?}");
        assert!(debug.contains("BranchInfo"));
        assert!(debug.contains("main"));
    }

    #[test]
    fn branch_info_inequality_ahead() {
        let a = sample_branch();
        let b = BranchInfo::new(
            BranchName::try_new("main").expect("valid"),
            true,
            Some("origin/main".to_string()),
            99,
            1,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn branch_info_inequality_upstream() {
        let a = sample_branch();
        let b = BranchInfo::new(
            BranchName::try_new("main").expect("valid"),
            true,
            Some("upstream/main".to_string()),
            2,
            1,
        );
        assert_ne!(a, b);
    }
}
