/// ZIP import/export functionality for AURORA architectures
use crate::models::{ArchitectureModel, Card, Link};
use serde_json::json;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipArchive;

/// Export an architecture model to a ZIP file
pub fn export_to_zip(
	model: &ArchitectureModel,
	output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
	let file = File::create(output_path)?;
	let mut zip_writer = zip::ZipWriter::new(file);

	// Create folders and add cards by type
	for card in model.cards.values() {
		let folder = card.r#type.folder_name();
		let filename = format!("cards/{}/{}-{}.json", folder, folder, normalize_name(&card.id));

		let card_json = serde_json::to_string_pretty(&card)?;

		// Use empty tuple () as the extension type
		let options: zip::write::FileOptions<()> = zip::write::FileOptions::default();
		zip_writer.start_file(filename, options)?;
		zip_writer.write_all(card_json.as_bytes())?;
	}

	// Add links file
	if !model.links.is_empty() {
		let links_json = serde_json::to_string_pretty(&model.links)?;
		let options: zip::write::FileOptions<()> = zip::write::FileOptions::default();
		zip_writer.start_file("links/links.json", options)?;
		zip_writer.write_all(links_json.as_bytes())?;
	}

	// Add metadata
	let metadata_json = serde_json::to_string_pretty(&model.metadata)?;
	let options: zip::write::FileOptions<()> = zip::write::FileOptions::default();
	zip_writer.start_file("metadata.json", options)?;
	zip_writer.write_all(metadata_json.as_bytes())?;

	// Add manifest
	let stats = model.statistics();
	let manifest = json!({
		"version": "1.0.0",
		"format": "aurora-archive-v1",
		"created": chrono::Utc::now().to_rfc3339(),
		"created_by": "AURORA Tooling v1.0.0",
		"model": {
			"name": model.metadata.name,
			"description": model.metadata.description,
			"root_driver_id": model.metadata.root_driver_id,
			"statistics": {
				"total_cards": stats.total_cards,
				"total_links": stats.total_links,
				"cards_by_type": stats.cards_by_type,
			}
		}
	});

	let manifest_json = serde_json::to_string_pretty(&manifest)?;
	let options: zip::write::FileOptions<()> = zip::write::FileOptions::default();
	zip_writer.start_file("MANIFEST.json", options)?;
	zip_writer.write_all(manifest_json.as_bytes())?;

	zip_writer.finish()?;
	Ok(())
}

/// Import an architecture model from a ZIP file
pub fn import_from_zip(
	input_path: &Path,
) -> Result<ArchitectureModel, Box<dyn std::error::Error>> {
	let file = File::open(input_path)?;
	let mut archive = ZipArchive::new(file)?;

	let mut model = ArchitectureModel::new();

	// Load metadata if present
	if let Ok(mut metadata_file) = archive.by_name("metadata.json") {
		let mut contents = String::new();
		metadata_file.read_to_string(&mut contents)?;
		model.metadata = serde_json::from_str(&contents)?;
	}

	// Load all cards
	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;
		if file.is_file() && file.name().starts_with("cards/") && file.name().ends_with(".json") {
			let mut contents = String::new();
			file.read_to_string(&mut contents)?;
			if let Ok(card) = serde_json::from_str::<Card>(&contents) {
				model.add_card(card);
			}
		}
	}

	// Load links
	if let Ok(mut links_file) = archive.by_name("links/links.json") {
		let mut contents = String::new();
		links_file.read_to_string(&mut contents)?;
		if let Ok(links) = serde_json::from_str::<Vec<Link>>(&contents) {
			for link in links {
				model.add_link(link);
			}
		}
	}

	Ok(model)
}

/// Normalize a name for use in filenames
fn normalize_name(s: &str) -> String {
	s.to_lowercase()
		.chars()
		.map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
		.collect::<String>()
		.split('-')
		.filter(|s| !s.is_empty())
		.collect::<Vec<_>>()
		.join("-")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_normalize_name() {
		assert_eq!(normalize_name("Hello World"), "hello-world");
		assert_eq!(normalize_name("API_Gateway"), "api-gateway");
	}
}
