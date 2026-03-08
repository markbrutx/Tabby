use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Type)]
pub enum ShellError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("state error: {0}")]
    State(String),
    #[error("item not found: {0}")]
    NotFound(String),
    #[error("PTY error: {0}")]
    Pty(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Store error: {0}")]
    Store(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

impl Serialize for ShellError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl From<std::io::Error> for ShellError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
