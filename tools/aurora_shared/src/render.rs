mod dot;
mod geometry;
mod icons;
pub mod render_error;
mod svg;

use crate::{
	Aurora, Card, Model,
	registry::{CardDefinition, ViewDefinition},
	render::{dot::Diagram, render_error::RenderError},
};
use std::{
	collections::HashSet,
	fs,
	io::{ErrorKind, Write},
	path::PathBuf,
	process::{Command, Stdio},
};
pub use svg::{SvgDocument, SvgError, SvgRenderOptions, SvgRenderer};
use tracing::{debug, info};

/// Represents a set of cards to render, possibly nested within boundaries.
#[derive(Debug)]
enum CardSet {
	Cards(HashSet<String>),
	Boundary(String, String, Box<CardSet>),
}

/// Render the canonical view set (DOT → SVG) defined by the registry.
pub fn render(aurora: &Aurora, path: &PathBuf) -> Result<(), RenderError> {
	for model in &*aurora.models {
		let output_path = path.clone().join(format!("{}-views", model.root_card.id));

		info!(
			"Rendering {} to {}",
			model.root_card.id,
			output_path.display()
		);
		render_views(&model, &output_path)?;
	}

	Ok(())
}

fn render_views(model: &Model, path: &PathBuf) -> Result<(), RenderError> {
	let mut source_path = path.clone();
	source_path.push("source");

	match fs::remove_dir_all(&path) {
		Ok(_) => debug!("Removed existing output path at {}", path.display()),
		Err(e) => {
			if e.kind() == ErrorKind::NotFound {
				debug!("No existing output path at {}", path.display());
			} else {
				return Err(RenderError::IoError(e));
			}
		}
	}

	fs::create_dir_all(&path)?;
	fs::create_dir_all(&source_path)?;
	info!("Output path created at {}", path.display());

	for view in &ViewDefinition::get_all() {
		let view_filename = view.name.replace(" ", "_");
		let view_path = path.join(format!("{}.view.svg", view_filename));
		let view_source_path = source_path.join(format!("{}.view.dot", view_filename));
		let rendered_source_path = source_path.join(format!("{}.view.gen", view_filename));

		debug!(
			"Rendering model {} view {} to {}",
			model.root_card.id,
			view.name,
			view_path.display()
		);

		let mut root_ids: HashSet<String> = HashSet::new();
		if view.root_card_types.contains(&"Mission".to_string()) {
			root_ids.insert(model.root_card.id.clone());
		}
		root_ids.extend(collect_view_root_nodes(model, view));
		if root_ids.is_empty() {
			info!(
				"Skipping model {} view {} because it is empty.",
				model.root_card.id, view.name
			);
			continue;
		}

		let mut nodes: CardSet = CardSet::Cards(root_ids.clone());

		for root_id in root_ids {
			nodes = traverse(nodes, view, model, &root_id)?;
			// Skip root only views
			match &nodes {
				CardSet::Cards(cards) => {
					if cards.len() == 0 {
						info!(
							"Skipping model {} view {} because it contains only root nodes.",
							model.root_card.id, view.name
						);
						continue;
					}
				}
				CardSet::Boundary(..) => {}
			}

			let mut dot: String = String::from(
				r##"digraph {
					fontname="Noto Sans";
					fontcolor="#FFFFFF";
					fontsize=12;
				"##,
			);
			dot.push_str(&render_dot_nodes(model, &nodes)?);
			dot.push_str(&render_dot_links(view, model, &nodes)?);
			dot.push_str("}");
			fs::write(&view_source_path, &dot)?;
			let json_dot = dot_to_json(&dot)?;
			fs::write(
				&rendered_source_path,
				serde_json::to_string_pretty(&json_dot)?,
			)?;
			let svg_output = SvgRenderer::render_svg(&json_dot)?;
			fs::write(&view_path, svg_output)?;
		}
	}

	Ok(())
}

