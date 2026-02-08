use chrono::{DateTime, Utc};
use jsonschema::{CompilationError, Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
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
	pub fn try_load(path: &PathBuf, audit_schema: &Value) -> Result<Self, AuditLogError> {
		debug!("Loading audit log from {}", path.display());

		let data = std::fs::read_to_string(path).map_err(AuditLogError::IoError)?;
		let audit_json: Value = serde_json::from_str(&data)
			.map_err(|e| AuditLogError::ParseError(path.display().to_string(), e))?;
		let validation_errors = Self::validate_against_schema(&audit_json, audit_schema)?;

		let mut audit_log: AuditLog = serde_json::from_str(&data)
			.map_err(|e| AuditLogError::ParseError(path.display().to_string(), e))?;
		audit_log.source_path = path.clone();
		audit_log.validation_errors = validation_errors;
		Ok(audit_log)
	}

	pub fn entries_for_target<'a>(
		&'a self,
		target: &'a str,
	) -> impl Iterator<Item = &'a AuditLogEntry> {
		self.history.iter().filter(move |e| e.target == target)
	}

	pub fn entries_markdown(target: &str, entries: impl Iterator<Item = AuditLogEntry>) -> String {
		let mut md = String::new();
		md.push_str("| Timestamp | Editor | Change |\n");
		md.push_str("|-----------|--------|--------|\n");
		let mut any = false;
		for entry in entries {
			any = true;
			md.push_str(&format!(
				"| {} | {} | {} |\n",
				entry
					.timestamp
					.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
				entry.editor,
				entry.change_type.as_str(),
			));
		}
		if !any {
			md.push_str(&format!("| _No entries for {}_ |  |  |\n", target));
		}
		md
	}

	fn validate_against_schema(
		audit_json: &Value,
		audit_schema: &Value,
	) -> Result<Vec<String>, AuditLogError> {
		let compiled_schema = JSONSchema::options()
			.with_draft(Draft::Draft7)
			.compile(audit_schema)?;

		match compiled_schema.validate(audit_json) {
			Ok(_) => Ok(vec![]),
			Err(errors) => Ok(errors.map(|e| e.to_string()).collect()),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
	pub timestamp: DateTime<Utc>,
	pub editor: String,
	pub target: String,
	pub change_type: AuditChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditChangeType {
	#[serde(rename = "create")]
	Create,
	#[serde(rename = "change")]
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
