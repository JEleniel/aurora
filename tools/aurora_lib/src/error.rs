//! Error hierarchy shared by Aurora tooling crates.

use std::{fmt, path::PathBuf};

use thiserror::Error;

/// Error type returned by the shared Aurora library helpers.
#[derive(Debug, Error)]
pub enum AuroraLibError {
	/// Wrapper around filesystem level errors.
	#[error("failed to access {path}: {source}")]
	Io {
		/// Path that triggered the error.
		path: PathBuf,
		/// Underlying IO error.
		#[source]
		source: std::io::Error,
	},
	/// JSON serialization or deserialization failure.
	#[error("card serialization error: {0}")]
	Serialization(#[from] serde_json::Error),
	/// Deterministic validation issue (missing fields, bad identifiers, etc.).
	#[error("validation error: {0}")]
	Validation(String),
}

impl AuroraLibError {
	/// Helper for constructing IO errors with minimal boilerplate.
	pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
		Self::Io {
			path: path.into(),
			source,
		}
	}

	/// Helper for surfacing custom validation failures.
	pub(crate) fn validation(message: impl fmt::Display) -> Self {
		Self::Validation(message.to_string())
	}
}
