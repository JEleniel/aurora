//! Bottom-panel state loading for audit-log and diagnostics tabs.

use aurora_shared::{AuditLog, AuditLogEntry, Aurora};

use crate::EditorSession;

/// Bottom-panel tabs available in the editor shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BottomPanelTab {
	AuditLog,
	Diagnostics,
}

impl BottomPanelTab {
	pub(crate) fn label(self) -> &'static str {
		match self {
			Self::AuditLog => "Audit log",
			Self::Diagnostics => "Diagnostics",
		}
	}
}

/// Fully loaded bottom-panel data for the selected card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BottomPanelModel {
	pub root_card_id: String,
	pub audit_entries: Vec<AuditEntrySummary>,
	pub validation_errors: Vec<DiagnosticMessage>,
	pub validation_warnings: Vec<DiagnosticMessage>,
}

/// Summary of an audit-log entry shown in the bottom panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AuditEntrySummary {
	pub timestamp_label: String,
	pub editor: String,
	pub change_summary: String,
	pub navigation_target: String,
}

/// Navigation-capable diagnostic message shown in the bottom panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiagnosticMessage {
	pub message: String,
	pub navigation_target: String,
}

/// Current loading state of the bottom panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BottomPanelState {
	Unavailable(String),
	EmptySelection,
	Missing(String),
	Error(String),
	Loaded(Box<BottomPanelModel>),
}

/// Load audit-log and diagnostics data for the selected card.
pub(crate) fn load_bottom_panel_state(
	session: Option<&EditorSession>,
	selected_card_id: &str,
) -> BottomPanelState {
	if selected_card_id.trim().is_empty() {
		return BottomPanelState::EmptySelection;
	}

	let Some(session) = session else {
		return BottomPanelState::Unavailable("Editor session is not available yet.".to_string());
	};

	let root_card_id = match resolve_root_card_id(session, selected_card_id) {
		Ok(Some(root_card_id)) => root_card_id,
		Ok(None) => {
			return BottomPanelState::Missing(format!(
				"Card {selected_card_id} is not available in the current model home.",
			));
		}
		Err(error) => return BottomPanelState::Error(error.to_string()),
	};

	let aurora = match Aurora::try_load(session.model_home()) {
		Ok(aurora) => aurora,
		Err(error) => return BottomPanelState::Error(error.to_string()),
	};

	let Some(model) = aurora
		.models
		.iter()
		.find(|model| model.root_card.id == root_card_id)
	else {
		return BottomPanelState::Error(format!(
			"Model {root_card_id} is not available in the current model home.",
		));
	};

	BottomPanelState::Loaded(Box::new(BottomPanelModel {
		root_card_id: root_card_id.clone(),
		audit_entries: summarize_audit_entries(&model.audit_log, selected_card_id),
		validation_errors: build_diagnostic_messages(root_card_id.as_str(), model.validate()),
		validation_warnings: build_diagnostic_messages(
			root_card_id.as_str(),
			model.get_schema_validation_warnings(),
		),
	}))
}

fn resolve_root_card_id(
	session: &EditorSession,
	selected_card_id: &str,
) -> Result<Option<String>, crate::EditorSessionError> {
	let Some(card) = session.load_card(selected_card_id)? else {
		return Ok(None);
	};

	for root in session.roots() {
		if card.source_path == root.source_path {
			return Ok(Some(root.id.clone()));
		}

		let mission_home = session.model_home().join(root.id.as_str());
		if card.source_path.starts_with(mission_home) {
			return Ok(Some(root.id.clone()));
		}
	}

	Ok(session
		.breadcrumb(selected_card_id)?
		.first()
		.map(|card| card.id.clone()))
}

fn summarize_audit_entries(audit_log: &AuditLog, target: &str) -> Vec<AuditEntrySummary> {
	let mut entries: Vec<AuditLogEntry> = audit_log.entries_for_target(target).cloned().collect();
	entries.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
	entries
		.into_iter()
		.map(|entry| AuditEntrySummary {
			timestamp_label: entry.timestamp.to_rfc3339(),
			editor: entry.editor.clone(),
			change_summary: AuditLog::change_summary_for_target(&entry, target),
			navigation_target: target.to_string(),
		})
		.collect()
}

