use tabby_contracts::{PaneRuntimeView, RuntimeKindDto, RuntimeStatusDto};
use tabby_runtime::{PaneRuntime, RuntimeKind, RuntimeStatus};

// ---------------------------------------------------------------------------
// Domain → DTO (outbound / projections)
// ---------------------------------------------------------------------------

pub(crate) fn pane_runtime_to_view(runtime: &PaneRuntime) -> PaneRuntimeView {
    PaneRuntimeView {
        pane_id: runtime.pane_id.to_string(),
        runtime_session_id: runtime.runtime_session_id.as_ref().map(|id| id.to_string()),
        kind: runtime_kind_to_dto(runtime.kind),
        status: runtime_status_to_dto(runtime.status),
        last_error: runtime.last_error.clone(),
        browser_location: runtime
            .browser_location
            .as_ref()
            .map(|u| u.as_str().to_string()),
        terminal_cwd: runtime
            .terminal_cwd
            .as_ref()
            .map(|w| w.as_str().to_string()),
        git_repo_path: runtime
            .git_repo_path
            .as_ref()
            .map(|w| w.as_str().to_string()),
    }
}

pub(crate) fn runtime_kind_to_dto(value: RuntimeKind) -> RuntimeKindDto {
    match value {
        RuntimeKind::Terminal => RuntimeKindDto::Terminal,
        RuntimeKind::Browser => RuntimeKindDto::Browser,
        RuntimeKind::Git => RuntimeKindDto::Git,
    }
}

pub(crate) fn runtime_status_to_dto(value: RuntimeStatus) -> RuntimeStatusDto {
    match value {
        RuntimeStatus::Starting => RuntimeStatusDto::Starting,
        RuntimeStatus::Running => RuntimeStatusDto::Running,
        RuntimeStatus::Exited => RuntimeStatusDto::Exited,
        RuntimeStatus::Failed => RuntimeStatusDto::Failed,
    }
}
