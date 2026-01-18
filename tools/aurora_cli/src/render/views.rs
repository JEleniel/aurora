use crate::error::AuroraCliError;
use crate::model::{Card, Model};
use crate::output::OutputPaths;
use std::collections::{BTreeMap, HashMap};

/// Summary describing how many view files were generated or skipped.
#[derive(Debug, Default)]
pub struct ViewRenderSummary {
	pub rendered: usize,
	pub skipped: usize,
}

pub fn render_views(
	model: &Model,
	outputs: &OutputPaths,
) -> Result<ViewRenderSummary, AuroraCliError> {
	let mut summary = ViewRenderSummary::default();
	for spec in STANDARD_VIEWS.iter() {
		let nodes = collect_view_nodes(model, spec);
		if nodes.is_empty() || !spec.has_anchor(&nodes) {
			summary.skipped += 1;
			continue;
		}
		let contents = build_view(spec, &nodes);
		let target = outputs.views_dir().join(spec.file_name);
		outputs.write(&target, &contents)?;
		summary.rendered += 1;
	}
	Ok(summary)
}

fn build_view(spec: &ViewSpec, nodes: &[&Card]) -> String {
	let mut out = String::new();
	out.push_str(&format!("# {}\n\n", spec.name));
	out.push_str("**Cards included:**\n\n");
	for ty in spec.card_types {
		out.push_str(&format!("- {}\n", ty));
	}
	out.push('\n');
	out.push_str("```mermaid\n");
	out.push_str("%%{init: {'flowchart': {'defaultRenderer': 'elk'}}}%%\n");
	out.push_str("graph LR\n");

	let mut node_map: HashMap<&str, String> = HashMap::new();
	let mut class_tokens: BTreeMap<String, String> = BTreeMap::new();
	let mut class_nodes: HashMap<String, Vec<String>> = HashMap::new();
	let ordered = sort_nodes(spec, nodes);
	for card in ordered {
		let node_name = card.id.clone();
		out.push_str(&format!("  {}\n", node_markup(card)));
		node_map.insert(card.id.as_str(), node_name.clone());
		let token = class_token(&card.card_type);
		class_tokens
			.entry(card.card_type.clone())
			.or_insert(token.clone());
		class_nodes.entry(token).or_default().push(node_name);
	}
	out.push('\n');

	for card in nodes {
		if let Some(from) = node_map.get(card.id.as_str()) {
			for link in &card.links {
				if let Some(to) = node_map.get(link.target.as_str()) {
					let relation = link.relationship.as_deref().unwrap_or("");
					if relation.is_empty() {
						out.push_str(&format!("  {} --> {}\n", from, to));
					} else {
						out.push_str(&format!("  {} -- {} --> {}\n", from, relation, to));
					}
				}
			}
		}
	}
	out.push('\n');

	for (card_type, token) in class_tokens.iter() {
		let style = class_style(card_type);
		out.push_str(&format!("  classDef {} {}\n", token, style));
	}
	out.push('\n');

	for token in class_tokens.values() {
		if let Some(nodes) = class_nodes.get(token) {
			let members = nodes.join(",");
			out.push_str(&format!("  class {} {}\n", members, token));
		}
	}
	out.push('\n');
	out.push_str("```\n");
	out
}

fn sort_nodes<'a>(spec: &ViewSpec, nodes: &[&'a Card]) -> Vec<&'a Card> {
	let mut ordered = nodes.to_vec();
	ordered.sort_by(|lhs, rhs| {
		let l_idx = spec.type_index(&lhs.card_type);
		let r_idx = spec.type_index(&rhs.card_type);
		l_idx.cmp(&r_idx).then_with(|| lhs.id.cmp(&rhs.id))
	});
	ordered
}

pub(crate) fn view_is_applicable(model: &Model, spec: &ViewSpec) -> bool {
	let nodes = collect_view_nodes(model, spec);
	!nodes.is_empty() && spec.has_anchor(&nodes)
}

pub(crate) fn collect_view_nodes<'a>(model: &'a Model, spec: &ViewSpec) -> Vec<&'a Card> {
	model
		.cards()
		.filter(|card| spec.card_types.contains(&card.card_type.as_str()))
		.collect()
}

fn node_markup(card: &Card) -> String {
	let label = format!("`{}<br />{}`", card.card_type, card.name);
	match card.card_type.as_str() {
		"Mission" => framed_node(card, "((", "))", &label),
		"Driver" => framed_node(card, "([", "])", &label),
		"Requirement" => framed_node(card, "[/", "/]", &label),
		"Capability" => framed_node(card, "[[", "]]", &label),
		"Feature" => framed_node(card, "(", ")", &label),
		"System" | "Application" | "Boundary" => framed_node(card, "[", "]", &label),
		"Component" | "Activity" | "State" => framed_node(card, "(", ")", &label),
		"Interface" | "Control" => framed_node(card, ">", "]", &label),
		"Artifact" => custom_shape_node(card, "documents", &label),
		"Data Store" => framed_node(card, "[(", ")]", &label),
		"Asset" => custom_shape_node(card, "document", &label),
		"Deployment" => framed_node(card, "[\\", "\\]", &label),
		"Node" | "Threat" => framed_node(card, "{", "}", &label),
		"Node Instance" => framed_node(card, "[/", "/]", &label),
		"Process" | "Actor" => framed_node(card, "([", "])", &label),
		"Event" => framed_node(card, "((", "))", &label),
		"State Machine" | "Test" => framed_node(card, "[[", "]]", &label),
		"Condition" => framed_node(card, "{{", "}}", &label),
		"Constraint" => framed_node(card, "[\\", "\\]", &label),
		"Risk" => framed_node(card, "[/", "/]", &label),
		"Note" => custom_shape_node(card, "comment", &label),
		_ => framed_node(card, "[", "]", &label),
	}
}

