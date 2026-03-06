use std::sync::Arc;

use tauri::State;

use crate::domain::error::TabbyError;
use crate::domain::types::PtyResizeRequest;
use crate::managers::pty::PtyManager;
use crate::managers::tab::TabManager;

#[tauri::command]
pub fn write_pty(
    tab_manager: State<'_, Arc<TabManager>>,
    pty_manager: State<'_, Arc<PtyManager>>,
    pane_id: String,
    data: String,
) -> Result<(), TabbyError> {
    let session_id = tab_manager.session_id_for_pane(&pane_id)?;
    pty_manager.write(&session_id, &data)
}

#[tauri::command]
pub fn resize_pty(
    tab_manager: State<'_, Arc<TabManager>>,
    pty_manager: State<'_, Arc<PtyManager>>,
    request: PtyResizeRequest,
) -> Result<(), TabbyError> {
    let session_id = tab_manager.session_id_for_pane(&request.pane_id)?;
    pty_manager.resize(&session_id, request.cols, request.rows)
}
