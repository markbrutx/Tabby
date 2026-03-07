use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tauri::{AppHandle, Emitter};
use tracing::warn;

use crate::domain::error::TabbyError;
use crate::domain::events::{
    PaneLifecycleEvent, PtyOutputEvent, PANE_LIFECYCLE_EVENT_NAME, PTY_OUTPUT_EVENT_NAME,
};
use crate::domain::snapshot::PaneRuntimeStatus;
use crate::domain::types::ResolvedProfile;

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

fn default_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| String::from("/bin/zsh"))
}

fn build_pty_command(cwd: &str, profile: &ResolvedProfile) -> CommandBuilder {
    let shell = default_shell();

    let startup_command = profile
        .startup_command
        .as_deref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty());

    let mut command = match startup_command {
        Some(cmd) => {
            let mut cb = CommandBuilder::new(&shell);
            cb.arg("-l");
            cb.arg("-c");
            cb.arg(format!("exec {cmd}"));
            cb
        }
        None => {
            let mut cb = CommandBuilder::new(&shell);
            cb.arg("-l");
            cb
        }
    };

    command.env("TERM", "xterm-256color");
    command.env_remove("CLAUDECODE");
    command.env_remove("CLAUDE_CODE_ENTRYPOINT");

    if !cwd.trim().is_empty() {
        command.cwd(PathBuf::from(cwd));
    }

    command
}

/// Extracts valid UTF-8 from raw bytes, carrying incomplete sequences across reads.
///
/// Prepends any leftover `carry` bytes from the previous read, scans for an
/// incomplete multi-byte sequence at the tail, moves it into `carry`, and
/// returns the valid UTF-8 prefix as a `String`.
fn extract_valid_utf8(carry: &mut Vec<u8>, raw: &[u8]) -> String {
    let mut buf = std::mem::take(carry);
    buf.extend_from_slice(raw);

    if buf.is_empty() {
        return String::new();
    }

    // Find the boundary: walk backwards to detect an incomplete trailing sequence.
    let valid_up_to = match std::str::from_utf8(&buf) {
        Ok(_) => buf.len(),
        Err(error) => {
            // If there's an error length, the bytes are truly invalid (not just
            // incomplete). We still split at the valid boundary — the invalid
            // bytes become carry and will be re-evaluated with the next read.
            error.valid_up_to()
        }
    };

    let remainder = buf.split_off(valid_up_to);
    *carry = remainder;

    // SAFETY: we split at a validated UTF-8 boundary.
    unsafe { String::from_utf8_unchecked(buf) }
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

#[cfg(test)]
mod tests {
    use super::{build_pty_command, extract_valid_utf8};
    use crate::domain::types::ResolvedProfile;

    #[test]
    fn ascii_only_no_carry() {
        let mut carry = Vec::new();
        let result = extract_valid_utf8(&mut carry, b"hello world");
        assert_eq!(result, "hello world");
        assert!(carry.is_empty());
    }

    #[test]
    fn complete_multibyte_passes_through() {
        let mut carry = Vec::new();
        let emoji = "\u{1F600}"; // 4-byte char
        let result = extract_valid_utf8(&mut carry, emoji.as_bytes());
        assert_eq!(result, emoji);
        assert!(carry.is_empty());
    }

    #[test]
    fn split_at_multibyte_boundary() {
        let mut carry = Vec::new();
        // \u{00E9} = 0xC3 0xA9 (2 bytes). Send only first byte.
        let result = extract_valid_utf8(&mut carry, &[0xC3]);
        assert_eq!(result, "");
        assert_eq!(carry, vec![0xC3]);
    }

    #[test]
    fn carry_prepended_to_next_read() {
        let mut carry = vec![0xC3]; // leftover from previous read
        let result = extract_valid_utf8(&mut carry, &[0xA9]); // completes \u{00E9}
        assert_eq!(result, "\u{00E9}");
        assert!(carry.is_empty());
    }

    #[test]
    fn empty_buffer_returns_empty() {
        let mut carry = Vec::new();
        let result = extract_valid_utf8(&mut carry, &[]);
        assert_eq!(result, "");
        assert!(carry.is_empty());
    }

    #[test]
    fn three_byte_char_split_after_first_byte() {
        // \u{4E16} = 0xE4 0xB8 0x96 ("世")
        let mut carry = Vec::new();
        let result = extract_valid_utf8(&mut carry, &[b'A', 0xE4]);
        assert_eq!(result, "A");
        assert_eq!(carry, vec![0xE4]);

        let result = extract_valid_utf8(&mut carry, &[0xB8, 0x96, b'B']);
        assert_eq!(result, "\u{4E16}B");
        assert!(carry.is_empty());
    }

    #[test]
    fn terminal_profile_builds_login_shell() {
        let profile = ResolvedProfile {
            id: String::from("terminal"),
            label: String::from("Terminal"),
            startup_command: None,
        };
        let cmd = build_pty_command("/tmp", &profile);
        let argv = cmd.get_argv();
        // Should be [shell, -l] with no -c
        assert_eq!(argv.len(), 2);
        assert_eq!(argv[1].to_str().unwrap(), "-l");
    }

    #[test]
    fn profile_with_command_builds_shell_c() {
        let profile = ResolvedProfile {
            id: String::from("claude"),
            label: String::from("Claude Code"),
            startup_command: Some(String::from("claude")),
        };
        let cmd = build_pty_command("/tmp", &profile);
        let argv = cmd.get_argv();
        assert_eq!(argv.len(), 4);
        assert_eq!(argv[1].to_str().unwrap(), "-l");
        assert_eq!(argv[2].to_str().unwrap(), "-c");
        assert_eq!(argv[3].to_str().unwrap(), "exec claude");
    }

    #[test]
    fn empty_command_uses_login_shell() {
        let profile = ResolvedProfile {
            id: String::from("terminal"),
            label: String::from("Terminal"),
            startup_command: Some(String::from("  ")),
        };
        let cmd = build_pty_command("/tmp", &profile);
        let argv = cmd.get_argv();
        assert_eq!(argv.len(), 2);
        assert_eq!(argv[1].to_str().unwrap(), "-l");
    }

    #[test]
    fn env_vars_are_set() {
        let profile = ResolvedProfile {
            id: String::from("terminal"),
            label: String::from("Terminal"),
            startup_command: None,
        };
        let cmd = build_pty_command("/tmp", &profile);
        let term = cmd.get_env("TERM");
        assert_eq!(term.map(|v| v.to_str().unwrap()), Some("xterm-256color"));
    }

    #[test]
    fn four_byte_char_split_across_three_reads() {
        // \u{1F600} = 0xF0 0x9F 0x98 0x80
        let mut carry = Vec::new();

        let r1 = extract_valid_utf8(&mut carry, &[0xF0, 0x9F]);
        assert_eq!(r1, "");
        assert_eq!(carry.len(), 2);

        let r2 = extract_valid_utf8(&mut carry, &[0x98]);
        assert_eq!(r2, "");
        assert_eq!(carry.len(), 3);

        let r3 = extract_valid_utf8(&mut carry, &[0x80]);
        assert_eq!(r3, "\u{1F600}");
        assert!(carry.is_empty());
    }
}
