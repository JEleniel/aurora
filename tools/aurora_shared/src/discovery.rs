use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::errors::{AuroraError, Result};
use crate::model::{AuroraModel, Card};

const DEFAULT_INPUT: &str = "docs/design/aurora";

/// Root directory for an Aurora model home (contains the schema and mission cards).
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ModelHome {
	root: PathBuf,
}

impl ModelHome {
	pub fn new(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		if !path.exists() {
			return Err(AuroraError::MissingInput {
				path: path.to_path_buf(),
			});
		}
		let canonical = fs::canonicalize(path).map_err(|err| AuroraError::io(path, err))?;
		Ok(ModelHome { root: canonical })
	}

	pub fn root(&self) -> &Path {
		&self.root
	}

	pub fn default_input() -> PathBuf {
		PathBuf::from(DEFAULT_INPUT)
	}

	fn schema_candidates(&self) -> [PathBuf; 2] {
		[
			self.root.join("Aurora.schema.json"),
			self.root.join("Aurora.schema.jsjson"),
		]
	}

	pub fn has_schema(&self) -> bool {
		self.schema_candidates().iter().any(|p| p.exists())
	}
}

/// Discover one or more model homes starting from an input path.
pub fn discover_model_homes(input: impl AsRef<Path>) -> Result<Vec<ModelHome>> {
	let input = input.as_ref();
	if !input.exists() {
		return Err(AuroraError::MissingInput {
			path: input.to_path_buf(),
		});
	}

	if input.is_file() {
		return discover_from_file(input);
	}

	if is_model_home(input) {
		return Ok(vec![ModelHome::new(input)?]);
	}

	if let Some(child) = find_direct_child_home(input) {
		return Ok(vec![child]);
	}

	let mut homes: BTreeSet<ModelHome> = BTreeSet::new();
	for entry in WalkDir::new(input)
		.max_depth(5)
		.follow_links(false)
		.into_iter()
		.filter_map(|res| res.ok())
	{
		if entry.file_type().is_dir() && is_model_home(entry.path()) {
			let home = ModelHome::new(entry.path())?;
			homes.insert(home);
		}
	}

	Ok(homes.into_iter().collect())
}

fn discover_from_file(file: &Path) -> Result<Vec<ModelHome>> {
	let mut current = file.parent();
	while let Some(dir) = current {
		if is_model_home(dir) {
			return Ok(vec![ModelHome::new(dir)?]);
		}
		current = dir.parent();
	}
	Err(AuroraError::InvalidInput {
		message: format!(
			"Unable to discover an Aurora model home above file {}",
			file.display()
		),
	})
}

fn find_direct_child_home(parent: &Path) -> Option<ModelHome> {
	let child = parent.join("aurora");
	if is_model_home(&child) {
		return ModelHome::new(child).ok();
	}
	None
}

fn is_model_home(path: &Path) -> bool {
	path.join("Aurora.schema.json").exists() || path.join("Aurora.schema.jsjson").exists()
}

/// Load every card belonging to a model home into memory.
pub fn load_model(home: &ModelHome) -> Result<AuroraModel> {
	let mut cards: Vec<Card> = Vec::new();
	for entry in WalkDir::new(home.root())
		.follow_links(false)
		.into_iter()
		.filter_map(|res| res.ok())
	{
		if !entry.file_type().is_file() {
			continue;
		}
		let path = entry.path();
		if !is_card_file(path) {
			continue;
		}
		let contents = fs::read_to_string(path).map_err(|err| AuroraError::io(path, err))?;
		match serde_json::from_str::<Card>(&contents) {
			Ok(card) => {
				cards.push(card.with_source_path(path.to_path_buf()));
			}
			Err(err) => {
				return Err(AuroraError::parse(path, err));
			}
		}
	}

	if cards.is_empty() {
		return Err(AuroraError::InvalidInput {
			message: format!("no cards were discovered under {}", home.root().display()),
		});
	}

	Ok(AuroraModel::new(home.clone(), cards))
}

fn is_card_file(path: &Path) -> bool {
	match path.extension().and_then(|ext| ext.to_str()) {
		Some("json") | Some("jsjson") => {}
		_ => return false,
	}

	if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
		if name.starts_with("Aurora.schema") || name.starts_with("Aurora.compact.schema") {
			return false;
		}
		if name.starts_with("AGENT-") {
			return false;
		}
	}

	true
}
