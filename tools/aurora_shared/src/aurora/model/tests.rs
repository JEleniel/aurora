use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

use crate::ModelHome;
use crate::model::{AuditTrail, AuroraModel, Card};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn audit_trail() -> AuditTrail {
	AuditTrail {
		version: "0.0.1".to_string(),
		hash: None,
		history: Vec::new(),
	}
}

fn card(id: &str, card_type: &str, source_path: Option<PathBuf>) -> Card {
	Card {
		schema: None,
		id: id.to_string(),
		card_type: card_type.to_string(),
		card_subtype: None,
		name: id.to_string(),
		description: "test".to_string(),
		status: None,
		links: Vec::new(),
		audit_trail: audit_trail(),
		attributes: Value::Null,
		source_path,
		extra: BTreeMap::new(),
	}
}

fn missing(message: &str) -> Box<dyn std::error::Error> {
	Box::new(std::io::Error::new(std::io::ErrorKind::Other, message))
}

fn mission_id(model: &AuroraModel) -> Option<String> {
	model
		.iter_cards()
		.find(|card| card.card_type == "Mission")
		.map(|card| card.id.clone())
}

#[test]
fn split_by_mission_partitions_cards() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let home = ModelHome::new(temp.path())?;

	let mission_one_path = temp.path().join("MIS-001-Alpha.jsjson");
	let mission_two_path = temp.path().join("MIS-002-Bravo.jsjson");
	let req_one_path = temp
		.path()
		.join("MIS-001")
		.join("Requirement")
		.join("REQ-001.jsjson");
	let req_two_path = temp
		.path()
		.join("MIS-002")
		.join("Requirement")
		.join("REQ-002.jsjson");
	let shared_path = temp.path().join("Shared").join("CTL-001.jsjson");

	let cards = vec![
		card("MIS-001", "Mission", Some(mission_one_path)),
		card("MIS-002", "Mission", Some(mission_two_path)),
		card("REQ-001", "Requirement", Some(req_one_path)),
		card("REQ-002", "Requirement", Some(req_two_path)),
		card("CTL-001", "Control", Some(shared_path)),
	];
	let model = AuroraModel::new(home, cards);
	let models = model.split_by_mission();
	assert_eq!(models.len(), 2);

	let mut by_mission = BTreeMap::new();
	for model in models {
		if let Some(id) = mission_id(&model) {
			by_mission.insert(id, model);
		}
	}

	assert!(by_mission.contains_key("MIS-001"));
	assert!(by_mission.contains_key("MIS-002"));

	let model_one = by_mission
		.get("MIS-001")
		.ok_or_else(|| missing("missing MIS-001"))?;
	assert!(model_one.get("MIS-001").is_some());
	assert!(model_one.get("REQ-001").is_some());
	assert!(model_one.get("REQ-002").is_none());
	assert!(model_one.get("CTL-001").is_some());

	let model_two = by_mission
		.get("MIS-002")
		.ok_or_else(|| missing("missing MIS-002"))?;
	assert!(model_two.get("MIS-002").is_some());
	assert!(model_two.get("REQ-002").is_some());
	assert!(model_two.get("REQ-001").is_none());
	assert!(model_two.get("CTL-001").is_some());

	Ok(())
}
