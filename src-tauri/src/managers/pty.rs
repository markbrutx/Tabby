use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tauri::{AppHandle, Emitter};
use tracing::warn;

use crate::domain::error::TabbyError;
use crate::domain::types::{PtyOutputEvent, ResolvedProfile};

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

        let shell = std::env::var("SHELL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| String::from("/bin/zsh"));

        let mut command = CommandBuilder::new(shell);
        command.arg("-l");
        command.env("TERM", "xterm-256color");

        if !request.cwd.trim().is_empty() {
            command.cwd(PathBuf::from(&request.cwd));
        }

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

        self.lock_sessions()?.insert(session_id.clone(), session);

        let app = self.app.clone();
        let pane_id = request.pane_id.clone();
        let session_id_for_thread = session_id.clone();
        std::thread::spawn(move || {
            let mut buffer = [0_u8; 8192];

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(size) => {
                        let chunk = String::from_utf8_lossy(&buffer[..size]).to_string();
                        if let Err(error) = app.emit(
                            "pty-output",
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
        });

        if let Some(startup_command) = request
            .profile
            .startup_command
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            self.write(&session_id, &format!("{startup_command}\n"))?;
        }

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
