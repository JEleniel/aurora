//! Read-only inspector state for the editor sidebar.

use aurora_shared::{Card, CardRef, Link};
use serde_json::Value;

use crate::EditorSession;

/// View model for the selected card inspector.
#[derive(Clone, Debug)]
pub(crate) struct CardInspectorModel {
	pub card: Card,
	pub inbound_links: Vec<CardRef>,
	pub outbound_links: Vec<OutboundLinkSummary>,
	pub registry_warnings: Vec<String>,
}

impl PartialEq for CardInspectorModel {
	fn eq(&self, other: &Self) -> bool {
		card_equals(&self.card, &other.card)
			&& self.inbound_links == other.inbound_links
			&& self.outbound_links == other.outbound_links
			&& self.registry_warnings == other.registry_warnings
	}
}

/// Outbound link information enriched with the best available label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OutboundLinkSummary {
	pub relationship: String,
	pub target_id: String,
	pub target_name: Option<String>,
}

/// Current right-sidebar inspector state.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CardInspectorState {
	Unavailable(String),
	EmptySelection,
	Missing(String),
	Error(String),
	Loaded(Box<CardInspectorModel>),
}

/// Load the current inspector state using the live editor session.
pub(crate) fn load_card_inspector_state(
	session: Option<&EditorSession>,
	selected_card_id: &str,
) -> CardInspectorState {
	if selected_card_id.trim().is_empty() {
		return CardInspectorState::EmptySelection;
	}

	let Some(session) = session else {
		return CardInspectorState::Unavailable("Editor session is not available yet.".to_string());
	};

	let card = match session.load_card(selected_card_id) {
		Ok(Some(card)) => card,
		Ok(None) => {
			return CardInspectorState::Missing(format!(
				"Card {selected_card_id} is not available in the current model home.",
			));
		}
		Err(error) => return CardInspectorState::Error(error.to_string()),
	};

	let inbound_links = match session.cards_linking_to(selected_card_id) {
		Ok(cards) => cards,
		Err(error) => return CardInspectorState::Error(error.to_string()),
	};
	let outbound_links = summarize_outbound_links(session, &card.links);
	let registry_warnings = card.check_registry(session.card_registry());

	CardInspectorState::Loaded(Box::new(CardInspectorModel {
		card,
		inbound_links,
		outbound_links,
		registry_warnings,
	}))
}

/// Format an attribute value for readable display in the inspector.
pub(crate) fn format_attribute_value(value: &Value) -> String {
	match value {
		Value::String(text) => text.clone(),
		_ => serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()),
	}
}

fn summarize_outbound_links(session: &EditorSession, links: &[Link]) -> Vec<OutboundLinkSummary> {
	links
		.iter()
		.map(|link| OutboundLinkSummary {
			relationship: link.relationship.clone(),
			target_id: link.target.clone(),
			target_name: session
				.load_card(link.target.as_str())
				.ok()
				.flatten()
				.map(|card| card.name),
		})
		.collect()
}

fn card_equals(left: &Card, right: &Card) -> bool {
	left.schema == right.schema
		&& left.id == right.id
		&& left.card_type == right.card_type
		&& left.card_subtype == right.card_subtype
		&& left.name == right.name
		&& left.description == right.description
		&& left.version == right.version
		&& left.status == right.status
		&& left.boundary == right.boundary
		&& left.notes == right.notes
		&& left.icon == right.icon
		&& left.attributes == right.attributes
		&& left.external_references == right.external_references
		&& links_equal(&left.links, &right.links)
		&& left.source_path == right.source_path
		&& left.validation_errors == right.validation_errors
		&& left.validation_warnings == right.validation_warnings
}

