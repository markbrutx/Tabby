use std::sync::Arc;

use tauri::State;

use crate::application::coordinator::Coordinator;
use crate::shared::error::TabbyError;
use crate::workspace::domain::requests::PtyResizeRequest;

#[tauri::command]
#[specta::specta]
pub fn write_pty(
    coordinator: State<'_, Arc<Coordinator>>,
    pane_id: String,
    data: String,
) -> Result<(), TabbyError> {
    coordinator.write_pty(&pane_id, &data)
}

#[tauri::command]
#[specta::specta]
pub fn resize_pty(
    coordinator: State<'_, Arc<Coordinator>>,
    request: PtyResizeRequest,
) -> Result<(), TabbyError> {
    coordinator.resize_pty(&request.pane_id, request.cols, request.rows)
}
