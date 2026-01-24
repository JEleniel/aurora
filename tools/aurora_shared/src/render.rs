use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::errors::{AuroraError, Result};
use crate::model::AuroraModel;

/// Summary information produced by rendering helpers.
#[derive(Debug, Clone)]
pub struct RenderSummary {
	pub cards_written: usize,
	pub views_written: usize,
	pub output_dir: PathBuf,
}

/// Render per-card Markdown documentation.
pub fn render_markdown(model: &AuroraModel, output_dir: impl AsRef<Path>) -> Result<RenderSummary> {
	let output_dir = output_dir.as_ref();
	fs::create_dir_all(output_dir).map_err(|err| AuroraError::io(output_dir, err))?;

	let mut cards_written = 0;
	for card in model.iter_cards() {
		let file_name = format!("{}-{}.md", card.id, card.name.replace(' ', "_"));
		let path = output_dir.join(file_name);
		let mut buffer = String::new();
		buffer.push_str(&format!("# {} ({})\n\n", card.id, card.card_type));
		buffer.push_str(&format!("**Name:** {}\\\n\n", card.name));
		buffer.push_str("## Description\n\n");
		buffer.push_str(&card.description);
		buffer.push_str("\n\n## Links\n\n");
		if card.links.is_empty() {
			buffer.push_str("_No outgoing links._\n");
		} else {
			for link in &card.links {
				buffer.push_str(&format!("- `{}` → `{}`\n", link.relationship, link.target));
			}
		}

		write_text(&path, buffer)?;
		cards_written += 1;
	}

	Ok(RenderSummary {
		cards_written,
		views_written: 0,
		output_dir: output_dir.to_path_buf(),
	})
}

/// Render a lightweight relationship view helpful for quick inspection.
pub fn render_views(model: &AuroraModel, output_dir: impl AsRef<Path>) -> Result<RenderSummary> {
	let output_dir = output_dir.as_ref();
	fs::create_dir_all(output_dir).map_err(|err| AuroraError::io(output_dir, err))?;
	let views_path = output_dir.join("relationships.md");

	let mut buffer = String::from(
		"# Relationships\n\n| Source | Relationship | Target |\n| --- | --- | --- |\n",
	);
	for card in model.iter_cards() {
		for link in &card.links {
			buffer.push_str(&format!(
				"| {} | {} | {} |\n",
				card.id, link.relationship, link.target
			));
		}
	}

	write_text(&views_path, buffer)?;

	Ok(RenderSummary {
		cards_written: 0,
		views_written: 1,
		output_dir: output_dir.to_path_buf(),
	})
}

/// Convenience helper that renders both cards and the relationship view.
pub fn render_all(model: &AuroraModel, output_dir: impl AsRef<Path>) -> Result<RenderSummary> {
	let output_dir = output_dir.as_ref();
	let cards = render_markdown(model, output_dir)?;
	let views = render_views(model, output_dir)?;
	Ok(RenderSummary {
		cards_written: cards.cards_written,
		views_written: views.views_written,
		output_dir: output_dir.to_path_buf(),
	})
}

/// Write a compact single-file representation of the model for agent consumption.
pub fn write_compact_model(model: &AuroraModel, output_path: Option<PathBuf>) -> Result<PathBuf> {
	let target_path = if let Some(path) = output_path {
		path
	} else {
		let mission_id = model
			.iter_cards()
			.find(|card| card.card_type == "Mission")
			.map(|card| card.id.clone())
			.unwrap_or_else(|| "MIS-COMPACT".to_string());
		model
			.home()
			.root()
			.join(format!("AGENT-{}.jsjson", mission_id))
	};

	let mut compact_cards = Vec::new();
	for card in model.iter_cards() {
		let mut value = serde_json::to_value(card).map_err(|err| AuroraError::InvalidInput {
			message: err.to_string(),
		})?;
		if let serde_json::Value::Object(ref mut map) = value {
			map.remove("$schema");
			map.remove("audit_trail");
			map.remove("source_path");
		}
		compact_cards.push(value);
	}

	let payload = json!({
		"cards": compact_cards,
		"generated_at": SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map(|duration| duration.as_secs())
			.unwrap_or_default(),
	});

	write_json_pretty(&target_path, &payload)?;
	Ok(target_path)
}

fn write_text(path: &Path, contents: String) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).map_err(|err| AuroraError::io(parent, err))?;
	}
	let tmp_path = tmp_path(path);
	{
		let mut file = File::create(&tmp_path).map_err(|err| AuroraError::io(&tmp_path, err))?;
		file.write_all(contents.as_bytes())
			.map_err(|err| AuroraError::io(&tmp_path, err))?;
		file.flush()
			.map_err(|err| AuroraError::io(&tmp_path, err))?;
	}
	fs::rename(&tmp_path, path).map_err(|err| AuroraError::io(path, err))?;
	Ok(())
}

fn write_json_pretty(path: &Path, value: &serde_json::Value) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).map_err(|err| AuroraError::io(parent, err))?;
	}
	let tmp_path = tmp_path(path);
	{
		let file = File::create(&tmp_path).map_err(|err| AuroraError::io(&tmp_path, err))?;
		serde_json::to_writer_pretty(&file, value).map_err(|err| AuroraError::InvalidInput {
			message: err.to_string(),
		})?;
	}
	fs::rename(&tmp_path, path).map_err(|err| AuroraError::io(path, err))?;
	Ok(())
}

fn tmp_path(path: &Path) -> PathBuf {
	let mut tmp_name = path
		.file_name()
		.and_then(|name| name.to_str())
		.map(|name| format!(".{name}.tmp"))
		.unwrap_or_else(|| ".aurora.tmp".to_string());
	if tmp_name == path.file_name().and_then(|n| n.to_str()).unwrap_or("") {
		tmp_name.push_str(".tmp");
	}
	path.with_file_name(tmp_name)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::discovery::ModelHome;
	use crate::model::{AuditTrail, AuroraModel, Card};
	use serde_json::Value;
	use std::collections::BTreeMap;
	use tempfile::TempDir;

	#[test]
	fn render_markdown_creates_card_files() {
		let (tmp, model) = sample_model();
		let output = tmp.path().join("render");
		let summary = render_markdown(&model, &output).expect("render should succeed");
		assert_eq!(summary.cards_written, 1);
		assert!(output.join("MIS-001-Provide_Default_Tooling.md").exists());
	}

	#[test]
	fn tmp_path_is_hidden() {
		let path = PathBuf::from("/tmp/output.md");
		let tmp = tmp_path(&path);
		let name = tmp.file_name().and_then(|name| name.to_str()).unwrap();
		assert!(name.starts_with(".output.md"));
		assert!(name.ends_with(".tmp"));
	}

	fn sample_model() -> (TempDir, AuroraModel) {
		let tmp_dir = tempfile::tempdir().expect("temp dir");
		std::fs::write(tmp_dir.path().join("Aurora.schema.json"), "{}").expect("schema file");
		let home = ModelHome::new(tmp_dir.path()).expect("model home");
		let card = Card {
			schema: None,
			id: "MIS-001".into(),
			card_type: "Mission".into(),
			card_subtype: None,
			name: "Provide Default Tooling".into(),
			description: "Test mission".into(),
			status: None,
			links: Vec::new(),
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let model = AuroraModel::new(home, vec![card]);
		(tmp_dir, model)
	}
}
