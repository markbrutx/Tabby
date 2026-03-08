pub mod ids;

use std::collections::HashMap;

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
    pub pane_id: String,
    pub runtime_session_id: Option<RuntimeSessionId>,
    pub kind: RuntimeKind,
    pub status: RuntimeStatus,
    pub last_error: Option<String>,
    pub browser_location: Option<String>,
}

#[derive(Debug, Default)]
pub struct RuntimeRegistry {
    runtimes: HashMap<String, PaneRuntime>,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("runtime not found for pane {0}")]
    NotFound(String),
}

impl RuntimeRegistry {
    pub fn register_terminal(
        &mut self,
        pane_id: &str,
        runtime_session_id: RuntimeSessionId,
    ) -> PaneRuntime {
        let runtime = PaneRuntime {
            pane_id: String::from(pane_id),
            runtime_session_id: Some(runtime_session_id),
            kind: RuntimeKind::Terminal,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: None,
        };
        self.runtimes.insert(String::from(pane_id), runtime.clone());
        runtime
    }

    pub fn register_browser(
        &mut self,
        pane_id: &str,
        runtime_session_id: RuntimeSessionId,
        initial_url: String,
    ) -> PaneRuntime {
        let runtime = PaneRuntime {
            pane_id: String::from(pane_id),
            runtime_session_id: Some(runtime_session_id),
            kind: RuntimeKind::Browser,
            status: RuntimeStatus::Running,
            last_error: None,
            browser_location: Some(initial_url),
        };
        self.runtimes.insert(String::from(pane_id), runtime.clone());
        runtime
    }

    pub fn mark_terminal_exit(
        &mut self,
        pane_id: &str,
        runtime_session_id: Option<&RuntimeSessionId>,
        failed: bool,
        message: Option<String>,
    ) -> Result<PaneRuntime, RuntimeError> {
        let runtime = self
            .runtimes
            .get_mut(pane_id)
            .ok_or_else(|| RuntimeError::NotFound(String::from(pane_id)))?;

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
        pane_id: &str,
        url: String,
    ) -> Result<PaneRuntime, RuntimeError> {
        let runtime = self
            .runtimes
            .get_mut(pane_id)
            .ok_or_else(|| RuntimeError::NotFound(String::from(pane_id)))?;
        runtime.browser_location = Some(url);
        Ok(runtime.clone())
    }

    pub fn remove(&mut self, pane_id: &str) -> Option<PaneRuntime> {
        self.runtimes.remove(pane_id)
    }

    pub fn get(&self, pane_id: &str) -> Option<&PaneRuntime> {
        self.runtimes.get(pane_id)
    }

    pub fn terminal_session_id(&self, pane_id: &str) -> Option<RuntimeSessionId> {
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

    #[test]
    fn registers_terminal_and_marks_exit() {
        let mut registry = RuntimeRegistry::default();
        let session_id = RuntimeSessionId::from(String::from("session-1"));
        registry.register_terminal("pane-1", session_id.clone());

        let runtime = registry
            .mark_terminal_exit("pane-1", Some(&session_id), false, None)
            .expect("runtime should exist");
        assert_eq!(runtime.status, RuntimeStatus::Exited);
    }

    #[test]
    fn browser_location_updates() {
        let mut registry = RuntimeRegistry::default();
        registry.register_browser(
            "pane-1",
            RuntimeSessionId::from(String::from("browser-1")),
            String::from("https://example.com"),
        );

        let runtime = registry
            .update_browser_location("pane-1", String::from("https://openai.com"))
            .expect("browser runtime should exist");
        assert_eq!(
            runtime.browser_location.as_deref(),
            Some("https://openai.com")
        );
    }
}