fn framed_node(card: &Card, start: &str, end: &str, label: &str) -> String {
	format!("{}{}\"{}\"{}", card.id, start, label, end)
}

fn custom_shape_node(card: &Card, shape: &str, label: &str) -> String {
	format!("{}@{{shape: {}, label: \"{}\"}}", card.id, shape, label)
}

fn class_token(card_type: &str) -> String {
	let mut sanitized: String = card_type
		.chars()
		.map(|c| {
			if c.is_ascii_alphanumeric() {
				c.to_ascii_lowercase()
			} else {
				'_'
			}
		})
		.collect();
	if sanitized.is_empty() {
		sanitized.push_str("card");
	}
	if sanitized
		.chars()
		.next()
		.map(|c| c.is_ascii_digit())
		.unwrap_or(false)
	{
		sanitized.insert(0, '_');
	}
	format!("cls_{}", sanitized)
}

fn class_style(card_type: &str) -> &'static str {
	CLASS_STYLE_MAP
		.iter()
		.find(|(ty, _)| *ty == card_type)
		.map(|(_, style)| *style)
		.unwrap_or(DEFAULT_CLASS_STYLE)
}

pub(crate) struct ViewSpec {
	pub(crate) name: &'static str,
	pub(crate) file_name: &'static str,
	card_types: &'static [&'static str],
}

impl ViewSpec {
	const fn new(
		name: &'static str,
		file_name: &'static str,
		card_types: &'static [&'static str],
	) -> Self {
		Self {
			name,
			file_name,
			card_types,
		}
	}

	fn type_index(&self, ty: &str) -> usize {
		self.card_types
			.iter()
			.position(|candidate| *candidate == ty)
			.unwrap_or(usize::MAX)
	}

	fn has_anchor(&self, nodes: &[&Card]) -> bool {
		match self.card_types.first() {
			Some(anchor) => nodes.iter().any(|card| card.card_type == *anchor),
			None => true,
		}
	}
}

pub(crate) const STANDARD_VIEWS: [ViewSpec; 5] = [
	ViewSpec::new(
		"Requirements View",
		"requirements-view.md",
		&["Mission", "Driver", "Requirement", "Capability", "Feature"],
	),
	ViewSpec::new(
		"System Composition",
		"system-composition.md",
		&[
			"System",
			"Application",
			"Component",
			"Interface",
			"Data Store",
			"Artifact",
		],
	),
	ViewSpec::new(
		"Deployment Topology",
		"deployment-topology.md",
		&[
			"Deployment",
			"Node",
			"Node Instance",
			"Application",
			"Component",
			"Data Store",
		],
	),
	ViewSpec::new(
		"Process Flow",
		"process-flow.md",
		&[
			"Process",
			"Actor",
			"Event",
			"Activity",
			"Condition",
			"Control",
		],
	),
	ViewSpec::new(
		"State Machine",
		"state-machine.md",
		&["State Machine", "State", "Event", "Condition"],
	),
];

const DEFAULT_CLASS_STYLE: &str = "fill:#1f2937,color:#FFFFFF";

const CLASS_STYLE_MAP: &[(&str, &str)] = &[
	("Mission", "fill:#022c22,color:#FFFFFF"),
	("Driver", "fill:#064e3b,color:#FFFFFF"),
	("Requirement", "fill:#065f46,color:#FFFFFF"),
	("Capability", "fill:#052e16,color:#FFFFFF"),
	("Feature", "fill:#14532d,color:#FFFFFF"),
	("System", "fill:#172554,color:#FFFFFF"),
	("Application", "fill:#1e3a8a,color:#FFFFFF"),
	("Component", "fill:#1e40af,color:#FFFFFF"),
	("Interface", "fill:#082f49,color:#FFFFFF"),
	("Artifact", "fill:#1e293b,color:#FFFFFF"),
	("Asset", "fill:#334155,color:#FFFFFF"),
	("Data Store", "fill:#075985,color:#FFFFFF"),
	("Deployment", "fill:#1e1b4b,color:#FFFFFF"),
	("Node", "fill:#312e81,color:#FFFFFF"),
	("Node Instance", "fill:#3730a3,color:#FFFFFF"),
	("Process", "fill:#2e1065,color:#FFFFFF"),
	("Activity", "fill:#4c1d95,color:#FFFFFF"),
	("Event", "fill:#5b21b6,color:#FFFFFF"),
	("Condition", "fill:#422006,color:#FFFFFF"),
	("Control", "fill:#713f12,color:#FFFFFF"),
	("Constraint", "fill:#854d0e,color:#FFFFFF"),
	("State Machine", "fill:#4a044e,color:#FFFFFF"),
	("State", "fill:#701a75,color:#FFFFFF"),
	("Actor", "fill:#1a2e05,color:#FFFFFF"),
	("Story", "fill:#365314,color:#FFFFFF"),
	("Risk", "fill:#881337,color:#FFFFFF"),
	("Threat", "fill:#4c0519,color:#FFFFFF"),
	("Test", "fill:#022c22,color:#FFFFFF"),
	("Boundary", "stroke-dasharray:5 5,stroke-width:4"),
	("Note", "fill:#1f2937,color:#FFFFFF"),
];
