//! Strongly typed representations of Aurora cards and their supporting records.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A directional relationship between two Aurora cards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuroraLink {
	/// Identifier of the card that receives the link.
	pub target: String,
	/// Verb describing how the source and target relate.
	pub relationship: String,
}

/// A single change entry inside an audit trail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
	/// Person or agent that performed the edit.
	pub editor: String,
	/// RFC3339 timestamp string describing when the edit happened.
	pub timestamp: String,
	/// Event type (created, edited, deleted, etc.).
	pub event: String,
	/// Optional cryptographic hash for offline verification.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hash: Option<String>,
}

/// Versioned audit information for a card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTrail {
	/// Semantic version for the current state.
	pub version: String,
	/// Optional overall hash for the card contents.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hash: Option<String>,
	/// Chronological history of edits to the card.
	pub history: Vec<AuditEntry>,
}

/// High-level representation of an Aurora card JSON file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuroraCard {
	/// Optional schema path to validate against.
	#[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
	pub schema: Option<String>,
	/// Unique card identifier such as `MIS-001`.
	pub id: String,
	/// Primary card type (Mission, Requirement, etc.).
	pub card_type: String,
	/// Optional subtype for more granular typing.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub card_subtype: Option<String>,
	/// Human-readable card title.
	pub name: String,
	/// Detailed description of the card.
	pub description: String,
	/// Optional status value (Proposed, Released, etc.).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub status: Option<String>,
	/// Outgoing links that describe the card relationships.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub links: Vec<AuroraLink>,
	/// Audit information that enforces deterministic history.
	pub audit_trail: AuditTrail,
	/// Additional structured metadata.
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub attributes: IndexMap<String, Value>,
}
