//! Data transfer types for the Aurora Editor backend API.

use std::collections::BTreeMap;

use aurora_shared::{Card, Link};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Workspace metadata returned to the UI.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceInfo {
	pub root: String,
	pub trusted: bool,
}

/// Summary of a discovered model home.
#[derive(Debug, Clone, Serialize)]
pub struct ModelHomeInfo {
	pub root: String,
	pub has_schema: bool,
}

/// Single card plus the path it originated from.
#[derive(Debug, Clone, Serialize)]
pub struct CardRecord {
	pub card: Card,
	pub source_path: String,
}

/// Full model snapshot returned to the UI.
#[derive(Debug, Clone, Serialize)]
pub struct ModelSnapshot {
	pub home: String,
	pub cards: Vec<CardRecord>,
}

/// Summary of a render operation for the UI.
#[derive(Debug, Clone, Serialize)]
pub struct RenderSummaryDto {
	pub cards_written: usize,
	pub views_written: usize,
	pub output_dir: String,
}

/// Request to render model artifacts.
#[derive(Debug, Clone, Deserialize)]
pub struct RenderRequest {
	pub model_home: String,
	pub output_dir: String,
}

/// Request to write the compact model export.
#[derive(Debug, Clone, Deserialize)]
pub struct CompactRequest {
	pub model_home: String,
	pub output_path: Option<String>,
}

/// Draft representation of a card used for create/update operations.
#[derive(Debug, Clone, Deserialize)]
pub struct CardDraft {
	pub id: String,
	pub card_type: String,
	#[serde(default)]
	pub card_subtype: Option<String>,
	pub name: String,
	pub description: String,
	#[serde(default)]
	pub status: Option<String>,
	#[serde(default)]
	pub links: Vec<Link>,
	#[serde(default = "default_value")]
	pub attributes: Value,
	#[serde(flatten, default)]
	pub extra: BTreeMap<String, Value>,
}

/// Request to create a new card.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCardRequest {
	pub model_home: String,
	pub relative_path: String,
	pub editor: String,
	pub card: CardDraft,
}

/// Request to update an existing card.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCardRequest {
	pub model_home: String,
	pub relative_path: String,
	pub editor: String,
	#[serde(default)]
	pub bump: Option<AuditBump>,
	pub card: CardDraft,
}

/// Request to delete (tombstone) a card.
#[derive(Debug, Clone, Deserialize)]
pub struct DeleteCardRequest {
	pub model_home: String,
	pub relative_path: String,
	pub editor: String,
	#[serde(default)]
	pub bump: Option<AuditBump>,
}

/// Semver bump types for audit trail updates.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditBump {
	Patch,
	Minor,
	Major,
}

fn default_value() -> Value {
	Value::Null
}
