use std::path::PathBuf;

use aurora_shared::AuroraError;
use thiserror::Error;

pub type BackendResult<T> = Result<T, BackendError>;

#[derive(Debug, Error)]
pub enum BackendError {
	#[error("no model is currently loaded")]
	NoSession,

	#[error("card with id '{0}' was not found in the active model")]
	CardNotFound(String),

	#[error("card '{0}' does not have a resolvable source path on disk")]
	MissingCardSource(String),

	#[error("unable to parse or serialize data: {0}")]
	Serde(#[from] serde_json::Error),

	#[error("I/O error while accessing {path}: {source}")]
	Io {
		path: PathBuf,
		#[source]
		source: std::io::Error,
	},

	#[error(transparent)]
	Aurora(#[from] AuroraError),

	#[error("{message}")]
	Other { message: String },
}

impl BackendError {
	pub fn custom(message: impl Into<String>) -> Self {
		BackendError::Other {
			message: message.into(),
		}
	}

	pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
		BackendError::Io {
			path: path.into(),
			source,
		}
	}
}
