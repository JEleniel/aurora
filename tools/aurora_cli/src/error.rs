//! Error types for aurora_cli operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur while loading, validating, or rendering Aurora models.
#[derive(Debug, Error)]
pub enum AuroraCliError {
	#[error("model path does not exist: {0}")]
	MissingModelPath(PathBuf),
	#[error("model root is not a directory: {0}")]
	InvalidModelRoot(PathBuf),
	#[error("mission file is missing a parent directory: {0}")]
	MissionHasNoParent(PathBuf),
	#[error("failed to canonicalize {path}: {source}")]
	Canonicalize {
		path: PathBuf,
		#[source]
		source: std::io::Error,
	},
	#[error("failed to read {path}: {source}")]
	Io {
		path: PathBuf,
		#[source]
		source: std::io::Error,
	},
	#[error("failed to parse JSON at {path}: {source}")]
	Json {
		path: PathBuf,
		#[source]
		source: serde_json::Error,
	},
	#[error("duplicate card id {id} at {path}")]
	DuplicateCardId { id: String, path: PathBuf },
	#[error("missing Aurora schema copy at {path}")]
	MissingSchema { path: PathBuf },
	#[error("model root {root} does not contain any cards")]
	NoCards { root: PathBuf },
	#[error("model root {root} must contain exactly one Mission card")]
	MissingMission { root: PathBuf },
	#[error("model root {root} has multiple Mission cards: {ids:?}")]
	MultipleMissions { root: PathBuf, ids: Vec<String> },
	#[error("failed to compile schema at {path}: {message}")]
	SchemaCompilation { path: PathBuf, message: String },
	#[error("output path escapes configured root {root}: attempted {attempted}")]
	OutputEscapesRoot { root: PathBuf, attempted: PathBuf },
	#[error("unable to compute relative path from {from} to {to}")]
	RelativePath { from: PathBuf, to: PathBuf },
}
