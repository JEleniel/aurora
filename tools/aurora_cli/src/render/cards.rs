use crate::error::AuroraCliError;
use crate::model::{Card, Model};
use crate::output::{OutputPaths, relative_markdown_link};
use crate::render::views::{STANDARD_VIEWS, view_is_applicable};
use pathdiff::diff_paths;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

const CARD_TYPE_ORDER: &[&str] = &[
	"Mission",
	"Driver",
	"Constraint",
	"Requirement",
	"Capability",
	"Feature",
	"System",
	"Application",
	"Component",
	"Interface",
	"Artifact",
	"Asset",
	"Data Store",
	"Deployment",
	"Node",
	"Node Instance",
	"Process",
	"Story",
	"Actor",
	"Event",
	"Activity",
	"Condition",
	"Control",
	"State Machine",
	"State",
	"Boundary",
	"Note",
	"Test",
	"Threat",
	"Risk",
];

/// Summary of rendered card artifacts.
#[derive(Debug, Default)]
pub struct CardRenderSummary {
	pub cards_written: usize,
	pub index_written: bool,
}

pub fn render_cards(
	model: &Model,
	outputs: &OutputPaths,
) -> Result<CardRenderSummary, AuroraCliError> {
	let mut written = 0;
	for card in model.cards() {
		let target = card_markdown_path(card, outputs.cards_dir());
		let contents = render_card_markdown(model, card, &target, outputs)?;
		outputs.write(&target, &contents)?;
		written += 1;
	}
	let index_path = outputs.index_readme();
	let index_contents = render_index_markdown(model, outputs, index_path)?;
	outputs.write(index_path, &index_contents)?;

	// Also write a mission-specific README file named `README-{mission_id}.md`
	let mission = model.mission();
	let mission_readme = outputs.root().join(format!("README-{}.md", mission.id));
	let mission_contents = render_index_markdown(model, outputs, &mission_readme)?;
	outputs.write(&mission_readme, &mission_contents)?;
	Ok(CardRenderSummary {
		cards_written: written,
		index_written: true,
	})
}

fn render_card_markdown(
	model: &Model,
	card: &Card,
	current_card_path: &Path,
	outputs: &OutputPaths,
) -> Result<String, AuroraCliError> {
	let mut out = String::new();
	out.push_str(&format!("# {}\n\n", card.name));
	out.push_str(&format!("- Type: {}\n", card.card_type));
	if let Some(subtype) = &card.card_subtype {
		out.push_str(&format!("- Subtype: {}\n", subtype));
	}
	if let Some(status) = &card.status {
		out.push_str(&format!("- Status: {}\n", status));
	}
	out.push_str("\n## Description\n\n");
	out.push_str(card.description.trim());
	out.push_str("\n\n");
	out.push_str("## Links\n\n");
	if card.links.is_empty() {
		out.push_str("_None_\n\n");
	} else {
		out.push_str("| Relationship | Target |\n");
		out.push_str("| --- | --- |\n");
		for link in &card.links {
			let relation = link.relationship.as_deref().unwrap_or("links to");
			if let Some(target) = model.card(&link.target) {
				let target_path = card_markdown_path(target, outputs.cards_dir());
				let rel_link = relative_markdown_link(current_card_path, &target_path)?;
				out.push_str(&format!(
					"| {} | [{}]({}) ({}) |\n",
					relation, target.name, rel_link, target.id
				));
			} else {
				out.push_str(&format!("| {} | {} |\n", relation, link.target));
			}
		}
		out.push('\n');
	}
	out.push_str("---\n\n");
	out.push_str(&format!("ID: {}\n", card.id));
	Ok(out)
}

