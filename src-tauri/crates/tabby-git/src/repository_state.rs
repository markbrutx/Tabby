use crate::value_objects::BranchName;
use tabby_kernel::WorkingDirectory;

/// High-level state of a Git repository.
#[derive(Debug, Clone, PartialEq)]
pub struct GitRepositoryState {
    repo_path: WorkingDirectory,
    head_branch: Option<BranchName>,
    is_detached: bool,
    status_clean: bool,
}

impl GitRepositoryState {
    pub fn new(
        repo_path: WorkingDirectory,
        head_branch: Option<BranchName>,
        is_detached: bool,
        status_clean: bool,
    ) -> Self {
        Self {
            repo_path,
            head_branch,
            is_detached,
            status_clean,
        }
    }

    pub fn repo_path(&self) -> &WorkingDirectory {
        &self.repo_path
    }

    pub fn head_branch(&self) -> Option<&BranchName> {
        self.head_branch.as_ref()
    }

    pub fn is_detached(&self) -> bool {
        self.is_detached
    }

    pub fn status_clean(&self) -> bool {
        self.status_clean
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> GitRepositoryState {
        GitRepositoryState::new(
            WorkingDirectory::new("/home/user/project").expect("valid path"),
            Some(BranchName::try_new("main").expect("valid")),
            false,
            true,
        )
    }

    #[test]
    fn repository_state_field_access() {
        let state = sample_state();
        assert_eq!(
            state.repo_path(),
            &WorkingDirectory::new("/home/user/project").expect("valid")
        );
        assert_eq!(state.head_branch().map(|b| b.as_ref()), Some("main"));
        assert!(!state.is_detached());
        assert!(state.status_clean());
    }

    #[test]
    fn repository_state_detached_head() {
        let state = GitRepositoryState::new(
            WorkingDirectory::new("/repo").expect("valid"),
            None,
            true,
            false,
        );
        assert!(state.head_branch().is_none());
        assert!(state.is_detached());
        assert!(!state.status_clean());
    }

    #[test]
    fn repository_state_equality() {
        let a = sample_state();
        let b = sample_state();
        assert_eq!(a, b);
    }

    #[test]
    fn repository_state_inequality() {
        let a = sample_state();
        let b = GitRepositoryState::new(
            WorkingDirectory::new("/other/path").expect("valid"),
            Some(BranchName::try_new("develop").expect("valid")),
            true,
            false,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn repository_state_clone() {
        let a = sample_state();
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn repository_state_dirty_with_head_branch() {
        let state = GitRepositoryState::new(
            WorkingDirectory::new("/repo").expect("valid"),
            Some(BranchName::try_new("feature/wip").expect("valid")),
            false,
            false,
        );
        assert!(!state.status_clean());
        assert_eq!(state.head_branch().map(|b| b.as_ref()), Some("feature/wip"));
    }

    #[test]
    fn repository_state_clean_detached_head() {
        let state = GitRepositoryState::new(
            WorkingDirectory::new("/repo").expect("valid"),
            None,
            true,
            true,
        );
        assert!(state.is_detached());
        assert!(state.status_clean());
        assert!(state.head_branch().is_none());
    }

    #[test]
    fn repository_state_head_branch_returns_reference() {
        let state = sample_state();
        let branch: &BranchName = state.head_branch().expect("should have head branch");
        assert_eq!(branch.as_ref(), "main");
    }

    #[test]
    fn repository_state_repo_path_returns_reference() {
        let state = sample_state();
        let path = state.repo_path();
        assert_eq!(
            path,
            &WorkingDirectory::new("/home/user/project").expect("valid")
        );
    }

    #[test]
    fn repository_state_debug() {
        let state = sample_state();
        let debug = format!("{state:?}");
        assert!(debug.contains("GitRepositoryState"));
    }

    #[test]
    fn repository_state_inequality_clean_vs_dirty() {
        let clean = GitRepositoryState::new(
            WorkingDirectory::new("/repo").expect("valid"),
            Some(BranchName::try_new("main").expect("valid")),
            false,
            true,
        );
        let dirty = GitRepositoryState::new(
            WorkingDirectory::new("/repo").expect("valid"),
            Some(BranchName::try_new("main").expect("valid")),
            false,
            false,
        );
        assert_ne!(clean, dirty);
    }
}
