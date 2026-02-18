use std::path::PathBuf;

use crate::{Attributes, AuditLog, Card, Link, Model};

pub(super) fn make_card(id: &str, card_type: &str, targets: &[&str]) -> Card {
	let links = targets
		.iter()
		.map(|target| Link {
			target: target.to_string(),
			relationship: "rel".to_string(),
		})
		.collect();

	Card {
		schema: None,
		id: id.to_string(),
		card_type: card_type.to_string(),
		card_subtype: None,
		name: format!("{} name", id),
		description: "desc".to_string(),
		version: Some("1.0.0".to_string()),
		status: None,
		boundary: None,
		notes: None,
		icon: None,
		attributes: Attributes::new(),
		links,
		source_path: PathBuf::from(format!("{}.json", id)),
		validation_errors: Vec::new(),
		validation_warnings: Vec::new(),
	}
}

pub(super) fn make_model(root_card: Card, cards: Vec<Card>) -> Model {
	Model {
		root_card,
		cards,
		audit_log: AuditLog {
			schema: None,
			history: Vec::new(),
			source_path: PathBuf::from("AuditLog.ndjson"),
			validation_errors: Vec::new(),
		},
		model_home: PathBuf::from("model"),
		mission_home: PathBuf::from("mission"),
	}
}
