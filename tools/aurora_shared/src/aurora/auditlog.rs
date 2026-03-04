use chrono::{DateTime, Utc};
use jsonschema::{CompilationError, Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
	#[serde(rename = "$schema", default)]
	pub schema: Option<String>,
	pub history: Vec<AuditLogEntry>,
	#[serde(skip)]
	pub source_path: PathBuf,
	#[serde(skip)]
	pub validation_errors: Vec<String>,
}

impl AuditLog {
	pub fn try_load(path: &Path, audit_schema: &Value) -> Result<Self, AuditLogError> {
		debug!("Loading audit log from {}", path.display());

		let data = std::fs::read_to_string(path)?;
		let mut validation_errors: Vec<String> = Vec::new();
		let mut history: Vec<AuditLogEntry> = Vec::new();
		let compiled_schema = JSONSchema::options()
			.with_draft(Draft::Draft7)
			.compile(audit_schema)?;

		for (line_index, line) in data.lines().enumerate() {
			let line_number = line_index + 1;
			let trimmed = line.trim();
			if trimmed.is_empty() {
				continue;
			}

			let parsed: Value = match serde_json::from_str(trimmed) {
				Ok(parsed) => parsed,
				Err(error) => {
					validation_errors.push(format!(
						"Line {}: failed to parse audit entry: {}",
						line_number, error
					));
					continue;
				}
			};

			if let Err(errors) = compiled_schema.validate(&parsed) {
				for error in errors {
					validation_errors.push(format!("Line {}: {}", line_number, error));
				}
			}

			match serde_json::from_value::<AuditLogEntry>(parsed) {
				Ok(entry) => history.push(entry),
				Err(error) => validation_errors.push(format!(
					"Line {}: failed to decode audit entry: {}",
					line_number, error
				)),
			}
		}

		Ok(AuditLog {
			schema: None,
			history,
			source_path: path.to_path_buf(),
			validation_errors,
		})
	}

	pub fn entries_for_target(&self, target: &str) -> impl Iterator<Item = &AuditLogEntry> {
		self.history.iter().filter(move |entry| {
			entry.changes.iter().any(|change| change.card_id == target) || entry.target == target
		})
	}

	pub fn change_summary_for_target(entry: &AuditLogEntry, target: &str) -> String {
		let mut changes: Vec<&str> = entry
			.changes
			.iter()
			.filter(|change| change.card_id == target)
			.map(|change| change.change_type.as_str())
			.collect();
		if changes.is_empty() && entry.target == target {
			changes.push(entry.change_type.as_str());
		}
		changes.sort_unstable();
		changes.dedup();
		if changes.is_empty() {
			"change".to_string()
		} else {
			changes.join(",")
		}
	}

	pub fn entries_markdown(target: &str, entries: impl Iterator<Item = AuditLogEntry>) -> String {
		let mut md = String::new();
		md.push_str("| Timestamp | Editor | Change |\n");
		md.push_str("|-----------|--------|--------|\n");
		let mut any = false;
		for entry in entries {
			any = true;
			let change_summary = Self::change_summary_for_target(&entry, target);
			md.push_str(&format!(
				"| {} | {} | {} |\n",
				entry
					.timestamp
					.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
				entry.editor,
				change_summary,
			));
		}
		if !any {
			md.push_str(&format!("| _No entries for {}_ |  |  |\n", target));
		}
		md
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
	pub timestamp: DateTime<Utc>,
	pub editor: String,
	#[serde(default)]
	pub target: String,
	#[serde(default)]
	pub change_type: AuditChangeType,
	#[serde(default)]
	pub changes: Vec<AuditCardChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditCardChange {
	pub card_id: String,
	pub change_type: AuditChangeType,
	#[serde(default)]
	pub link_changes: Vec<AuditLinkChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLinkChange {
	pub change_type: AuditChangeType,
	pub relationship: String,
	pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AuditChangeType {
	#[serde(rename = "create")]
	Create,
	#[serde(rename = "change")]
	#[default]
	Change,
	#[serde(rename = "delete")]
	Delete,
}

impl AuditChangeType {
	pub fn as_str(&self) -> &'static str {
		match self {
			AuditChangeType::Create => "create",
			AuditChangeType::Change => "change",
			AuditChangeType::Delete => "delete",
		}
	}
}

#[derive(Debug, Error)]
pub enum AuditLogError {
	#[error("I/O error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Parse error at {0}: {1}")]
	ParseError(String, serde_json::Error),
	#[error("Schema compilation error: {0}")]
	SchemaCompilationError(#[from] CompilationError),
}

#[cfg(test)]
#[path = "auditlog_tests.rs"]
mod auditlog_tests;
