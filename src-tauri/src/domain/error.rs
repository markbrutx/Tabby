use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TabbyError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("State error: {0}")]
    State(String),
    #[error("Workspace item not found: {0}")]
    NotFound(String),
    #[error("PTY error: {0}")]
    Pty(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Store error: {0}")]
    Store(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl Serialize for TabbyError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
