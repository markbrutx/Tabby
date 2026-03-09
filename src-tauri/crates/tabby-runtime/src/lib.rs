pub mod ids;

use std::collections::HashMap;

use tabby_contracts::PaneId;
use thiserror::Error;

pub use ids::RuntimeSessionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeKind {
    Terminal,
    Browser,
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
    pub browser_location: Option<String>,
    pub terminal_cwd: Option<String>,
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
        };
        self.runtimes.insert(pane_id.clone(), runtime.clone());
        runtime
    }

    pub fn register_browser(
        &mut self,
        pane_id: &PaneId,
        runtime_session_id: RuntimeSessionId,
        initial_url: String,
    ) -> PaneRuntime {
        let runtime = PaneRuntime {
            pane_id: pane_id.clone(),
            runtime_session_id: Some(runtime_session_id),
            kind: RuntimeKind::Browser,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: Some(initial_url),
            terminal_cwd: None,
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
        url: String,
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
        cwd: String,
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
    use super::{RuntimeRegistry, RuntimeSessionId, RuntimeStatus};
    use tabby_contracts::PaneId;

    fn pid(id: &str) -> PaneId {
        PaneId::from(String::from(id))
    }

    #[test]
    fn registers_terminal_and_marks_exit() {
        let mut registry = RuntimeRegistry::default();
        let session_id = RuntimeSessionId::from(String::from("session-1"));
        let pane_id = pid("pane-1");
        registry.register_terminal(&pane_id, session_id.clone());

        let runtime = registry
            .mark_terminal_exit(&pane_id, Some(&session_id), false, None)
            .expect("runtime should exist");
        assert_eq!(runtime.status, RuntimeStatus::Exited);
    }

    #[test]
    fn terminal_cwd_updates() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-1");
        registry.register_terminal(&pane_id, RuntimeSessionId::from(String::from("session-1")));

        let runtime = registry
            .update_terminal_cwd(&pane_id, String::from("/projects/tabby"))
            .expect("runtime should exist");
        assert_eq!(runtime.terminal_cwd.as_deref(), Some("/projects/tabby"));
    }

    #[test]
    fn terminal_cwd_update_for_nonexistent_pane_returns_error() {
        let mut registry = RuntimeRegistry::default();
        let result = registry.update_terminal_cwd(&pid("nonexistent"), String::from("/tmp"));
        assert!(
            result.is_err(),
            "updating cwd for nonexistent pane should fail"
        );
    }

    #[test]
    fn browser_location_updates() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-1");
        registry.register_browser(
            &pane_id,
            RuntimeSessionId::from(String::from("browser-1")),
            String::from("https://example.com"),
        );

        let runtime = registry
            .update_browser_location(&pane_id, String::from("https://openai.com"))
            .expect("browser runtime should exist");
        assert_eq!(
            runtime.browser_location.as_deref(),
            Some("https://openai.com")
        );
    }

    #[test]
    fn pane_runtime_uses_pane_id_not_raw_string() {
        let mut registry = RuntimeRegistry::default();
        let pane_id = pid("pane-typed");
        let runtime =
            registry.register_terminal(&pane_id, RuntimeSessionId::from(String::from("session-1")));
        assert_eq!(runtime.pane_id, pane_id);
        assert_eq!(runtime.pane_id.as_ref(), "pane-typed");
    }
}