fn links_equal(left: &[Link], right: &[Link]) -> bool {
	left.len() == right.len()
		&& left.iter().zip(right).all(|(left, right)| {
			left.target == right.target && left.relationship == right.relationship
		})
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use serde_json::json;

	use super::{CardInspectorState, format_attribute_value, load_card_inspector_state};
	use crate::EditorSession;

	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn empty_selection_short_circuits_before_session_work() {
		assert!(matches!(
			load_card_inspector_state(None, "  "),
			CardInspectorState::EmptySelection
		));
	}

	#[test]
	fn unavailable_state_preserves_message() {
		let state = load_card_inspector_state(None, "ACT-001");
		let CardInspectorState::Unavailable(message) = state else {
			panic!("expected unavailable state");
		};
		assert!(message.contains("session"));
	}

	#[test]
	fn error_state_can_be_pattern_matched() {
		let state = CardInspectorState::Error("broken index".to_string());
		let CardInspectorState::Error(message) = state else {
			panic!("expected error state");
		};
		assert_eq!(message, "broken index");
	}

	#[test]
	fn object_attributes_are_pretty_printed() {
		let rendered = format_attribute_value(&json!({
			"priority": "high",
			"score": 7
		}));

		assert!(rendered.contains("\"priority\": \"high\""));
		assert!(rendered.contains('\n'));
	}

	#[test]
	fn inspector_loads_selected_card_and_inbound_links() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = seed_model_home(temp.path())?;
		write_card(
			&model_home.join("MIS-001-Root.json"),
			json!({
				"$schema": "schemas/Aurora.card.schema.json",
				"id": "MIS-001",
				"card_type": "Mission",
				"name": "Mission",
				"description": "root",
				"links": [{"target": "ACT-001", "relationship": "contains"}]
			}),
		)?;
		write_card(
			&model_home.join("MIS-001").join("ACT-001-Alpha.json"),
			json!({
				"$schema": "../schemas/Aurora.card.schema.json",
				"id": "ACT-001",
				"card_type": "Activity",
				"name": "Alpha Workflow",
				"description": "loaded by inspector",
				"status": "draft",
				"attributes": {"owner": "ops"},
				"links": [{"target": "ACT-002", "relationship": "supports"}]
			}),
		)?;
		write_card(
			&model_home.join("MIS-001").join("ACT-002-Beta.json"),
			json!({
				"$schema": "../schemas/Aurora.card.schema.json",
				"id": "ACT-002",
				"card_type": "Activity",
				"name": "Beta Workflow",
				"description": "linked target",
				"links": []
			}),
		)?;

		let session = EditorSession::open(temp.path())?;
		let state = load_card_inspector_state(Some(&session), "ACT-001");

		let CardInspectorState::Loaded(model) = state else {
			panic!("expected loaded inspector state");
		};
		assert_eq!(model.card.name, "Alpha Workflow");
		assert_eq!(model.card.status.as_deref(), Some("draft"));
		assert_eq!(model.inbound_links.len(), 1);
		assert_eq!(model.inbound_links[0].id, "MIS-001");
		assert_eq!(model.outbound_links.len(), 1);
		assert!(model.registry_warnings.is_empty());
		assert_eq!(model.outbound_links[0].target_id, "ACT-002");
		assert_eq!(
			model.outbound_links[0].target_name.as_deref(),
			Some("Beta Workflow")
		);
		Ok(())
	}

	#[test]
	fn inspector_reports_missing_cards() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = seed_model_home(temp.path())?;
		write_card(
			&model_home.join("MIS-001-Root.json"),
			json!({
				"$schema": "schemas/Aurora.card.schema.json",
				"id": "MIS-001",
				"card_type": "Mission",
				"name": "Mission",
				"description": "root",
				"links": []
			}),
		)?;

		let session = EditorSession::open(temp.path())?;
		let state = load_card_inspector_state(Some(&session), "ACT-404");
		let CardInspectorState::Missing(message) = state else {
			panic!("expected missing state");
		};
		assert!(message.contains("ACT-404"));
		Ok(())
	}

	fn seed_model_home(root: &Path) -> Result<std::path::PathBuf> {
		let model_home = root.join("aurora");
		std::fs::create_dir_all(model_home.join("schemas"))?;
		std::fs::create_dir_all(model_home.join("reference"))?;
		std::fs::create_dir_all(model_home.join("MIS-001"))?;
		std::fs::write(model_home.join("MIS-001").join("AuditLog.ndjson"), b"")?;
		std::fs::write(
			model_home.join("schemas").join("Aurora.card.schema.json"),
			serde_json::to_string_pretty(&json!({
				"$schema": "http://json-schema.org/draft-07/schema#",
				"type": "object",
				"required": ["$schema", "id", "card_type", "name", "description", "links"],
				"properties": {
					"$schema": { "type": "string" },
					"id": { "type": "string" },
					"card_type": { "type": "string" },
					"card_subtype": { "type": "string" },
					"name": { "type": "string" },
					"description": { "type": "string" },
					"status": { "type": "string" },
					"attributes": { "type": "object" },
					"links": {
						"type": "array",
						"items": {
							"type": "object",
							"required": ["target", "relationship"],
							"properties": {
								"target": { "type": "string" },
								"relationship": { "type": "string" }
							}
						}
					}
				}
			}))?,
		)?;
		std::fs::write(
			model_home
				.join("reference")
				.join("Aurora.modelconfiguration.json"),
			serde_json::to_string_pretty(&json!({
				"cards": [
					{
						"acronym": "MIS",
						"card_type": "Mission",
						"description": "Mission",
						"relationships": [{"target": "ACT", "relationship": "contains"}]
					},
					{
						"acronym": "ACT",
						"card_type": "Activity",
						"description": "Activity",
						"relationships": [
							{"target": "ACT", "relationship": "supports"},
							{"target": "ACT", "relationship": "contains"}
						]
					}
				]
			}))?,
		)?;
		std::fs::write(
			model_home
				.join("reference")
				.join("Aurora.viewconfiguration.json"),
			serde_json::to_string_pretty(&json!({
				"available_icons": [],
				"cards": [
					{
						"acronym": "MIS",
						"shape": "rectangle",
						"fill": "#ffffff",
						"stroke": "#0f172a",
						"text": "#0f172a"
					},
					{
						"acronym": "ACT",
						"shape": "rectangle",
						"fill": "#ffffff",
						"stroke": "#0f172a",
						"text": "#0f172a"
					}
				],
				"views": []
			}))?,
		)?;
		Ok(model_home)
	}

	fn write_card(path: &Path, value: serde_json::Value) -> Result<()> {
		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)?;
		}
		std::fs::write(path, serde_json::to_string_pretty(&value)?)?;
		Ok(())
	}
}