fn render_index_markdown(
	model: &Model,
	outputs: &OutputPaths,
	current_doc: &Path,
) -> Result<String, AuroraCliError> {
	let mission = model.mission();
	let cards_rel = diff_paths(outputs.cards_dir(), outputs.root())
		.unwrap_or_else(|| outputs.cards_dir().to_path_buf());
	let views_rel = diff_paths(outputs.views_dir(), outputs.root())
		.unwrap_or_else(|| outputs.views_dir().to_path_buf());
	let model_root = path_to_posix(model.root());
	let mut out = String::new();
	out.push_str("# Design Documentation\n\n");
	out.push_str(&format!(
		"This folder contains the generated Aurora documentation for the `{}` mission.\n\n",
		mission.name
	));

	// Model root and mission entrypoint as links (must start with '/')
	let model_link = relative_markdown_link(current_doc, model.root())?;
	out.push_str("## Model\n\n");
	out.push_str(&format!(
		"- Aurora model root: [{}]({})\n",
		model_root, model_link
	));
	let mission_rel = path_to_posix(&mission.relative_path);
	let mission_link = relative_markdown_link(current_doc, &mission.source_path)?;
	out.push_str(&format!(
		"- Mission entrypoint: [{}]({})\n\n",
		mission_rel, mission_link
	));

	// Views first
	out.push_str("## Views\n\n");
	out.push_str(&format!(
		"Standard views are rendered under `{}`. Links to generated views are below.\n\n",
		path_to_posix(&views_rel)
	));
	let mut any_views = false;
	for spec in STANDARD_VIEWS.iter() {
		if view_is_applicable(model, spec) {
			let vf = outputs.views_dir().join(spec.file_name);
			let rel = relative_markdown_link(current_doc, &vf)?;
			out.push_str(&format!("- [{}]({})\n", spec.name, rel));
			any_views = true;
		}
	}
	if !any_views {
		out.push_str("_None_\n");
	}
	out.push_str("\n");

	out.push_str("## Cards\n\n");
	out.push_str(&format!(
		"Cards are rendered under `{}` and grouped below by card type. Link text is the card name.\n\n",
		path_to_posix(&cards_rel)
	));

	let mut grouped = group_cards_by_type(model);
	for &card_type in CARD_TYPE_ORDER {
		if let Some(cards) = grouped.remove(card_type) {
			write_type_section(card_type, &cards, outputs, current_doc, &mut out)?;
		}
	}
	let leftovers: Vec<_> = grouped.into_iter().collect();
	for (card_type, cards) in leftovers {
		write_type_section(&card_type, &cards, outputs, current_doc, &mut out)?;
	}
	if out.ends_with('\n') {
		out.pop();
		if out.ends_with('\r') {
			out.pop();
		}
	}
	Ok(out)
}

fn write_type_section(
	type_name: &str,
	cards: &[&Card],
	outputs: &OutputPaths,
	current_doc: &Path,
	buf: &mut String,
) -> Result<(), AuroraCliError> {
	buf.push_str(&format!("### {}\n\n", type_name));
	for card in cards {
		let target = card_markdown_path(card, outputs.cards_dir());
		let rel = relative_markdown_link(current_doc, &target)?;
		buf.push_str(&format!("- [{}]({})\n", card.name, rel));
	}
	buf.push('\n');
	Ok(())
}

fn group_cards_by_type(model: &Model) -> BTreeMap<String, Vec<&Card>> {
	let mut map: BTreeMap<String, Vec<&Card>> = BTreeMap::new();
	for card in model.cards() {
		map.entry(card.card_type.clone()).or_default().push(card);
	}
	for cards in map.values_mut() {
		cards.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));
	}
	map
}

fn card_markdown_path(card: &Card, cards_root: &Path) -> PathBuf {
	if card.is_mission() {
		cards_root.join(format!("{}.md", card.id))
	} else {
		cards_root
			.join(&card.card_type)
			.join(format!("{}.md", card.id))
	}
}

fn path_to_posix(path: &Path) -> String {
	path.components()
		.map(|component| match component {
			Component::Normal(seg) => seg.to_string_lossy().to_string(),
			Component::CurDir => String::from("."),
			Component::ParentDir => String::from(".."),
			Component::RootDir => String::from("/"),
			Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().to_string(),
		})
		.collect::<Vec<_>>()
		.join("/")
}
