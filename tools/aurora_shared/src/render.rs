mod layout;
pub mod svg;

pub mod render_error;

pub use layout::*;

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use crate::Aurora;
use crate::registry::ViewDefinition;
use render_error::RenderError;
use tracing::{info, warn};

/// Render all views for the provided Aurora models.
pub fn render(aurora: &Aurora, output_dir: &Path) -> Result<(), RenderError> {
	let view_definitions = match aurora.view_registry.try_get_all() {
		Ok(defs) => defs,
		Err(err) => {
			warn!(
				"Failed to load view definitions; skipping view rendering ({})",
				err
			);
			return Ok(());
		}
	};
	if view_definitions.is_empty() {
		warn!("No view definitions available; skipping view rendering.");
		return Ok(());
	}

	for model in &aurora.models {
		let views_dir = output_dir.join(model.root_card.id.as_str()).join("Views");
		std::fs::create_dir_all(&views_dir)?;

		let card_acronyms = model_card_acronyms(model);
		for view in &view_definitions {
			let has_root_type = view
				.root_card_types
				.iter()
				.any(|t| card_acronyms.contains(t.as_str()));
			if !has_root_type {
				info!(
					"Skipping view '{}' for {}: no roots of required types present",
					view.name, model.root_card.id
				);
				continue;
			}

			let included_types = union_view_card_types(view);
			let layout = match layout_model(model, &view.root_card_types, included_types.as_slice())
			{
				Ok(layout) => layout,
				Err(err) => {
					warn!(
						"Skipping view '{}' for {}: layout failed ({})",
						view.name, model.root_card.id, err
					);
					continue;
				}
			};

			if layout.nodes.len() <= 1 {
				info!(
					"Skipping view '{}' for {}: normalized graph has one node",
					view.name, model.root_card.id
				);
				continue;
			}
			if layout.edges.is_empty() {
				info!(
					"Skipping view '{}' for {}: normalized graph has no outgoing edges",
					view.name, model.root_card.id
				);
				continue;
			}

			let view_slug = sanitize_filename(view.name.as_str());
			let view_slug = if view_slug.is_empty() {
				"View".to_string()
			} else {
				view_slug
			};
			let output_path = views_dir.join(format!("{}.svg", view_slug));

			match svg::Svg::write_to_file(
				&output_path,
				model,
				&layout,
				&aurora.card_registry,
				&aurora.svg_template,
				None,
			) {
				Ok(()) => {
					info!(
						"Rendered view '{}' for {} to {}",
						view.name,
						model.root_card.id,
						output_path.display()
					);
				}
				Err(err) => {
					warn!(
						"Skipping view '{}' for {}: SVG render failed ({})",
						view.name, model.root_card.id, err
					);
				}
			}
		}
	}

	Ok(())
}

fn model_card_acronyms(model: &crate::Model) -> HashSet<String> {
	let mut out: HashSet<&str> = HashSet::new();
	for card in std::iter::once(&model.root_card).chain(model.cards.iter()) {
		if let Some(prefix) = card.id.split('-').next()
			&& !prefix.is_empty()
		{
			out.insert(prefix);
		}
	}
	out.into_iter().map(ToString::to_string).collect()
}

fn union_view_card_types(view: &ViewDefinition) -> Vec<String> {
	let mut all: BTreeSet<String> = BTreeSet::new();
	for t in &view.root_card_types {
		all.insert(t.clone());
	}
	for t in &view.included_card_types {
		all.insert(t.clone());
	}
	all.into_iter().collect()
}

fn sanitize_filename(name: &str) -> String {
	let mut out = String::new();
	let mut last_was_underscore = false;
	for ch in name.chars() {
		if ch.is_ascii_alphanumeric() {
			out.push(ch);
			last_was_underscore = false;
			continue;
		}
		if ch.is_whitespace() && !last_was_underscore {
			out.push('_');
			last_was_underscore = true;
		}
	}
	while out.contains("__") {
		out = out.replace("__", "_");
	}
	out.trim_matches('_').to_string()
}
