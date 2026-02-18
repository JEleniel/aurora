use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use serde_json::{Value, json};

use super::{AuditChangeType, AuditLog, AuditLogEntry};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn audit_schema(required_extra: bool) -> Value {
	let mut required = vec!["timestamp", "editor", "changes"];
	if required_extra {
		required.push("extra");
	}
	json!({
		"$schema": "http://json-schema.org/draft-07/schema#",
		"type": "object",
		"additionalProperties": true,
		"required": required,
		"properties": {
			"timestamp": { "type": "string" },
			"editor": { "type": "string" },
			"changes": { "type": "array" }
		}
	})
}

fn write_ndjson(path: &PathBuf, lines: &[&str]) -> Result<()> {
	let serialized = lines.join("\n");
	std::fs::write(path, serialized)?;
	Ok(())
}

#[test]
fn try_load_reads_audit_log() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let audit_path = temp.path().join("AuditLog.ndjson");
	write_ndjson(&audit_path, &[])?;

	let log = AuditLog::try_load(&audit_path, &audit_schema(false))?;
	assert!(log.validation_errors.is_empty());
	assert_eq!(log.history.len(), 0);
	assert_eq!(log.source_path, audit_path);
	Ok(())
}

#[test]
fn try_load_collects_schema_errors() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let audit_path = temp.path().join("AuditLog.ndjson");
	write_ndjson(
		&audit_path,
		&[r#"{"timestamp":"2026-02-07T00:00:00Z","editor":"Tester","changes":[]}"#],
	)?;

	let log = AuditLog::try_load(&audit_path, &audit_schema(true))?;
	assert!(!log.validation_errors.is_empty());
	Ok(())
}

#[test]
fn entries_for_target_filters_results() {
	let log = AuditLog {
		schema: None,
		history: vec![
			AuditLogEntry {
				timestamp: Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap(),
				editor: "Tester".to_string(),
				target: "A".to_string(),
				change_type: AuditChangeType::Create,
				changes: Vec::new(),
			},
			AuditLogEntry {
				timestamp: Utc.with_ymd_and_hms(2026, 2, 7, 1, 0, 0).unwrap(),
				editor: "Tester".to_string(),
				target: "B".to_string(),
				change_type: AuditChangeType::Change,
				changes: Vec::new(),
			},
		],
		source_path: PathBuf::new(),
		validation_errors: Vec::new(),
	};

	let entries: Vec<&AuditLogEntry> = log.entries_for_target("A").collect();
	assert_eq!(entries.len(), 1);
	assert_eq!(entries[0].target, "A");
}

#[test]
fn entries_markdown_formats_rows() {
	let entries = vec![AuditLogEntry {
		timestamp: Utc.with_ymd_and_hms(2026, 2, 7, 2, 0, 0).unwrap(),
		editor: "Tester".to_string(),
		target: "A".to_string(),
		change_type: AuditChangeType::Delete,
		changes: Vec::new(),
	}];

	let markdown = AuditLog::entries_markdown("A", entries.into_iter());
	assert!(markdown.contains("Tester"));
	assert!(markdown.contains("delete"));
}

#[test]
fn entries_markdown_handles_empty() {
	let markdown = AuditLog::entries_markdown("A", std::iter::empty::<AuditLogEntry>());
	assert!(markdown.contains("No entries for A"));
}

#[test]
fn audit_change_type_as_str_returns_labels() {
	assert_eq!(AuditChangeType::Create.as_str(), "create");
	assert_eq!(AuditChangeType::Change.as_str(), "change");
	assert_eq!(AuditChangeType::Delete.as_str(), "delete");
}