fn build_diagnostic_messages(root_card_id: &str, messages: Vec<String>) -> Vec<DiagnosticMessage> {
	messages
		.into_iter()
		.map(|message| DiagnosticMessage {
			navigation_target: extract_card_id(&message)
				.unwrap_or_else(|| root_card_id.to_string()),
			message,
		})
		.collect()
}

fn extract_card_id(message: &str) -> Option<String> {
	message
		.split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
		.find(|token| looks_like_card_id(token))
		.map(ToString::to_string)
}

fn looks_like_card_id(token: &str) -> bool {
	let Some((prefix, suffix)) = token.split_once('-') else {
		return false;
	};
	prefix.len() == 3
		&& prefix
			.chars()
			.all(|character| character.is_ascii_uppercase())
		&& !suffix.is_empty()
		&& suffix.chars().all(|character| character.is_ascii_digit())
}

#[cfg(test)]
mod tests {
	use aurora_shared::AuditLogEntry;
	use serde_json::json;

	use super::{
		BottomPanelState, BottomPanelTab, build_diagnostic_messages, extract_card_id,
		load_bottom_panel_state, summarize_audit_entries,
	};

	#[test]
	fn bottom_panel_tabs_have_stable_labels() {
		assert_eq!(BottomPanelTab::AuditLog.label(), "Audit log");
		assert_eq!(BottomPanelTab::Diagnostics.label(), "Diagnostics");
	}

	#[test]
	fn empty_selection_short_circuits_state_loading() {
		assert_eq!(
			load_bottom_panel_state(None, "   "),
			BottomPanelState::EmptySelection
		);
	}

	#[test]
	fn unavailable_state_preserves_message() {
		let state = load_bottom_panel_state(None, "ACT-001");
		let BottomPanelState::Unavailable(message) = state else {
			panic!("expected unavailable state");
		};
		assert!(message.contains("session"));
	}

	#[test]
	fn extracts_card_id_from_common_validation_messages() {
		assert_eq!(
			extract_card_id("ACT-001: missing required property"),
			Some("ACT-001".to_string())
		);
		assert_eq!(
			extract_card_id("Broken link from ACT-001 to REQ-404"),
			Some("ACT-001".to_string())
		);
		assert_eq!(extract_card_id("AuditLog: line 3 is invalid"), None);
	}

	#[test]
	fn diagnostic_messages_fall_back_to_root_for_model_level_entries() {
		let messages = build_diagnostic_messages(
			"MIS-001",
			vec!["AuditLog: failed to parse entry".to_string()],
		);

		assert_eq!(messages.len(), 1);
		assert_eq!(messages[0].navigation_target, "MIS-001");
	}

	#[test]
	fn audit_entries_are_sorted_newest_first() {
		let audit_log = aurora_shared::AuditLog {
			schema: None,
			history: vec![
				serde_json::from_value::<AuditLogEntry>(json!({
					"timestamp": "2026-03-12T09:30:00Z",
					"editor": "Older",
					"target": "ACT-001",
					"change_type": "change",
					"changes": [{
						"card_id": "ACT-001",
						"change_type": "change",
						"link_changes": []
					}]
				}))
				.expect("valid older audit entry"),
				serde_json::from_value::<AuditLogEntry>(json!({
					"timestamp": "2026-03-13T08:45:00Z",
					"editor": "Newer",
					"target": "ACT-001",
					"change_type": "change",
					"changes": [{
						"card_id": "ACT-001",
						"change_type": "change",
						"link_changes": []
					}]
				}))
				.expect("valid newer audit entry"),
			],
			source_path: std::path::PathBuf::new(),
			validation_errors: Vec::new(),
		};

		let entries = summarize_audit_entries(&audit_log, "ACT-001");
		assert_eq!(entries.len(), 2);
		assert_eq!(entries[0].editor, "Newer");
		assert_eq!(entries[1].editor, "Older");
	}
}
