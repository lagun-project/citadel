//! Error types for Lens.

use thiserror::Error;

/// Result type for Lens operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in Lens operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<rocksdb::Error> for Error {
    fn from(e: rocksdb::Error) -> Self {
        Error::Storage(e.to_string())
    }
}