fn traverse(
	nodes: CardSet,
	view_def: &ViewDefinition,
	model: &Model,
	root_id: &str,
) -> Result<CardSet, RenderError> {
	let mut seen: Vec<String> = Vec::new();

	let mut pending_cards: Vec<String> = vec![root_id.to_string()];
	let mut parent_sets: Vec<CardSet> = Vec::new();
	let mut current_set: CardSet = nodes;

	let included_cards: Vec<&Card> = model
		.cards
		.iter()
		.filter(|c| view_def.included_card_types.contains(&c.card_type))
		.collect();

	while let Some(current_id) = pending_cards.pop() {
		// Local loop completed
		if seen.contains(&current_id) {
			continue;
		}
		seen.push(current_id.clone());

		let card = if current_id == model.root_card.id {
			&model.root_card
		} else {
			included_cards
				.iter()
				.find(|c| c.id == current_id)
				.ok_or(RenderError::CardNotFound(current_id.clone()))?
		};
		if &card.card_type == "Boundary" {
			if let Some(subtype) = &card.card_subtype {
				if subtype == "End" {
					match current_set {
						CardSet::Cards(_) => {
							return Err(RenderError::NoOpenBoundary(
								card.id.clone(),
								card.name.clone(),
							));
						}
						CardSet::Boundary(_, name, _) => {
							if *name != card.name {
								return Err(RenderError::UnmatchedBoundaryClosure(
									card.id.clone(),
									card.name.clone(),
									name.clone(),
								));
							}
							if let Some(parent) = parent_sets.pop() {
								current_set = parent;
							} else {
								return Err(RenderError::InvalidParent(card.id.clone()));
							}
						}
					}
				}
			} else {
				parent_sets.push(current_set);
				let boundary_set = CardSet::Boundary(
					card.id.clone(),
					card.name.clone(),
					Box::new(CardSet::Cards(HashSet::new())),
				);
				current_set = boundary_set;
				continue;
			}
		}

		match current_set {
			CardSet::Cards(ref mut vec) => {
				vec.insert(current_id.clone());
			}
			CardSet::Boundary(_, _, ref mut boxed_set) => match **boxed_set {
				CardSet::Cards(ref mut vec) => {
					vec.insert(current_id.clone());
				}
				_ => (),
			},
		}

		pending_cards.extend(
			card.links
				.iter()
				.filter(|c| {
					view_def.included_card_types.contains(
						&CardDefinition::get_by_acronym(&c.target[0..3])
							.card_type
							.to_string(),
					)
				})
				.map(|l| l.target.clone()),
		);
	}

	let nodes = current_set;
	Ok(nodes)
}

fn collect_view_root_nodes(model: &Model, view_def: &ViewDefinition) -> Vec<String> {
	let mut root_ids: Vec<String> = Vec::new();
	model
		.cards
		.iter()
		.filter(|c| view_def.root_card_types.contains(&c.card_type))
		.for_each(|c| root_ids.push(c.id.clone()));

	root_ids
}

fn render_dot_nodes(model: &Model, nodes: &CardSet) -> Result<String, RenderError> {
	let mut dot: String = String::new();

	match nodes {
		CardSet::Cards(cards) => {
			for id in cards {
				let card = if id == &model.root_card.id {
					&model.root_card
				} else {
					&model
						.cards
						.iter()
						.find(|c| &c.id == id)
						.ok_or(RenderError::CardNotFound(id.clone()))?
				};

				let md_url = format!(
					"../{}-views/{}.view.svg",
					model.root_card.id,
					card.name.replace(" ", "_")
				);
				dot.push_str(render_node(&card, &md_url).as_str());
			}
		}
		CardSet::Boundary(id, name, cards) => {
			if let Some(subtype) = &model
				.cards
				.iter()
				.find(|c| &c.id == id)
				.ok_or(RenderError::CardNotFound(id.clone()))?
				.card_subtype
			{
				if subtype == "End" {
					return Ok(dot);
				}
			}
			dot.push_str(
				format!(
					r#"subgraph cluster_{} {{
						label=<{}<B>{}</B>>
						clusterrank=local;
						style=dashed;
					"#,
					id.replace("-", "_"),
					if let Some(subtype) = &model
						.cards
						.iter()
						.find(|c| &c.id == id)
						.ok_or(RenderError::CardNotFound(id.clone()))?
						.card_subtype
					{
						format!("<I>{}</I><BR/>", subtype)
					} else {
						"".to_string()
					},
					name
				)
				.as_str(),
			);
			dot.push_str(render_dot_nodes(model, cards)?.as_str());

			dot.push_str("}\n")
		}
	}

	Ok(dot)
}

