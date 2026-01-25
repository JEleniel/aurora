//! Audit trail helpers for editor mutations.

use aurora_shared::{AuditEvent, AuditTrail};
use chrono::{SecondsFormat, Utc};

use crate::types::AuditBump;

/// Supported audit events for card history.
#[derive(Debug, Clone, Copy)]
pub enum AuditEventKind {
	Created,
	Edited,
	Deleted,
}

impl AuditEventKind {
	pub fn as_str(self) -> &'static str {
		match self {
			AuditEventKind::Created => "created",
			AuditEventKind::Edited => "edited",
			AuditEventKind::Deleted => "deleted",
		}
	}
}

/// Apply a version bump and append a new audit history entry.
pub fn apply_audit_event(
	audit: &mut AuditTrail,
	bump: AuditBump,
	editor: &str,
	event: AuditEventKind,
) -> Result<(), String> {
	let new_version = bump_version(&audit.version, bump)?;
	audit.version = new_version;
	audit.history.push(AuditEvent {
		editor: editor.to_string(),
		timestamp: timestamp_now(),
		event: event.as_str().to_string(),
		hash: None,
	});
	Ok(())
}

/// Create a new audit trail for a freshly created card.
pub fn new_audit_trail(editor: &str) -> AuditTrail {
	AuditTrail {
		version: "1.0.0".to_string(),
		hash: None,
		history: vec![AuditEvent {
			editor: editor.to_string(),
			timestamp: timestamp_now(),
			event: AuditEventKind::Created.as_str().to_string(),
			hash: None,
		}],
	}
}

fn timestamp_now() -> String {
	Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn bump_version(version: &str, bump: AuditBump) -> Result<String, String> {
	let mut parts = version.split('.');
	let major = parse_part(parts.next(), "major")?;
	let minor = parse_part(parts.next(), "minor")?;
	let patch = parse_part(parts.next(), "patch")?;
	if parts.next().is_some() {
		return Err(format!(
			"Invalid audit version '{version}': expected semver"
		));
	}
	let (major, minor, patch) = match bump {
		AuditBump::Patch => (major, minor, patch + 1),
		AuditBump::Minor => (major, minor + 1, 0),
		AuditBump::Major => (major + 1, 0, 0),
	};
	Ok(format!("{major}.{minor}.{patch}"))
}

fn parse_part(value: Option<&str>, label: &str) -> Result<u64, String> {
	let value = value.ok_or_else(|| format!("Invalid audit version: missing {label} component"))?;
	value
		.parse::<u64>()
		.map_err(|_| format!("Invalid audit version: {label} component '{value}' is not a number"))
}
