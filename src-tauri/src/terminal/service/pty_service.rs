use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, PtySize};
use tauri::{AppHandle, Emitter};
use tracing::warn;

use crate::settings::domain::profiles::ResolvedProfile;
use crate::shared::error::TabbyError;
use crate::shared::events::{
    PaneLifecycleEvent, PtyOutputEvent, PANE_LIFECYCLE_EVENT_NAME, PTY_OUTPUT_EVENT_NAME,
};
use crate::terminal::service::command_builder::build_pty_command;
use crate::terminal::service::utf8_decoder::extract_valid_utf8;
use crate::workspace::domain::snapshot::PaneRuntimeStatus;

#[derive(Debug, Clone)]
pub struct SpawnRequest {
    pub pane_id: String,
    pub cwd: String,
    pub profile: ResolvedProfile,
}

struct PtySession {
    writer: Mutex<Box<dyn Write + Send>>,
    master: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    child: Mutex<Box<dyn portable_pty::Child + Send>>,
}

#[derive(Clone)]
pub struct PtyManager {
    app: AppHandle,
    sessions: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
}

impl std::fmt::Debug for PtyManager {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("PtyManager").finish_non_exhaustive()
    }
}

impl PtyManager {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn spawn(&self, request: SpawnRequest) -> Result<String, TabbyError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| TabbyError::Pty(error.to_string()))?;

        let command = build_pty_command(&request.cwd, &request.profile);

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| TabbyError::Pty(error.to_string()))?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| TabbyError::Pty(error.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| TabbyError::Pty(error.to_string()))?;

        let session_id = uuid::Uuid::new_v4().to_string();
        let session = Arc::new(PtySession {
            writer: Mutex::new(writer),
            master: Mutex::new(pair.master),
            child: Mutex::new(child),
        });

        let session_for_thread = session.clone();
        self.lock_sessions()?.insert(session_id.clone(), session);

        let app = self.app.clone();
        let pane_id = request.pane_id.clone();
        let session_id_for_thread = session_id.clone();
        std::thread::spawn(move || {
            let mut buffer = [0_u8; 8192];
            let mut carry = Vec::new();

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(size) => {
                        let chunk = extract_valid_utf8(&mut carry, &buffer[..size]);
                        if chunk.is_empty() {
                            continue;
                        }
                        if let Err(error) = app.emit(
                            PTY_OUTPUT_EVENT_NAME,
                            PtyOutputEvent {
                                pane_id: pane_id.clone(),
                                session_id: session_id_for_thread.clone(),
                                chunk,
                            },
                        ) {
                            warn!(?error, "Failed to emit PTY output event");
                            break;
                        }
                    }
                    Err(error) => {
                        warn!(?error, "PTY reader loop stopped");
                        break;
                    }
                }
            }

            // Reader loop ended — process exited or errored. Emit lifecycle event.
            let event = build_exit_event(&pane_id, &session_id_for_thread, &session_for_thread);
            if let Err(error) = app.emit(PANE_LIFECYCLE_EVENT_NAME, event) {
                warn!(?error, "Failed to emit exit lifecycle event");
            }
        });

        Ok(session_id)
    }

    pub fn write(&self, session_id: &str, data: &str) -> Result<(), TabbyError> {
        let session = self.get_session(session_id)?;
        let mut writer = session
            .writer
            .lock()
            .map_err(|_| TabbyError::State(String::from("PTY writer lock poisoned")))?;
        writer.write_all(data.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), TabbyError> {
        if cols == 0 || rows == 0 {
            return Ok(());
        }

        let session = self.get_session(session_id)?;
        let master = session
            .master
            .lock()
            .map_err(|_| TabbyError::State(String::from("PTY master lock poisoned")))?;
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| TabbyError::Pty(error.to_string()))?;
        Ok(())
    }

    pub fn kill(&self, session_id: &str) -> Result<(), TabbyError> {
        let session = self
            .lock_sessions()?
            .remove(session_id)
            .ok_or_else(|| TabbyError::NotFound(format!("Session {session_id}")))?;
        let mut child = session
            .child
            .lock()
            .map_err(|_| TabbyError::State(String::from("PTY child lock poisoned")))?;
        child
            .kill()
            .map_err(|error| TabbyError::Pty(error.to_string()))?;
        Ok(())
    }

    pub fn kill_many(&self, session_ids: &[String]) {
        for session_id in session_ids {
            if let Err(error) = self.kill(session_id) {
                warn!(?error, session_id, "Failed to kill PTY session");
            }
        }
    }

    fn get_session(&self, session_id: &str) -> Result<Arc<PtySession>, TabbyError> {
        self.lock_sessions()?
            .get(session_id)
            .cloned()
            .ok_or_else(|| TabbyError::NotFound(format!("Session {session_id}")))
    }

    fn lock_sessions(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, Arc<PtySession>>>, TabbyError> {
        self.sessions
            .lock()
            .map_err(|_| TabbyError::State(String::from("PTY sessions lock poisoned")))
    }
}

fn build_exit_event(pane_id: &str, session_id: &str, session: &PtySession) -> PaneLifecycleEvent {
    let (status, error_message) = match session.child.lock() {
        Ok(mut child) => match child.try_wait() {
            Ok(Some(exit_status)) => {
                if exit_status.success() {
                    (PaneRuntimeStatus::Exited, None)
                } else {
                    let code = exit_status.exit_code();
                    let msg = format!("Process exited with code {code}");
                    (PaneRuntimeStatus::Failed, Some(msg))
                }
            }
            Ok(None) => {
                // Still running (shouldn't happen after reader EOF, but be safe)
                (PaneRuntimeStatus::Exited, None)
            }
            Err(_) => (
                PaneRuntimeStatus::Failed,
                Some(String::from("Failed to read process exit status")),
            ),
        },
        Err(_) => (
            PaneRuntimeStatus::Failed,
            Some(String::from("Child lock poisoned")),
        ),
    };

    PaneLifecycleEvent {
        pane_id: String::from(pane_id),
        session_id: Some(String::from(session_id)),
        status,
        error_message,
    }
}
