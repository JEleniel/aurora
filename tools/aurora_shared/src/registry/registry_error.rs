use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
	#[error("Failed to parse registry JSON: {0}")]
	ParseError(#[from] serde_json::Error),
	#[error("Card type not found: {0}")]
	CardTypeNotFound(String),
	#[error("Card acronym not found: {0}")]
	CardAcronymNotFound(String),
	#[error("Appearance definition not found for acronym: {0}")]
	MissingAppearance(String),
	#[error("Appearance provided for unknown acronym: {0}")]
	UnknownAppearanceAcronym(String),
	#[error("Duplicate appearance acronym in view configuration: {0}")]
	DuplicateAppearanceAcronym(String),
	#[error("Relationship source not found for acronym: {0}")]
	UnknownRelationshipSource(String),
	#[error("Relationship target not found for acronym: {0}")]
	UnknownRelationshipTarget(String),
	#[error("Duplicate card acronym in model configuration: {0}")]
	DuplicateCardAcronym(String),
	#[error("Card acronym assigned to more than one domain path: {0}")]
	DuplicateDomainAssignment(String),
	#[error("Card acronym referenced by domains but not defined in the view configuration: {0}")]
	UnknownDomainCardAcronym(String),
	#[error("Unknown icon id referenced by model configuration: {0}")]
	UnknownIcon(String),
}
