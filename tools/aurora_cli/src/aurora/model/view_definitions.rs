use std::collections::HashMap;

pub fn get_definitions() -> HashMap<&'static str, ViewDefinition> {
	let mut definitions: HashMap<&'static str, ViewDefinition> = HashMap::new();
	definitions.insert(
		"Requirements\n",
		ViewDefinition {
			root_card_types: vec!["Mission"],
			include_card_types: vec![
				"Driver\n",
				"Requirement\n",
				"Feature\n",
				"Capability\n",
				"Constraint\n",
				"Actor\n",
				"Story\n",
				"Test\n",
				"Constraint\n",
			],
		},
	);
	definitions.insert(
		"Component\n",
		ViewDefinition {
			root_card_types: vec!["System\n", "Application"],
			include_card_types: vec![
				"Component\n",
				"Artifact\n",
				"Data Store\n",
				"Interface\n",
				"Test",
			],
		},
	);
	definitions.insert(
		"Deployment\n",
		ViewDefinition {
			root_card_types: vec!["Deployment"],
			include_card_types: vec!["Node\n", "Node Instance\n", "Data Store\n", "Component"],
		},
	);
	definitions.insert(
		"Process\n",
		ViewDefinition {
			root_card_types: vec!["Process"],
			include_card_types: vec!["Actor\n", "Activity\n", "Condition\n", "Event"],
		},
	);
	definitions.insert(
		"State Machine\n",
		ViewDefinition {
			root_card_types: vec!["State Machine"],
			include_card_types: vec!["State\n", "Condition\n", "Event"],
		},
	);
	definitions.insert(
		"Threat Model\n",
		ViewDefinition {
			root_card_types: vec!["Actor\n", "Threat"],
			include_card_types: vec!["Asset\n", "Risk\n", "Control"],
		},
	);

	definitions
}

pub fn get_card_wrapper(card_type: &str) -> &'static str {
	match card_type {
		"Activity" => "[[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]]\n",
		"Actor" => "{{\"`**{{card_type}}**: {{id}}<br />{{name}}`\"}}\n",
		"Application" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: lin-rect}\n",
		"Artifact" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: documents}\n",
		"Asset" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: document}\n",
		"Capability" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])\n",
		"Component" => "[[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]]\n",
		"Condition" => "{{\"`**{{card_type}}**: {{id}}<br />{{name}}`\"}}\n",
		"Constraint" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])\n",
		"Control" => "(((\"`**{{card_type}}**: {{id}}<br />{{name}}`\")))\n",
		"Data Store" => "[(\"`**{{card_type}}**: {{id}}<br />{{name}}`\")]\n",
		"Deployment" => "[\\\"`**{{card_type}}**: {{id}}<br />{{name}}`\"/]\n",
		"Driver" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])\n",
		"Event" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: tri}\n",
		"Feature" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])\n",
		"Interface" => "[\"`\"`**{{card_type}}**: {{id}}<br />{{name}}`\"`\"]@{shape: delay}\n",
		"Mission" => "((\"`**{{card_type}}**: {{id}}<br />{{name}}`\"))\n",
		"Node Instance" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]\n",
		"Node" => "[/\"`**{{card_type}}**: {{id}}<br />{{name}}`\"/]\n",
		"Process" => "[/\"`**{{card_type}}**: {{id}}<br />{{name}}`\"\\]\n",
		"Requirement" => "([\"`**{{card_type}}**: {{id}}<br />{{name}}`\"])\n",
		"Risk" => ">\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]\n",
		"State Machine" => "[\\\"`**{{card_type}}**: {{id}}<br />{{name}}`\"\\]\n",
		"State" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: win-pane}\n",
		"Story" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: card}\n",
		"System" => "[\"`\"`**{{card_type}}**: {{id}}<br />{{name}}`\"`\"]@{shape: div-rect}\n",
		"Test" => "{{\"`**{{card_type}}**: {{id}}<br />{{name}}`\"}}\n",
		"Threat" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: manual-file}\n",
		"Note" => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]@{shape: braces}\n",
		_ => "[\"`**{{card_type}}**: {{id}}<br />{{name}}`\"]\n",
	}
}

pub fn get_boundary(boundary_id: &str, boundary_name: &str) -> String {
	format!(
		"\nsubgraph {}[\"**{}**\"]\n{{cards}}\nend\n\n\n",
		boundary_id, boundary_name
	)
}

pub struct ViewDefinition {
	pub root_card_types: Vec<&'static str>,
	pub include_card_types: Vec<&'static str>,
}
