//! Error types for the Aurora editor.

use std::path::PathBuf;

use aurora_shared::AuroraError;
use thiserror::Error;

/// Errors emitted by the Aurora editor.
#[derive(Debug, Error)]
pub enum EditorError {
	#[error("Failed to initialize logging: {0}")]
	Logging(String),

	#[error("Aurora shared error: {0}")]
	Aurora(#[from] AuroraError),

	#[error("No Aurora model home could be discovered from {path}")]
	ModelHomeNotFound { path: PathBuf },

	#[error("Mission card not found in the loaded model")]
	MissingMission,
}
