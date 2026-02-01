//! Compact model export for aurora_cli.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use aurora_shared::{Card, Model};
use serde_json::{Map, Value};

/// Write a compact representation of the model.
pub fn write_compact(model: &Model, output_path: &Path, schema_path: &Path) -> Result<()> {
	let schema_ref = schema_reference(output_path, schema_path)
		.unwrap_or_else(|| schema_path.to_string_lossy().to_string());
	let mut root = Map::new();
	root.insert("$schema".to_string(), Value::String(schema_ref));
	let mut cards = Vec::new();
	for card in model.cards.values() {
		cards.push(compact_card(card));
	}
	root.insert("cards".to_string(), Value::Array(cards));
	let data = Value::Object(root);
	let output = serde_json::to_string(&data)?;
	fs::write(output_path, output)
		.with_context(|| format!("failed to write compact model to {}", output_path.display()))?;
	Ok(())
}

fn schema_reference(output_path: &Path, schema_path: &Path) -> Option<String> {
	let output_dir = output_path.parent().unwrap_or(output_path);
	relative_path(output_dir, schema_path).map(|path| path.to_string_lossy().to_string())
}

fn relative_path(from: &Path, to: &Path) -> Option<std::path::PathBuf> {
	let from_components: Vec<std::path::Component<'_>> = from.components().collect();
	let to_components: Vec<std::path::Component<'_>> = to.components().collect();
	let mut shared = 0usize;
	while shared < from_components.len()
		&& shared < to_components.len()
		&& from_components[shared] == to_components[shared]
	{
		shared += 1;
	}
	if shared == 0 {
		return None;
	}
	let mut relative = std::path::PathBuf::new();
	for _ in shared..from_components.len() {
		relative.push("..");
	}
	for component in &to_components[shared..] {
		relative.push(component.as_os_str());
	}
	Some(relative)
}

fn compact_card(card: &Card) -> Value {
	let mut map = Map::new();
	map.insert("id".to_string(), Value::String(card.id.clone()));
	map.insert(
		"card_type".to_string(),
		Value::String(card.card_type.clone()),
	);
	if let Some(subtype) = &card.card_subtype {
		map.insert("card_subtype".to_string(), Value::String(subtype.clone()));
	}
	map.insert("name".to_string(), Value::String(card.name.clone()));
	map.insert(
		"description".to_string(),
		Value::String(card.description.clone()),
	);
	if let Some(status) = &card.status {
		map.insert("status".to_string(), Value::String(status.clone()));
	}
	if !card.attributes.is_empty() {
		let mut attributes = Map::new();
		for attribute in &card.attributes {
			attributes.insert(attribute.name.clone(), attribute.value.clone());
		}
		map.insert("attributes".to_string(), Value::Object(attributes));
	}
	if !card.links.is_empty() {
		let links = card
			.links
			.iter()
			.map(|link| {
				let mut link_map = Map::new();
				link_map.insert(
					"relationship".to_string(),
					Value::String(link.relationship.clone()),
				);
				link_map.insert("target".to_string(), Value::String(link.target.clone()));
				Value::Object(link_map)
			})
			.collect();
		map.insert("links".to_string(), Value::Array(links));
	}
	Value::Object(map)
}
