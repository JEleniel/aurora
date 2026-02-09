use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
	#[error("Failed to parse registry JSON: {0}")]
	ParseError(#[from] serde_json::Error),
	#[error("Card type not found: {0}")]
	CardTypeNotFound(String),
	#[error("Card acronym not found: {0}")]
	CardAcronymNotFound(String),
}
