pub mod model;

use std::{
	fs::File,
	io::{BufReader, Write},
	path::{Path, PathBuf},
	vec,
};

use jsonschema::{Validator, draft7::meta};
use log::debug;
use thiserror::Error;

use crate::{
	aurora::model::{CompactModelBorrowed, Model, ModelError},
	cli::{bump_args::BumpArgs, output_args::OutputArgs},
	logging::LoggingError,
};

pub struct Aurora {
	pub models: Vec<Model>,
}

impl Aurora {
	pub fn load(path: &Path) -> Result<Self, AuroraError> {
		let aurora_path = Self::find_aurora_path(path)?;
		let schema_path = aurora_path.join("Aurora.schema.json");
		let compact_schema_path = aurora_path.join("Aurora.compact.schema.json");
		debug!("Aurora home found at {}", aurora_path.display());

		let card_validator = Self::validate_and_load_schema(&schema_path)?;
		let compact_validator = Self::validate_and_load_schema(&compact_schema_path)?;
		debug!("Loaded schema and validators.");

		let models = Model::load(&aurora_path, &card_validator, &compact_validator)?;
		debug!("Loaded {} models.", models.len());

		Ok(Self { models })
	}

	pub fn validate(&self) -> Result<Vec<String>, AuroraError> {
		let mut all_errors: Vec<String> = Vec::new();
		for model in &self.models {
			all_errors.extend(model.card_schema_errors.clone());
			all_errors.extend(model.card_invariant_errors.clone());
		}

		if all_errors.is_empty() {
			all_errors.push("All models are valid.".to_string());
		}
		Ok(all_errors)
	}

	pub fn render_models(&self, args: &OutputArgs) -> Result<Vec<String>, AuroraError> {
		std::fs::create_dir_all(&args.output_path)?;
		for model in &self.models {
			model.render_markdown(args)?;
		}
		self.write_root_readme(args)?;
		Ok(vec![format!(
			"Rendered cards for {} models.",
			self.models.len()
		)])
	}

	pub fn render_views(&self, args: &OutputArgs) -> Result<Vec<String>, AuroraError> {
		let mut results: Vec<String> = Vec::new();
		for model in &self.models {
			model.render_views(args)?;
			results.push(format!(
				"Rendered views for model {}.",
				model.mission_card.id
			));
		}
		Ok(results)
	}

	pub fn render_all(&self, args: &OutputArgs) -> Result<Vec<String>, AuroraError> {
		let mut render_args = args.clone();
		if render_args.clear {
			Model::clear_output_dir(&render_args.output_path)?;
			render_args.clear = false;
		}

		// Render views first so the model indexer can find them
		let mut result = self.render_views(&render_args)?;
		result.extend(self.render_models(&render_args)?);
		Ok(result)
	}

	pub fn compact(&self, args: &OutputArgs) -> Result<Vec<String>, AuroraError> {
		let mut results: Vec<String> = Vec::new();

		for model in &self.models {
			let compact_model = CompactModelBorrowed::from(model);
			let output_path = args
				.output_path
				.join(format!("AGENT-{}.json", model.mission_card.id));
			let mut file = File::create(&output_path)?;
			serde_json::to_writer_pretty(&mut file, &compact_model)?;

			results.push(format!(
				"Wrote compact model for {} to {}.",
				model.mission_card.id,
				output_path.display()
			));
		}
		Ok(results)
	}

	pub fn bump_patch(&mut self, args: &BumpArgs) -> Result<Vec<String>, AuroraError> {
		for model in &mut self.models {
			if model.cards.contains_key(&args.card_id) {
				model.bump_patch(args)?;
				return Ok(vec![format!(
					"Bumped patch version for card {} in model {}.",
					args.card_id, model.mission_card.id
				)]);
			}
		}
		Err(AuroraError::CardNotFound(args.card_id.clone()))
	}

