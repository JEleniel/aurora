use thiserror::Error;

use crate::render::svg::SvgError;

#[derive(Debug, Error)]
pub enum RenderError {
	#[error("An IO error has occurred: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Closure found with no boundaries open: {0} - {1}")]
	NoOpenBoundary(String, String),
	#[error("Closure found for the wrong boundary: expected {0}, found {1}: {2}")]
	UnmatchedBoundaryClosure(String, String, String),
	#[error("Invalid parent found for boundary: {0}")]
	InvalidParent(String),
	#[error("Invalid rood card set.")]
	InvalidRootSet,
	#[error("The Graphviz terminal command failed: {0}")]
	TerminalFailed(String),
	#[error("An error occurred parsing UTF-8: {0}")]
	Utf8Error(#[from] std::string::FromUtf8Error),
	#[error("A Serde error occurred: {0}")]
	SerdeError(#[from] serde_json::Error),
	#[error("An SVG rendering error has occurred: {0}")]
	SvgError(#[from] SvgError),
	#[error("Card not found: {0}")]
	CardNotFound(String),
}
