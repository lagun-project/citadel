//! Error types for citadel-protocols.

use thiserror::Error;

/// Result type for citadel-protocols operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during protocol operations.
#[derive(Debug, Error)]
pub enum Error {
    /// The underlying TGP protocol encountered an error.
    #[error("TGP protocol error: {0}")]
    Protocol(#[from] two_generals::Error),

    /// The coordinator is in an invalid state for the requested operation.
    #[error("invalid coordinator state: expected {expected}, got {actual}")]
    InvalidState {
        expected: &'static str,
        actual: String,
    },

    /// The coordinator has already been aborted.
    #[error("coordinator has been aborted")]
    Aborted,

    /// Message validation failed.
    #[error("invalid message: {0}")]
    InvalidMessage(String),

    /// Timeout waiting for coordination.
    #[error("coordination timeout after {0:?}")]
    Timeout(std::time::Duration),
}
