use std::path::PathBuf;

use thiserror::Error;

/// Convenient alias for results produced by the shared Aurora library.
pub type Result<T, E = AuroraError> = std::result::Result<T, E>;

/// Error type representing all recoverable failures emitted by the shared library.
#[derive(Debug, Error)]
pub enum AuroraError {
	#[error("input path does not exist: {path}")]
	MissingInput { path: PathBuf },

	#[error("invalid input: {message}")]
	InvalidInput { message: String },

	#[error("I/O error while accessing {path}: {source}")]
	Io {
		#[source]
		source: std::io::Error,
		path: PathBuf,
	},

	#[error("failed to parse JSON in {path}: {source}")]
	Parse {
		#[source]
		source: serde_json::Error,
		path: PathBuf,
	},
}

impl AuroraError {
	pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
		AuroraError::Io {
			source,
			path: path.into(),
		}
	}

	pub(crate) fn parse(path: impl Into<PathBuf>, source: serde_json::Error) -> Self {
		AuroraError::Parse {
			source,
			path: path.into(),
		}
	}
}