	pub fn bump_minor(&mut self, args: &BumpArgs) -> Result<Vec<String>, AuroraError> {
		for model in &mut self.models {
			if model.cards.contains_key(&args.card_id) {
				model.bump_minor(args)?;
				return Ok(vec![format!(
					"Bumped minor version for card {} in model {}.",
					args.card_id, model.mission_card.id
				)]);
			}
		}
		Err(AuroraError::CardNotFound(args.card_id.clone()))
	}

	pub fn bump_major(&mut self, args: &BumpArgs) -> Result<Vec<String>, AuroraError> {
		for model in &mut self.models {
			if model.cards.contains_key(&args.card_id) {
				model.bump_major(args)?;
				return Ok(vec![format!(
					"Bumped major version for card {} in model {}.",
					args.card_id, model.mission_card.id
				)]);
			}
		}
		Err(AuroraError::CardNotFound(args.card_id.clone()))
	}

	fn find_aurora_path(path: &Path) -> Result<PathBuf, AuroraError> {
		let start = if path.is_file() {
			path.parent().unwrap_or(path)
		} else {
			path
		};

		let mut current: Option<&Path> = Some(start);
		while let Some(dir) = current {
			if Self::is_aurora_home(dir) {
				return Ok(dir.to_path_buf());
			}

			let aurora_folder = dir.join("aurora");
			if Self::is_aurora_home(&aurora_folder) {
				return Ok(aurora_folder);
			}

			let docs_design_aurora = dir.join("docs").join("design").join("aurora");
			if Self::is_aurora_home(&docs_design_aurora) {
				return Ok(docs_design_aurora);
			}

			current = dir.parent();
		}

		Err(AuroraError::InvalidPath(path.display().to_string()))
	}

	fn is_aurora_home(path: &Path) -> bool {
		path.is_dir()
			&& path.join("Aurora.schema.json").is_file()
			&& path.join("Aurora.compact.schema.json").is_file()
	}

	fn validate_and_load_schema(path: &Path) -> Result<Validator, AuroraError> {
		let file = File::open(path)?;
		let reader = BufReader::new(file);
		let schema: serde_json::Value = serde_json::from_reader(reader)?;
		if meta::is_valid(&schema) {
			Validator::new(&schema)
				.map_err(|err| AuroraError::SchemaValidationError(err.to_string()))
		} else {
			Err(AuroraError::InvalidSchema)
		}
	}

