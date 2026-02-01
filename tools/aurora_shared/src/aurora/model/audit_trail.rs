use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditTrail {
	pub version: String,
	pub hash: Option<String>,
	pub history: Vec<AuditHistoryEntry>,
}

impl Default for AuditTrail {
	fn default() -> Self {
		AuditTrail {
			version: "1.0.0".to_string(),
			hash: None,
			history: Vec::new(),
		}
	}
}

impl AuditTrail {
	pub fn get_history_markdown(&self) -> String {
		let mut md = String::new();
		md.push_str("| Timestamp | Editor | Event |\n");
		md.push_str("|-----------|--------|-------|\n");
		for entry in &self.history {
			md.push_str(&format!(
				"| {} | {} | {} |\n",
				entry
					.timestamp
					.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
				entry.editor,
				entry.event.as_str()
			));
		}
		md
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditHistoryEntry {
	pub timestamp: DateTime<Utc>,
	pub editor: String,
	pub event: AuditHistoryEvent,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuditHistoryEvent {
	#[serde(rename = "created")]
	Created,
	#[serde(rename = "edited")]
	Edited,
	#[serde(rename = "deleted")]
	Deleted,
}

impl AuditHistoryEvent {
	pub fn as_str(&self) -> &str {
		match self {
			AuditHistoryEvent::Created => "created",
			AuditHistoryEvent::Edited => "edited",
			AuditHistoryEvent::Deleted => "deleted",
		}
	}
}

impl TryFrom<String> for AuditHistoryEvent {
	type Error = AuditHistoryError;

	fn try_from(event: String) -> Result<Self, Self::Error> {
		match event.to_lowercase().as_str() {
			"created" => Ok(AuditHistoryEvent::Created),
			"edited" => Ok(AuditHistoryEvent::Edited),
			"deleted" => Ok(AuditHistoryEvent::Deleted),
			_ => {
				return Err(AuditHistoryError::InvalidEvent(event));
			}
		}
	}
}

impl Into<&str> for AuditHistoryEvent {
	fn into(self) -> &'static str {
		match self {
			AuditHistoryEvent::Created => "created",
			AuditHistoryEvent::Edited => "edited",
			AuditHistoryEvent::Deleted => "deleted",
		}
	}
}

#[derive(Debug, Error)]
pub enum AuditHistoryError {
	#[error("Invalid audit history event: {0}")]
	InvalidEvent(String),
}
