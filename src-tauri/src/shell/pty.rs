use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tauri::{AppHandle, Emitter};
use tracing::warn;

use tabby_contracts::TerminalOutputEvent;
use tabby_workspace::PaneId;

use crate::application::ports::TerminalProcessPort;
use crate::application::runtime_observation_receiver::RuntimeObservationReceiver;
use crate::shell::error::ShellError;
use crate::shell::TERMINAL_OUTPUT_RECEIVED_EVENT;

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

    pub fn spawn(
        &self,
        pane_id: &str,
        working_directory: &str,
        startup_command: Option<&str>,
        observation_receiver: Arc<dyn RuntimeObservationReceiver>,
    ) -> Result<String, ShellError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| ShellError::Pty(error.to_string()))?;

        let command = build_pty_command(working_directory, startup_command);

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| ShellError::Pty(error.to_string()))?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| ShellError::Pty(error.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| ShellError::Pty(error.to_string()))?;

        let runtime_session_id = uuid::Uuid::new_v4().to_string();
        let session = Arc::new(PtySession {
            writer: Mutex::new(writer),
            master: Mutex::new(pair.master),
            child: Mutex::new(child),
        });
        self.lock_sessions()?
            .insert(runtime_session_id.clone(), session.clone());

        let app = self.app.clone();
        let pane_id = String::from(pane_id);
        let runtime_session_id_for_thread = runtime_session_id.clone();
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
                            TERMINAL_OUTPUT_RECEIVED_EVENT,
                            TerminalOutputEvent {
                                pane_id: pane_id.clone(),
                                runtime_session_id: runtime_session_id_for_thread.clone(),
                                chunk,
                            },
                        ) {
                            warn!(?error, "Failed to emit terminal output event");
                            break;
                        }
                    }
                    Err(error) => {
                        warn!(?error, "PTY reader loop stopped");
                        break;
                    }
                }
            }

            let exit_code = resolve_exit_code(&session);
            let domain_pane_id = PaneId::from(pane_id);
            observation_receiver.on_terminal_exited(&domain_pane_id, exit_code);
        });

        Ok(runtime_session_id)
    }

    pub fn write(&self, runtime_session_id: &str, data: &str) -> Result<(), ShellError> {
        let session = self.get_session(runtime_session_id)?;
        let mut writer = session
            .writer
            .lock()
            .map_err(|_| ShellError::State(String::from("PTY writer lock poisoned")))?;
        writer.write_all(data.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, runtime_session_id: &str, cols: u16, rows: u16) -> Result<(), ShellError> {
        if cols == 0 || rows == 0 {
            return Ok(());
        }

        let session = self.get_session(runtime_session_id)?;
        let master = session
            .master
            .lock()
            .map_err(|_| ShellError::State(String::from("PTY master lock poisoned")))?;
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| ShellError::Pty(error.to_string()))?;
        Ok(())
    }

    pub fn kill(&self, runtime_session_id: &str) -> Result<(), ShellError> {
        let session = self
            .lock_sessions()?
            .remove(runtime_session_id)
            .ok_or_else(|| ShellError::NotFound(format!("session {runtime_session_id}")))?;
        let mut child = session
            .child
            .lock()
            .map_err(|_| ShellError::State(String::from("PTY child lock poisoned")))?;
        child
            .kill()
            .map_err(|error| ShellError::Pty(error.to_string()))?;
        Ok(())
    }

    fn get_session(&self, runtime_session_id: &str) -> Result<Arc<PtySession>, ShellError> {
        self.lock_sessions()?
            .get(runtime_session_id)
            .cloned()
            .ok_or_else(|| ShellError::NotFound(format!("session {runtime_session_id}")))
    }

    fn lock_sessions(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, Arc<PtySession>>>, ShellError> {
        self.sessions
            .lock()
            .map_err(|_| ShellError::State(String::from("PTY sessions lock poisoned")))
    }
}

impl TerminalProcessPort for PtyManager {
    fn spawn(
        &self,
        pane_id: &str,
        working_directory: &str,
        startup_command: Option<&str>,
        observation_receiver: Arc<dyn RuntimeObservationReceiver>,
    ) -> Result<String, ShellError> {
        PtyManager::spawn(
            self,
            pane_id,
            working_directory,
            startup_command,
            observation_receiver,
        )
    }

    fn kill(&self, runtime_session_id: &str) -> Result<(), ShellError> {
        PtyManager::kill(self, runtime_session_id)
    }

    fn resize(&self, runtime_session_id: &str, cols: u16, rows: u16) -> Result<(), ShellError> {
        PtyManager::resize(self, runtime_session_id, cols, rows)
    }

    fn write_input(&self, runtime_session_id: &str, data: &str) -> Result<(), ShellError> {
        PtyManager::write(self, runtime_session_id, data)
    }
}

fn default_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| String::from("/bin/zsh"))
}

fn build_pty_command(working_directory: &str, startup_command: Option<&str>) -> CommandBuilder {
    let shell = default_shell();
    let startup_command = startup_command
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let mut command = match startup_command {
        Some(cmd) => {
            let mut builder = CommandBuilder::new(&shell);
            builder.arg("-l");
            builder.arg("-c");
            builder.arg(format!("exec {cmd}"));
            builder
        }
        None => {
            let mut builder = CommandBuilder::new(&shell);
            builder.arg("-l");
            builder
        }
    };

    command.env("TERM", "xterm-256color");
    command.env_remove("CLAUDECODE");
    command.env_remove("CLAUDE_CODE_ENTRYPOINT");
    if !working_directory.trim().is_empty() {
        command.cwd(PathBuf::from(working_directory));
    }
    command
}

fn extract_valid_utf8(carry: &mut Vec<u8>, chunk: &[u8]) -> String {
    carry.extend_from_slice(chunk);
    match std::str::from_utf8(carry) {
        Ok(value) => {
            let output = String::from(value);
            carry.clear();
            output
        }
        Err(error) => {
            let valid_up_to = error.valid_up_to();
            let output = String::from_utf8_lossy(&carry[..valid_up_to]).to_string();
            let rest = carry[valid_up_to..].to_vec();
            *carry = rest;
            output
        }
    }
}

/// Extracts the exit code from the PTY child process.
/// Returns `None` if the status could not be determined (lock poisoned, still running, etc.).
fn resolve_exit_code(session: &PtySession) -> Option<i32> {
    let mut child = session.child.lock().ok()?;
    match child.try_wait() {
        Ok(Some(exit_status)) => {
            let code = exit_status.exit_code();
            Some(i32::try_from(code).unwrap_or(i32::MAX))
        }
        Ok(None) => None,
        Err(_) => None,
    }
}