fn render_dot_links(
	view_def: &ViewDefinition,
	model: &Model,
	nodes: &CardSet,
) -> Result<String, RenderError> {
	let mut dot: String = String::new();

	match nodes {
		CardSet::Cards(cards) => {
			for id in cards {
				if id.starts_with("BND") {
					continue;
				}

				let card = if id == &model.root_card.id {
					&model.root_card
				} else {
					&model
						.cards
						.iter()
						.find(|c| &c.id == id)
						.ok_or(RenderError::CardNotFound(id.clone()))?
				};
				if !view_def.included_card_types.contains(&card.card_type)
					&& !view_def.root_card_types.contains(&card.card_type)
				{
					continue;
				}
				for link in &card.links {
					if link.target.starts_with("NOT") {
						dot.push_str(
							format!(
								r#"	{} -> {} [style="dotted"];
								"#,
								card.id.replace("-", "_"),
								link.target.replace("-", "_"),
							)
							.as_str(),
						);
					} else if link.target.starts_with("BND") {
						continue;
					} else {
						dot.push_str(
							format!(
								r#"	{} -> {} [label="{}"];
								"#,
								card.id.replace("-", "_"),
								link.target.replace("-", "_"),
								link.relationship,
							)
							.as_str(),
						);
					}
				}
			}
		}
		CardSet::Boundary(_, _, cards) => {
			dot.push_str(&render_dot_links(view_def, model, cards)?);
		}
	}

	Ok(dot)
}

fn dot_to_json(dot_src: &str) -> Result<Diagram, RenderError> {
	let mut child = Command::new("dot")
		.args(["-Tjson"]) // or "-Txdot_json"
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	{
		let stdin = child
			.stdin
			.as_mut()
			.ok_or("failed to open stdin")
			.map_err(|e| RenderError::TerminalFailed(e.to_string()));
		stdin?.write_all(dot_src.as_bytes())?;
	}

	let output = child.wait_with_output()?;

	if !output.status.success() {
		let err = String::from_utf8_lossy(&output.stderr);
		return Err(RenderError::TerminalFailed(err.to_string()));
	}
	Ok(serde_json::from_str(&String::from_utf8(output.stdout)?)?)
}

fn render_node(card: &Card, md_url: &str) -> String {
	format!(
		r#"{}[
			label=<
				<FONT COLOR="{}"><B>{}</B>{}<BR/>
				<B>{}</B><BR/>
				<BR/>
				{}</FONT>
			>
			height=1.0;
			width=1.6;
			href="{}";
			style=filled;
			fillcolor="{}";
			svg_shape="{}";
			icon="{}";
		];
		"#,
		card.id.replace("-", "_"),
		CardDefinition::get_color(&card.card_type),
		card.card_type,
		if let Some(subtype) = &card.card_subtype {
			format!("<BR/><I>{}</I>", subtype)
		} else {
			"".to_string()
		},
		card.name,
		wrap_text(&card.description, 80).join(""),
		md_url,
		CardDefinition::get_fill(&card.card_type),
		CardDefinition::get_shape(&card.card_type),
		CardDefinition::get_icon(&card.card_type),
	)
}

fn escape_text(value: &str) -> String {
	value
		.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
		.replace('\'', "&apos;")
}

fn wrap_text(s: &str, width: usize) -> Vec<String> {
	let mut results: Vec<String> = s
		.chars()
		.collect::<Vec<_>>()
		.chunks(width)
		.map(|c| c.iter().collect())
		.collect();
	for result in results.iter_mut() {
		*result = format!("{}<BR/>", escape_text(result));
	}
	results
}
