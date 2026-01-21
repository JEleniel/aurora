use std::collections::HashMap;

pub fn get_definitions() -> HashMap<&'static str, ViewDefinition> {
	let mut definitions: HashMap<&'static str, ViewDefinition> = HashMap::new();
	definitions.insert(
		"Requirements",
		ViewDefinition {
			root_card_types: vec!["Mission"],
			include_card_types: vec![
				"Driver",
				"Requirement",
				"Feature",
				"Capability",
				"Constraint",
				"Actor",
				"Story",
				"Test",
				"Constraint",
			],
		},
	);
	definitions.insert(
		"Component",
		ViewDefinition {
			root_card_types: vec!["System", "Application"],
			include_card_types: vec!["Component", "Artifact", "Data Store", "Interface", "Test"],
		},
	);
	definitions.insert(
		"Deployment",
		ViewDefinition {
			root_card_types: vec!["Deployment"],
			include_card_types: vec!["Node", "Node Instance", "Data Store", "Component"],
		},
	);
	definitions.insert(
		"Process",
		ViewDefinition {
			root_card_types: vec!["Process"],
			include_card_types: vec!["Actor", "Activity", "Condition", "Event"],
		},
	);
	definitions.insert(
		"State Machine",
		ViewDefinition {
			root_card_types: vec!["State Machine"],
			include_card_types: vec!["State", "Condition", "Event"],
		},
	);
	definitions.insert(
		"Threat Model",
		ViewDefinition {
			root_card_types: vec!["Actor", "Threat"],
			include_card_types: vec!["Asset", "Risk", "Control"],
		},
	);

	definitions
}

pub fn get_card_wrapper(card_type: &str) -> &'static str {
	match card_type {
		"Activity" => "[[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]]",
		"Actor" => "{{\"`**{{card_type}}**: {{id}}<br />{{name}}`\"}}",
		"Application" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: lin-rect}",
		"Artifact" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: documents}",
		"Asset" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: document}",
		"Capability" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])",
		"Component" => "[[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]]",
		"Condition" => "{{\"`**{{card_type}}**: {{id}}<br />{{name}}`\"}}",
		"Constraint" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])",
		"Control" => "(((\"`**{{card_type}}**: {{id}}<br />{{name}}`\")))",
		"Data Store" => "[(\"`**{{card_type}}**: {{id}}<br />{{name}}`\")]",
		"Deployment" => "[\\\"`**{{card_type}}**: {{id}}<br />{{name}}`\"/]",
		"Driver" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])",
		"Event" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: tri}",
		"Feature" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])",
		"Interface" => "[\"`\"`**{{card_type}}**: {{id}}<br />{{name}}`\"`\"]@{shape: delay}",
		"Mission" => "((\"`**{{card_type}}**: {{id}}<br />{{name}}`\"))",
		"Node Instance" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]",
		"Node" => "[/\"`**{{card_type}}**: {{id}}<br />{{name}}`\"/]",
		"Process" => "[/\"`**{{card_type}}**: {{id}}<br />{{name}}`\"\\]",
		"Requirement" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])",
		"Risk" => ">\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]",
		"State Machine" => "[\\\"`**{{card_type}}**: {{id}}<br />{{name}}`\"\\]",
		"State" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: win-pane}",
		"Story" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: card}",
		"System" => "[\"`\"`**{{card_type}}**: {{id}}<br />{{name}}`\"`\"]@{shape: div-rect}",
		"Test" => "{{\"`**{{card_type}}**: {{id}}<br />{{name}}`\"}}",
		"Threat" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: manual-file}",
		"Note" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: braces}",
		_ => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]",
	}
}

pub fn get_boundary(boundary_id: &str, boundary_name: &str) -> String {
	format!(
		"subgraph {}[\"**{}**\"]{{cards}}end",
		boundary_id, boundary_name
	)
}

pub struct ViewDefinition {
	pub root_card_types: Vec<&'static str>,
	pub include_card_types: Vec<&'static str>,
}
