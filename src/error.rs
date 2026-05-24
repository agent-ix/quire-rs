//! Top-level error type (Task 019).

use thiserror::Error;

/// Top-level error type for the quire-rs crate.
#[derive(Debug, Error)]
pub enum QuireError {
    /// Placeholder variant; real variants are added as tasks land.
    #[error("quire-rs is not yet implemented")]
    Unimplemented,
}
