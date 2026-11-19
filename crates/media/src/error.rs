use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum MediaError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Decode error: {0}")]
    Decode(String),

    #[error("Hardware acceleration error: {0}")]
    HwAccel(String),

    #[error("Unsupported format or codec: {0}")]
    Unsupported(String),

    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Audio device error: {0}")]
    AudioDevice(String),

    #[error("Out of memory: requested buffer exceeds memory budget ({0} bytes)")]
    OutOfMemory(usize),

    #[error("Operation cancelled")]
    Cancelled,
}

pub type MediaResult<T> = Result<T, MediaError>;