	fn write_root_readme(&self, args: &OutputArgs) -> Result<(), AuroraError> {
		let mut entries: Vec<String> = Vec::new();
		for model in &self.models {
			let sanitized_name = Model::sanitize_name(&model.mission_card.name);
			let mission_readme = format!("README-{}-{}.md", model.mission_card.id, sanitized_name);
			entries.push(format!(
				"- [{}]({})",
				model.mission_card.name, mission_readme
			));
		}
		entries.sort();
		let mut file = File::create(args.output_path.join("README.md"))?;
		writeln!(file, "# Aurora Models\n")?;
		for entry in entries {
			writeln!(file, "{}", entry)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::Aurora;
	use crate::cli::output_args::OutputArgs;
	use std::{
		collections::HashMap,
		fs,
		path::{Path, PathBuf},
		time::{SystemTime, UNIX_EPOCH},
	};

	fn unique_temp_dir(prefix: &str) -> PathBuf {
		let nanos = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_nanos();
		std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), nanos))
	}

	fn write_empty_file(path: &Path) {
		fs::write(path, "{}").expect("write temp file");
	}

	#[test]
	fn render_views_includes_mission_root_when_configured() {
		use crate::aurora::model::{Card, Model};

		let mission_value = serde_json::json!({
			"$schema": "./Aurora.schema.json",
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Example Mission",
			"description": "Example mission for view rendering tests.",
			"audit_trail": { "hash": null, "version": "0.1.0", "history": [] },
			"links": [{ "relationship": "establishes", "target": "DRI-001" }]
		});
		let driver_value = serde_json::json!({
			"$schema": "./Aurora.schema.json",
			"id": "DRI-001",
			"card_type": "Driver",
			"name": "Example Driver",
			"description": "Example driver for view rendering tests.",
			"audit_trail": { "hash": null, "version": "0.1.0", "history": [] }
		});

		let mission_card = Card::from_value(Path::new("MIS-001.json"), mission_value)
			.expect("create mission card");
		let driver_card =
			Card::from_value(Path::new("DRI-001.json"), driver_value).expect("create driver card");

		let mut cards: HashMap<String, Card> = HashMap::new();
		cards.insert(driver_card.id.clone(), driver_card);

		let model = Model {
			mission_card,
			compact_model: None,
			cards,
			adjacency: HashMap::new(),
			card_schema_errors: Vec::new(),
			card_invariant_errors: Vec::new(),
		};

		let out_dir = unique_temp_dir("aurora_cli_views");
		let args = OutputArgs {
			output_path: out_dir.clone(),
			clear: true,
		};
		model.render_views(&args).expect("render views");

		let requirements_view = out_dir.join("MIS-001").join("Requirements.view.md");
		let contents = fs::read_to_string(&requirements_view).expect("read rendered view");
		assert!(
			contents.contains("\tMIS-001"),
			"expected mission node to be rendered in Requirements view"
		);

		fs::remove_dir_all(&out_dir).expect("cleanup temp output");
	}

	#[test]
	fn render_views_includes_boundary_and_note_children_of_included_cards() {
		use crate::aurora::model::{Card, Model};

		let mission_value = serde_json::json!({
			"$schema": "./Aurora.schema.json",
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Example Mission",
			"description": "Example mission for view rendering tests.",
			"audit_trail": { "hash": null, "version": "0.1.0", "history": [] },
			"links": [{ "relationship": "establishes", "target": "DRI-001" }]
		});
		let driver_value = serde_json::json!({
			"$schema": "./Aurora.schema.json",
			"id": "DRI-001",
			"card_type": "Driver",
			"name": "Example Driver",
			"description": "Example driver with boundary and note children.",
			"audit_trail": { "hash": null, "version": "0.1.0", "history": [] },
			"links": [
				{ "relationship": "drives", "target": "REQ-001" },
				{ "relationship": "includes", "target": "BND-001" },
				{ "relationship": "annotated_by", "target": "NOT-001" }
			]
		});
		let requirement_value = serde_json::json!({
			"$schema": "./Aurora.schema.json",
			"id": "REQ-001",
			"card_type": "Requirement",
			"name": "Example Requirement",
			"description": "Example requirement contained by a boundary.",
			"audit_trail": { "hash": null, "version": "0.1.0", "history": [] }
		});
		let boundary_value = serde_json::json!({
			"$schema": "./Aurora.schema.json",
			"id": "BND-001",
			"card_type": "Boundary",
			"name": "Example Boundary",
			"description": "Example boundary containing the requirement.",
			"audit_trail": { "hash": null, "version": "0.1.0", "history": [] },
			"links": [{ "relationship": "contains", "target": "REQ-001" }]
		});
		let note_value = serde_json::json!({
			"$schema": "./Aurora.schema.json",
			"id": "NOT-001",
			"card_type": "Note",
			"name": "Example Note",
			"description": "Example note linked from the driver.",
			"audit_trail": { "hash": null, "version": "0.1.0", "history": [] }
		});

		let mission_card = Card::from_value(Path::new("MIS-001.json"), mission_value)
			.expect("create mission card");
		let driver_card =
			Card::from_value(Path::new("DRI-001.json"), driver_value).expect("create driver card");
		let requirement_card = Card::from_value(Path::new("REQ-001.json"), requirement_value)
			.expect("create requirement card");
		let boundary_card = Card::from_value(Path::new("BND-001.json"), boundary_value)
			.expect("create boundary card");
		let note_card =
			Card::from_value(Path::new("NOT-001.json"), note_value).expect("create note card");

		let mut cards: HashMap<String, Card> = HashMap::new();
		cards.insert(driver_card.id.clone(), driver_card);
		cards.insert(requirement_card.id.clone(), requirement_card);
		cards.insert(boundary_card.id.clone(), boundary_card);
		cards.insert(note_card.id.clone(), note_card);

		let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
		adjacency.insert("BND-001".to_string(), vec!["REQ-001".to_string()]);

		let model = Model {
			mission_card,
			compact_model: None,
			cards,
			adjacency,
			card_schema_errors: Vec::new(),
			card_invariant_errors: Vec::new(),
		};

		let out_dir = unique_temp_dir("aurora_cli_views_children");
		let args = OutputArgs {
			output_path: out_dir.clone(),
			clear: true,
		};
		model.render_views(&args).expect("render views");

		let requirements_view = out_dir.join("MIS-001").join("Requirements.view.md");
		let contents = fs::read_to_string(&requirements_view).expect("read rendered view");
		assert!(
			contents.contains("subgraph BND-001"),
			"expected boundary subgraph to be rendered when linked from an included card"
		);
		assert!(
			contents.contains("\tNOT-001"),
			"expected note node to be rendered when linked from an included card"
		);

		fs::remove_dir_all(&out_dir).expect("cleanup temp output");
	}

	#[test]
	fn find_aurora_path_finds_docs_design_aurora_from_repo_root() {
		let root = unique_temp_dir("aurora_cli_find_model");
		fs::create_dir_all(&root).expect("create temp root");
		let model_home = root.join("docs").join("design").join("aurora");
		fs::create_dir_all(&model_home).expect("create aurora home");
		write_empty_file(&model_home.join("Aurora.schema.json"));
		write_empty_file(&model_home.join("Aurora.compact.schema.json"));

		let found = Aurora::find_aurora_path(&root).expect("find aurora home");
		assert_eq!(found, model_home);

		fs::remove_dir_all(&root).expect("cleanup temp root");
	}

	#[test]
	fn find_aurora_path_walks_upwards_from_nested_tool_dir() {
		let root = unique_temp_dir("aurora_cli_find_model_nested");
		let nested = root.join("tools").join("aurora_cli");
		fs::create_dir_all(&nested).expect("create nested dir");
		let model_home = root.join("docs").join("design").join("aurora");
		fs::create_dir_all(&model_home).expect("create aurora home");
		write_empty_file(&model_home.join("Aurora.schema.json"));
		write_empty_file(&model_home.join("Aurora.compact.schema.json"));

		let found = Aurora::find_aurora_path(&nested).expect("find aurora home");
		assert_eq!(found, model_home);

		fs::remove_dir_all(&root).expect("cleanup temp root");
	}

	#[test]
	fn find_aurora_path_accepts_mission_file_path() {
		let root = unique_temp_dir("aurora_cli_find_model_file");
		let model_home = root.join("docs").join("design").join("aurora");
		fs::create_dir_all(&model_home).expect("create aurora home");
		write_empty_file(&model_home.join("Aurora.schema.json"));
		write_empty_file(&model_home.join("Aurora.compact.schema.json"));

		let mission_path = model_home.join("MIS-001-Example.json");
		write_empty_file(&mission_path);

		let found = Aurora::find_aurora_path(&mission_path).expect("find aurora home");
		assert_eq!(found, model_home);

		fs::remove_dir_all(&root).expect("cleanup temp root");
	}
}

/// Errors that can occur while loading, validating, or rendering Aurora models.
#[derive(Debug, Error)]
pub enum AuroraError {
	#[error("An I/O error occurred: {0}")]
	IoError(#[from] std::io::Error),
	#[error("The Aurora.schema.json file is invalid")]
	InvalidSchema,
	#[error("Schema validation failed: {0}")]
	SchemaValidationError(String),
	#[error("A Model error occurred: {0}")]
	ModelError(#[from] ModelError),
	#[error("The specified path is not a valid Aurora directory: {0}")]
	InvalidPath(String),
	#[error("A logging error occurred: {0}")]
	LoggingError(#[from] LoggingError),
	#[error("A serialization/deserialization error occurred: {0}")]
	SerdeError(#[from] serde_json::Error),
	#[error("Card with ID '{0}' not found in any model.")]
	CardNotFound(String),
}
