pub mod ids;

use std::collections::HashMap;

use tabby_kernel::{BrowserUrl, PaneId, WorkingDirectory};
use thiserror::Error;

pub use ids::RuntimeSessionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeKind {
    Terminal,
    Browser,
    Git,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStatus {
    Starting,
    Running,
    Exited,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneRuntime {
    pub pane_id: PaneId,
    pub runtime_session_id: Option<RuntimeSessionId>,
    pub kind: RuntimeKind,
    pub status: RuntimeStatus,
    pub last_error: Option<String>,
    pub browser_location: Option<BrowserUrl>,
    pub terminal_cwd: Option<WorkingDirectory>,
    pub git_repo_path: Option<WorkingDirectory>,
}

#[derive(Debug, Default)]
pub struct RuntimeRegistry {
    runtimes: HashMap<PaneId, PaneRuntime>,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("runtime not found for pane {0}")]
    NotFound(String),
}

impl RuntimeRegistry {
    pub fn register_terminal(
        &mut self,
        pane_id: &PaneId,
        runtime_session_id: RuntimeSessionId,
    ) -> PaneRuntime {
        let runtime = PaneRuntime {
            pane_id: pane_id.clone(),
            runtime_session_id: Some(runtime_session_id),
            kind: RuntimeKind::Terminal,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: None,
            terminal_cwd: None,
            git_repo_path: None,
        };
        self.runtimes.insert(pane_id.clone(), runtime.clone());
        runtime
    }

    pub fn register_browser(
        &mut self,
        pane_id: &PaneId,
        runtime_session_id: RuntimeSessionId,
        initial_url: BrowserUrl,
    ) -> PaneRuntime {
        let runtime = PaneRuntime {
            pane_id: pane_id.clone(),
            runtime_session_id: Some(runtime_session_id),
            kind: RuntimeKind::Browser,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: Some(initial_url),
            terminal_cwd: None,
            git_repo_path: None,
        };
        self.runtimes.insert(pane_id.clone(), runtime.clone());
        runtime
    }

    pub fn register_git(
        &mut self,
        pane_id: &PaneId,
        runtime_session_id: RuntimeSessionId,
        repo_path: WorkingDirectory,
    ) -> PaneRuntime {
        let runtime = PaneRuntime {
            pane_id: pane_id.clone(),
            runtime_session_id: Some(runtime_session_id),
            kind: RuntimeKind::Git,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: None,
            terminal_cwd: None,
            git_repo_path: Some(repo_path),
        };
        self.runtimes.insert(pane_id.clone(), runtime.clone());
        runtime
    }

    pub fn mark_terminal_exit(
        &mut self,
        pane_id: &PaneId,
        runtime_session_id: Option<&RuntimeSessionId>,
        failed: bool,
        message: Option<String>,
    ) -> Result<PaneRuntime, RuntimeError> {
        let runtime = self
            .runtimes
            .get_mut(pane_id)
            .ok_or_else(|| RuntimeError::NotFound(pane_id.to_string()))?;

        if let Some(expected) = runtime_session_id {
            if runtime.runtime_session_id.as_ref() != Some(expected) {
                return Ok(runtime.clone());
            }
        }

        runtime.status = if failed {
            RuntimeStatus::Failed
        } else {
            RuntimeStatus::Exited
        };
        runtime.last_error = message;

        Ok(runtime.clone())
    }

    pub fn update_browser_location(
        &mut self,
        pane_id: &PaneId,
        url: BrowserUrl,
    ) -> Result<PaneRuntime, RuntimeError> {
        let runtime = self
            .runtimes
            .get_mut(pane_id)
            .ok_or_else(|| RuntimeError::NotFound(pane_id.to_string()))?;
        runtime.browser_location = Some(url);
        Ok(runtime.clone())
    }

    pub fn update_terminal_cwd(
        &mut self,
        pane_id: &PaneId,
        cwd: WorkingDirectory,
    ) -> Result<PaneRuntime, RuntimeError> {
        let runtime = self
            .runtimes
            .get_mut(pane_id)
            .ok_or_else(|| RuntimeError::NotFound(pane_id.to_string()))?;
        runtime.terminal_cwd = Some(cwd);
        Ok(runtime.clone())
    }

    pub fn remove(&mut self, pane_id: &PaneId) -> Option<PaneRuntime> {
        self.runtimes.remove(pane_id)
    }

    pub fn get(&self, pane_id: &PaneId) -> Option<&PaneRuntime> {
        self.runtimes.get(pane_id)
    }

    pub fn terminal_session_id(&self, pane_id: &PaneId) -> Option<RuntimeSessionId> {
        self.runtimes
            .get(pane_id)
            .and_then(|runtime| runtime.runtime_session_id.clone())
    }

    pub fn snapshot(&self) -> Vec<PaneRuntime> {
        self.runtimes.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::{PaneRuntime, RuntimeError, RuntimeKind, RuntimeRegistry, RuntimeSessionId, RuntimeStatus};
    use tabby_kernel::{BrowserUrl, PaneId, WorkingDirectory};

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    fn pid(id: &str) -> PaneId {
        PaneId::from(String::from(id))
    }

    fn sid(id: &str) -> RuntimeSessionId {
        RuntimeSessionId::from(String::from(id))
    }

    fn cwd(path: &str) -> WorkingDirectory {
        WorkingDirectory::new(path).expect("valid path")
    }

    fn url(u: &str) -> BrowserUrl {
        BrowserUrl::new(u)
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: construction
    // ---------------------------------------------------------------------------

    #[test]
    fn registry_default_is_empty() {
        let registry = RuntimeRegistry::default();
        assert!(registry.snapshot().is_empty());
    }

    #[test]
    fn registry_get_on_empty_registry_returns_none() {
        let registry = RuntimeRegistry::default();
        assert!(registry.get(&pid("no-such-pane")).is_none());
    }

    #[test]
    fn registry_terminal_session_id_on_empty_returns_none() {
        let registry = RuntimeRegistry::default();
        assert!(registry.terminal_session_id(&pid("no-such-pane")).is_none());
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: register_terminal
    // ---------------------------------------------------------------------------

    #[test]
    fn registers_terminal_and_marks_exit() {
        let mut registry = RuntimeRegistry::default();
        let session_id = sid("session-1");
        let pane_id = pid("pane-1");
        registry.register_terminal(&pane_id, session_id.clone());

        let runtime = registry
            .mark_terminal_exit(&pane_id, Some(&session_id), false, None)
            .expect("runtime should exist");
        assert_eq!(runtime.status, RuntimeStatus::Exited);
    }

    #[test]
    fn register_terminal_returns_correct_fields() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-term-fields");
        let session_id = sid("s-fields");

        let runtime = registry.register_terminal(&pane_id, session_id.clone());

        assert_eq!(runtime.pane_id, pane_id);
        assert_eq!(runtime.runtime_session_id, Some(session_id));
        assert_eq!(runtime.kind, RuntimeKind::Terminal);
        assert_eq!(runtime.status, RuntimeStatus::Running);
        assert!(runtime.last_error.is_none());
        assert!(runtime.browser_location.is_none());
        assert!(runtime.terminal_cwd.is_none());
        assert!(runtime.git_repo_path.is_none());
    }

    #[test]
    fn register_terminal_is_retrievable_via_get() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-get");
        registry.register_terminal(&pane_id, sid("s1"));

        let found = registry.get(&pane_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().kind, RuntimeKind::Terminal);
    }

    #[test]
    fn register_terminal_appears_in_snapshot() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal(&pid("p1"), sid("s1"));
        assert_eq!(registry.snapshot().len(), 1);
    }

    #[test]
    fn register_terminal_duplicate_overwrites_previous() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-dup");
        registry.register_terminal(&pane_id, sid("session-old"));
        registry.register_terminal(&pane_id, sid("session-new"));

        // Only one entry
        assert_eq!(registry.snapshot().len(), 1);
        // The latest session id wins
        let session = registry.terminal_session_id(&pane_id);
        assert_eq!(session, Some(sid("session-new")));
    }

    #[test]
    fn pane_runtime_uses_pane_id_not_raw_string() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-typed");
        let runtime = registry.register_terminal(&pane_id, sid("session-1"));
        assert_eq!(runtime.pane_id, pane_id);
        assert_eq!(runtime.pane_id.as_ref(), "pane-typed");
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: register_browser
    // ---------------------------------------------------------------------------

    #[test]
    fn register_browser_returns_correct_fields() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-browser");
        let session_id = sid("b-session");
        let initial_url = url("https://example.com");

        let runtime = registry.register_browser(&pane_id, session_id.clone(), initial_url.clone());

        assert_eq!(runtime.pane_id, pane_id);
        assert_eq!(runtime.runtime_session_id, Some(session_id));
        assert_eq!(runtime.kind, RuntimeKind::Browser);
        assert_eq!(runtime.status, RuntimeStatus::Running);
        assert_eq!(runtime.browser_location.as_ref().map(|u| u.as_str()), Some("https://example.com"));
        assert!(runtime.last_error.is_none());
        assert!(runtime.terminal_cwd.is_none());
        assert!(runtime.git_repo_path.is_none());
    }

    #[test]
    fn register_browser_duplicate_overwrites_previous() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-browser-dup");
        registry.register_browser(&pane_id, sid("b1"), url("https://a.com"));
        registry.register_browser(&pane_id, sid("b2"), url("https://b.com"));

        assert_eq!(registry.snapshot().len(), 1);
        let r = registry.get(&pane_id).unwrap();
        assert_eq!(r.browser_location.as_ref().map(|u| u.as_str()), Some("https://b.com"));
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: register_git
    // ---------------------------------------------------------------------------

    #[test]
    fn registers_git_runtime() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("git-pane-1");
        let session_id = sid("git-session-1");
        let repo_path = cwd("/projects/tabby");

        let runtime = registry.register_git(&pane_id, session_id, repo_path);

        assert_eq!(runtime.kind, RuntimeKind::Git);
        assert_eq!(runtime.status, RuntimeStatus::Running);
        assert_eq!(runtime.git_repo_path.as_ref().map(|p| p.as_str()), Some("/projects/tabby"));
        assert!(runtime.browser_location.is_none());
        assert!(runtime.terminal_cwd.is_none());
    }

    #[test]
    fn register_git_returns_correct_fields() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-git-fields");
        let session_id = sid("g-fields");
        let repo = cwd("/repo/path");

        let runtime = registry.register_git(&pane_id, session_id.clone(), repo);

        assert_eq!(runtime.pane_id, pane_id);
        assert_eq!(runtime.runtime_session_id, Some(session_id));
        assert_eq!(runtime.kind, RuntimeKind::Git);
        assert_eq!(runtime.status, RuntimeStatus::Running);
        assert!(runtime.last_error.is_none());
        assert!(runtime.browser_location.is_none());
        assert!(runtime.terminal_cwd.is_none());
    }

    #[test]
    fn git_runtime_snapshot_includes_git_entries() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("git-pane-2");
        let repo_path = cwd("/repos/my-repo");
        registry.register_git(&pane_id, sid("git-session-2"), repo_path);

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].kind, RuntimeKind::Git);
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: remove / unregister
    // ---------------------------------------------------------------------------

    #[test]
    fn remove_registered_pane_returns_runtime() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-remove");
        registry.register_terminal(&pane_id, sid("s1"));

        let removed = registry.remove(&pane_id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().kind, RuntimeKind::Terminal);
    }

    #[test]
    fn remove_registered_pane_decreases_snapshot_count() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-rm-dec");
        registry.register_terminal(&pane_id, sid("s1"));
        registry.remove(&pane_id);

        assert!(registry.snapshot().is_empty());
    }

    #[test]
    fn remove_nonexistent_pane_returns_none() {
        let mut registry = RuntimeRegistry::default();
        let removed = registry.remove(&pid("no-such-pane"));
        assert!(removed.is_none());
    }

    #[test]
    fn get_after_remove_returns_none() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-get-after-rm");
        registry.register_terminal(&pane_id, sid("s1"));
        registry.remove(&pane_id);

        assert!(registry.get(&pane_id).is_none());
    }

    #[test]
    fn git_runtime_remove() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("git-pane-3");
        registry.register_git(&pane_id, sid("git-session-3"), cwd("/repos/another"));

        let removed = registry.remove(&pane_id);
        assert!(removed.is_some());
        assert_eq!(removed.as_ref().map(|r| r.kind), Some(RuntimeKind::Git));
        assert!(registry.get(&pane_id).is_none());
    }

    #[test]
    fn remove_is_idempotent_second_call_returns_none() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-idempotent-rm");
        registry.register_terminal(&pane_id, sid("s1"));
        registry.remove(&pane_id);

        let second = registry.remove(&pane_id);
        assert!(second.is_none());
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: get / lookup
    // ---------------------------------------------------------------------------

    #[test]
    fn git_runtime_lookup() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("git-pane-4");
        registry.register_git(&pane_id, sid("git-session-4"), cwd("/repos/lookup"));

        let found = registry.get(&pane_id);
        assert!(found.is_some());
        assert_eq!(found.map(|r| r.kind), Some(RuntimeKind::Git));
    }

    #[test]
    fn get_returns_reference_not_owned() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-ref");
        registry.register_terminal(&pane_id, sid("s-ref"));

        let r: Option<&PaneRuntime> = registry.get(&pane_id);
        assert!(r.is_some());
    }

    #[test]
    fn get_different_pane_ids_are_independent() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal(&pid("pane-a"), sid("sa"));
        registry.register_browser(&pid("pane-b"), sid("sb"), url("https://b.com"));

        assert_eq!(registry.get(&pid("pane-a")).unwrap().kind, RuntimeKind::Terminal);
        assert_eq!(registry.get(&pid("pane-b")).unwrap().kind, RuntimeKind::Browser);
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: terminal_session_id
    // ---------------------------------------------------------------------------

    #[test]
    fn terminal_session_id_returns_session_for_registered_pane() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-tsid");
        let session_id = sid("tsid-session");
        registry.register_terminal(&pane_id, session_id.clone());

        let found = registry.terminal_session_id(&pane_id);
        assert_eq!(found, Some(session_id));
    }

    #[test]
    fn terminal_session_id_returns_none_after_removal() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-tsid-rm");
        registry.register_terminal(&pane_id, sid("tsid-rm-session"));
        registry.remove(&pane_id);

        assert!(registry.terminal_session_id(&pane_id).is_none());
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: mark_terminal_exit
    // ---------------------------------------------------------------------------

    #[test]
    fn mark_terminal_exit_clean_sets_status_exited() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-exit-clean");
        let session_id = sid("session-exit-clean");
        registry.register_terminal(&pane_id, session_id.clone());

        let runtime = registry
            .mark_terminal_exit(&pane_id, Some(&session_id), false, None)
            .unwrap();
        assert_eq!(runtime.status, RuntimeStatus::Exited);
        assert!(runtime.last_error.is_none());
    }

    #[test]
    fn mark_terminal_exit_failed_sets_status_failed() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-exit-fail");
        let session_id = sid("session-exit-fail");
        registry.register_terminal(&pane_id, session_id.clone());

        let runtime = registry
            .mark_terminal_exit(&pane_id, Some(&session_id), true, Some("process crashed".into()))
            .unwrap();
        assert_eq!(runtime.status, RuntimeStatus::Failed);
        assert_eq!(runtime.last_error.as_deref(), Some("process crashed"));
    }

    #[test]
    fn mark_terminal_exit_with_wrong_session_id_is_noop() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-wrong-sid");
        let real_session = sid("real-session");
        let stale_session = sid("stale-session");
        registry.register_terminal(&pane_id, real_session.clone());

        let runtime = registry
            .mark_terminal_exit(&pane_id, Some(&stale_session), true, Some("error".into()))
            .unwrap();
        // Status should remain Running because session IDs don't match
        assert_eq!(runtime.status, RuntimeStatus::Running);
    }

    #[test]
    fn mark_terminal_exit_with_no_session_id_always_applies() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-no-sid");
        registry.register_terminal(&pane_id, sid("some-session"));

        let runtime = registry
            .mark_terminal_exit(&pane_id, None, false, None)
            .unwrap();
        assert_eq!(runtime.status, RuntimeStatus::Exited);
    }

    #[test]
    fn mark_terminal_exit_for_nonexistent_pane_returns_error() {
        let mut registry = RuntimeRegistry::default();
        let result = registry.mark_terminal_exit(&pid("ghost"), None, false, None);
        assert!(result.is_err());

        // Check error message contains the pane id
        match result.unwrap_err() {
            RuntimeError::NotFound(msg) => assert!(msg.contains("ghost")),
        }
    }

    #[test]
    fn mark_terminal_exit_with_no_session_id_and_failed_true_sets_failed() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-no-sid-fail");
        registry.register_terminal(&pane_id, sid("s1"));

        let runtime = registry
            .mark_terminal_exit(&pane_id, None, true, Some("seg fault".into()))
            .unwrap();
        assert_eq!(runtime.status, RuntimeStatus::Failed);
        assert_eq!(runtime.last_error.as_deref(), Some("seg fault"));
    }

    #[test]
    fn mark_terminal_exit_persists_in_registry() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-persist-exit");
        let session_id = sid("s-persist");
        registry.register_terminal(&pane_id, session_id.clone());
        registry.mark_terminal_exit(&pane_id, Some(&session_id), false, None).unwrap();

        let r = registry.get(&pane_id).unwrap();
        assert_eq!(r.status, RuntimeStatus::Exited);
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: update_terminal_cwd
    // ---------------------------------------------------------------------------

    #[test]
    fn terminal_cwd_updates() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-1");
        registry.register_terminal(&pane_id, sid("session-1"));

        let new_cwd = cwd("/projects/tabby");
        let runtime = registry.update_terminal_cwd(&pane_id, new_cwd).unwrap();
        assert_eq!(runtime.terminal_cwd.as_ref().map(|w| w.as_str()), Some("/projects/tabby"));
    }

    #[test]
    fn terminal_cwd_update_overwrites_previous() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-cwd-overwrite");
        registry.register_terminal(&pane_id, sid("s1"));
        registry.update_terminal_cwd(&pane_id, cwd("/first")).unwrap();
        registry.update_terminal_cwd(&pane_id, cwd("/second")).unwrap();

        let r = registry.get(&pane_id).unwrap();
        assert_eq!(r.terminal_cwd.as_ref().map(|w| w.as_str()), Some("/second"));
    }

    #[test]
    fn terminal_cwd_update_for_nonexistent_pane_returns_error() {
        let mut registry = RuntimeRegistry::default();
        let result = registry.update_terminal_cwd(&pid("nonexistent"), cwd("/tmp"));
        assert!(result.is_err(), "updating cwd for nonexistent pane should fail");
    }

    #[test]
    fn terminal_cwd_update_persists_in_registry() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-cwd-persist");
        registry.register_terminal(&pane_id, sid("s1"));
        registry.update_terminal_cwd(&pane_id, cwd("/persisted")).unwrap();

        let r = registry.get(&pane_id).unwrap();
        assert_eq!(r.terminal_cwd.as_ref().map(|w| w.as_str()), Some("/persisted"));
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: update_browser_location
    // ---------------------------------------------------------------------------

    #[test]
    fn browser_location_updates() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-1");
        registry.register_browser(&pane_id, sid("browser-1"), url("https://example.com"));

        let runtime = registry
            .update_browser_location(&pane_id, url("https://openai.com"))
            .unwrap();
        assert_eq!(runtime.browser_location.as_ref().map(|u| u.as_str()), Some("https://openai.com"));
    }

    #[test]
    fn browser_location_update_overwrites_initial_url() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-url-overwrite");
        registry.register_browser(&pane_id, sid("b1"), url("https://initial.com"));
        registry.update_browser_location(&pane_id, url("https://updated.com")).unwrap();
        registry.update_browser_location(&pane_id, url("https://final.com")).unwrap();

        let r = registry.get(&pane_id).unwrap();
        assert_eq!(r.browser_location.as_ref().map(|u| u.as_str()), Some("https://final.com"));
    }

    #[test]
    fn browser_location_update_for_nonexistent_pane_returns_error() {
        let mut registry = RuntimeRegistry::default();
        let result = registry.update_browser_location(&pid("ghost"), url("https://x.com"));
        assert!(result.is_err());
    }

    #[test]
    fn browser_location_update_persists_in_registry() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-url-persist");
        registry.register_browser(&pane_id, sid("b1"), url("https://a.com"));
        registry.update_browser_location(&pane_id, url("https://b.com")).unwrap();

        let r = registry.get(&pane_id).unwrap();
        assert_eq!(r.browser_location.as_ref().map(|u| u.as_str()), Some("https://b.com"));
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: snapshot
    // ---------------------------------------------------------------------------

    #[test]
    fn snapshot_returns_all_registered_runtimes() {
        let mut registry = RuntimeRegistry::default();
        registry.register_terminal(&pid("p1"), sid("s1"));
        registry.register_browser(&pid("p2"), sid("s2"), url("https://a.com"));
        registry.register_git(&pid("p3"), sid("s3"), cwd("/repo"));

        assert_eq!(registry.snapshot().len(), 3);
    }

    #[test]
    fn snapshot_is_a_clone_not_a_reference() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-snap-clone");
        registry.register_terminal(&pane_id, sid("s1"));

        let snap = registry.snapshot();
        // Mutate the registry after taking a snapshot
        registry.remove(&pane_id);

        // The snapshot should still contain the pane
        assert_eq!(snap.len(), 1);
    }

    #[test]
    fn snapshot_after_all_removed_is_empty() {
        let mut registry = RuntimeRegistry::default();
        let p1 = pid("p1");
        let p2 = pid("p2");
        registry.register_terminal(&p1, sid("s1"));
        registry.register_terminal(&p2, sid("s2"));
        registry.remove(&p1);
        registry.remove(&p2);

        assert!(registry.snapshot().is_empty());
    }

    // ---------------------------------------------------------------------------
    // RuntimeRegistry: mixed kinds & capacity
    // ---------------------------------------------------------------------------

    #[test]
    fn registry_holds_many_different_pane_kinds() {
        let mut registry = RuntimeRegistry::default();
        for i in 0..10 {
            registry.register_terminal(&pid(&format!("term-{i}")), sid(&format!("st-{i}")));
        }
        for i in 0..5 {
            registry.register_browser(&pid(&format!("browser-{i}")), sid(&format!("sb-{i}")), url("https://x.com"));
        }
        for i in 0..3 {
            registry.register_git(&pid(&format!("git-{i}")), sid(&format!("sg-{i}")), cwd("/r"));
        }

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.len(), 18);

        let terminals = snapshot.iter().filter(|r| r.kind == RuntimeKind::Terminal).count();
        let browsers = snapshot.iter().filter(|r| r.kind == RuntimeKind::Browser).count();
        let gits = snapshot.iter().filter(|r| r.kind == RuntimeKind::Git).count();

        assert_eq!(terminals, 10);
        assert_eq!(browsers, 5);
        assert_eq!(gits, 3);
    }

    #[test]
    fn registry_unique_pane_ids_each_get_independent_state() {
        let mut registry = RuntimeRegistry::default();
        let p1 = pid("unique-p1");
        let p2 = pid("unique-p2");

        registry.register_terminal(&p1, sid("s1"));
        registry.register_terminal(&p2, sid("s2"));

        // Update cwd for p1 only
        registry.update_terminal_cwd(&p1, cwd("/for-p1")).unwrap();

        assert_eq!(registry.get(&p1).unwrap().terminal_cwd.as_ref().map(|w| w.as_str()), Some("/for-p1"));
        assert!(registry.get(&p2).unwrap().terminal_cwd.is_none());
    }

    // ---------------------------------------------------------------------------
    // RuntimeKind: variants and properties
    // ---------------------------------------------------------------------------

    #[test]
    fn runtime_kind_all_variants_are_distinct() {
        let terminal = RuntimeKind::Terminal;
        let browser = RuntimeKind::Browser;
        let git = RuntimeKind::Git;

        assert_ne!(terminal, browser);
        assert_ne!(terminal, git);
        assert_ne!(browser, git);
    }

    #[test]
    fn runtime_kind_is_copy() {
        let kind = RuntimeKind::Terminal;
        let copy = kind;
        assert_eq!(kind, copy);
    }

    #[test]
    fn runtime_kind_is_clone() {
        let kind = RuntimeKind::Browser;
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn runtime_kind_debug_format() {
        assert!(format!("{:?}", RuntimeKind::Terminal).contains("Terminal"));
        assert!(format!("{:?}", RuntimeKind::Browser).contains("Browser"));
        assert!(format!("{:?}", RuntimeKind::Git).contains("Git"));
    }

    // ---------------------------------------------------------------------------
    // RuntimeStatus: variants and transitions
    // ---------------------------------------------------------------------------

    #[test]
    fn runtime_status_all_variants_are_distinct() {
        assert_ne!(RuntimeStatus::Starting, RuntimeStatus::Running);
        assert_ne!(RuntimeStatus::Running, RuntimeStatus::Exited);
        assert_ne!(RuntimeStatus::Exited, RuntimeStatus::Failed);
        assert_ne!(RuntimeStatus::Starting, RuntimeStatus::Failed);
    }

    #[test]
    fn runtime_status_is_copy() {
        let status = RuntimeStatus::Running;
        let copy = status;
        assert_eq!(status, copy);
    }

    #[test]
    fn runtime_status_is_clone() {
        let status = RuntimeStatus::Exited;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn runtime_status_debug_format() {
        assert!(format!("{:?}", RuntimeStatus::Starting).contains("Starting"));
        assert!(format!("{:?}", RuntimeStatus::Running).contains("Running"));
        assert!(format!("{:?}", RuntimeStatus::Exited).contains("Exited"));
        assert!(format!("{:?}", RuntimeStatus::Failed).contains("Failed"));
    }

    #[test]
    fn terminal_initial_status_is_running_not_starting() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-initial-status");
        let runtime = registry.register_terminal(&pane_id, sid("s1"));
        // register_terminal directly sets Running (no Starting state in the current model)
        assert_eq!(runtime.status, RuntimeStatus::Running);
    }

    #[test]
    fn status_transition_running_to_exited() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-run-to-exit");
        let session_id = sid("s-run-exit");
        let runtime = registry.register_terminal(&pane_id, session_id.clone());
        assert_eq!(runtime.status, RuntimeStatus::Running);

        let after_exit = registry
            .mark_terminal_exit(&pane_id, Some(&session_id), false, None)
            .unwrap();
        assert_eq!(after_exit.status, RuntimeStatus::Exited);
    }

    #[test]
    fn status_transition_running_to_failed() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-run-to-fail");
        let session_id = sid("s-run-fail");
        registry.register_terminal(&pane_id, session_id.clone());

        let after_exit = registry
            .mark_terminal_exit(&pane_id, Some(&session_id), true, None)
            .unwrap();
        assert_eq!(after_exit.status, RuntimeStatus::Failed);
    }

    // ---------------------------------------------------------------------------
    // PaneRuntime: equality and Clone
    // ---------------------------------------------------------------------------

    #[test]
    fn pane_runtime_equality() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-eq");
        let r1 = registry.register_terminal(&pane_id, sid("s-eq"));
        let r2 = registry.get(&pane_id).unwrap().clone();
        assert_eq!(r1, r2);
    }

    #[test]
    fn pane_runtime_clone_is_independent() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-clone-indep");
        let original = registry.register_terminal(&pane_id, sid("s-clone"));
        let cloned = original.clone();

        // Both should be equal initially
        assert_eq!(original, cloned);
        // Mutating the registry does not affect the clone
        registry.update_terminal_cwd(&pane_id, cwd("/changed")).unwrap();
        // The clone still has no cwd
        assert!(cloned.terminal_cwd.is_none());
    }

    #[test]
    fn pane_runtime_debug_format() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-debug");
        let runtime = registry.register_terminal(&pane_id, sid("s-debug"));
        let debug = format!("{runtime:?}");
        assert!(debug.contains("pane-debug"));
    }

    // ---------------------------------------------------------------------------
    // RuntimeError
    // ---------------------------------------------------------------------------

    #[test]
    fn runtime_error_not_found_message_contains_pane_id() {
        let mut registry = RuntimeRegistry::default();
        let result = registry.mark_terminal_exit(&pid("missing-pane"), None, false, None);
        match result.unwrap_err() {
            RuntimeError::NotFound(msg) => assert!(msg.contains("missing-pane")),
        }
    }

    #[test]
    fn runtime_error_browser_update_not_found() {
        let mut registry = RuntimeRegistry::default();
        let result = registry.update_browser_location(&pid("no-browser"), url("https://x.com"));
        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::NotFound(msg) => assert!(msg.contains("no-browser")),
        }
    }

    #[test]
    fn runtime_error_terminal_cwd_not_found() {
        let mut registry = RuntimeRegistry::default();
        let result = registry.update_terminal_cwd(&pid("no-cwd-pane"), cwd("/tmp"));
        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::NotFound(msg) => assert!(msg.contains("no-cwd-pane")),
        }
    }

    // ---------------------------------------------------------------------------
    // RuntimeSessionId: use in collections
    // ---------------------------------------------------------------------------

    #[test]
    fn runtime_session_id_usable_as_hashmap_key() {
        let mut map: HashMap<RuntimeSessionId, &str> = HashMap::new();
        let id = sid("map-key");
        map.insert(id.clone(), "value");
        assert_eq!(map.get(&id), Some(&"value"));
    }

    #[test]
    fn runtime_session_id_usable_in_hashset() {
        let mut set: HashSet<RuntimeSessionId> = HashSet::new();
        let id = sid("set-entry");
        set.insert(id.clone());
        assert!(set.contains(&id));
    }

    #[test]
    fn runtime_session_id_same_value_same_hash_bucket() {
        let a = sid("hs-1");
        let b = sid("hs-1");
        let mut set: HashSet<RuntimeSessionId> = HashSet::new();
        set.insert(a);
        set.insert(b);
        // Two equal entries should collapse to one in the set
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn runtime_session_id_distinct_values_in_hashset() {
        let mut set: HashSet<RuntimeSessionId> = HashSet::new();
        set.insert(sid("x"));
        set.insert(sid("y"));
        set.insert(sid("z"));
        assert_eq!(set.len(), 3);
    }

    // ---------------------------------------------------------------------------
    // Edge cases: session ID mismatch guard
    // ---------------------------------------------------------------------------

    #[test]
    fn mark_exit_session_mismatch_leaves_status_running_in_registry() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-mismatch");
        let active_session = sid("active");
        registry.register_terminal(&pane_id, active_session);

        // Attempt to exit with a stale session from a previous incarnation
        let stale = sid("stale");
        registry.mark_terminal_exit(&pane_id, Some(&stale), true, Some("crash".into())).unwrap();

        // Status in registry must remain Running
        assert_eq!(registry.get(&pane_id).unwrap().status, RuntimeStatus::Running);
    }

    #[test]
    fn mark_exit_none_session_unconditionally_updates() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-unconditional");
        registry.register_terminal(&pane_id, sid("s1"));

        registry.mark_terminal_exit(&pane_id, None, false, None).unwrap();

        assert_eq!(registry.get(&pane_id).unwrap().status, RuntimeStatus::Exited);
    }

    // ---------------------------------------------------------------------------
    // Edge cases: re-register after exit (re-spawn scenario)
    // ---------------------------------------------------------------------------

    #[test]
    fn re_register_terminal_after_exit_resets_status_to_running() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-respawn");
        let session1 = sid("session-v1");

        registry.register_terminal(&pane_id, session1.clone());
        registry.mark_terminal_exit(&pane_id, Some(&session1), false, None).unwrap();
        assert_eq!(registry.get(&pane_id).unwrap().status, RuntimeStatus::Exited);

        // Respawn: register again with a new session
        let session2 = sid("session-v2");
        registry.register_terminal(&pane_id, session2.clone());

        let r = registry.get(&pane_id).unwrap();
        assert_eq!(r.status, RuntimeStatus::Running);
        assert_eq!(r.runtime_session_id, Some(session2));
    }
}
