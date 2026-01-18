//! Loading Aurora card models from disk.

use crate::error::AuroraCliError;
use crate::model::{Card, Model};
use indexmap::IndexMap;
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

/// Load an Aurora model rooted at either a mission card or a model folder.
pub fn load_model(input_path: &Path) -> Result<Model, AuroraCliError> {
	let resolved = resolve_existing_path(input_path)?;
	let root = if resolved.is_file() {
		resolved
			.parent()
			.map(Path::to_path_buf)
			.ok_or_else(|| AuroraCliError::MissionHasNoParent(resolved.clone()))?
	} else if resolved.is_dir() {
		resolved
	} else {
		return Err(AuroraCliError::InvalidModelRoot(resolved));
	};
	let root = canonicalize(&root)?;
	let schema_path = root.join("Aurora.schema.json");
	if !schema_path.exists() {
		return Err(AuroraCliError::MissingSchema { path: schema_path });
	}

	let files = collect_card_files(&root)?;

	let mut cards: IndexMap<String, Card> = IndexMap::new();
	for file in files {
		if file.file_name().and_then(|s| s.to_str()) == Some("Aurora.schema.json") {
			continue;
		}
		let raw_text = fs::read_to_string(&file).map_err(|source| AuroraCliError::Io {
			path: file.clone(),
			source,
		})?;
		let value: Value =
			serde_json::from_str(&raw_text).map_err(|source| AuroraCliError::Json {
				path: file.clone(),
				source,
			})?;
		if value.get("id").is_none() {
			continue;
		}
		let mut card: Card =
			serde_json::from_value(value.clone()).map_err(|source| AuroraCliError::Json {
				path: file.clone(),
				source,
			})?;
		let relative = file
			.strip_prefix(&root)
			.map(|p| p.to_path_buf())
			.unwrap_or_else(|_| file.clone());
		card = card.with_paths(file.clone(), relative, value);
		if cards.contains_key(&card.id) {
			return Err(AuroraCliError::DuplicateCardId {
				id: card.id.clone(),
				path: file,
			});
		}
		cards.insert(card.id.clone(), card);
	}

	if cards.is_empty() {
		return Err(AuroraCliError::NoCards { root: root.clone() });
	}
	let mission_ids: Vec<_> = cards
		.values()
		.filter(|card| card.is_mission())
		.map(|card| card.id.clone())
		.collect();
	if mission_ids.is_empty() {
		return Err(AuroraCliError::MissingMission { root: root.clone() });
	}
	if mission_ids.len() > 1 {
		return Err(AuroraCliError::MultipleMissions {
			root: root.clone(),
			ids: mission_ids,
		});
	}
	Ok(Model::new(
		root.clone(),
		schema_path,
		cards,
		mission_ids[0].clone(),
	))
}

fn collect_card_files(root: &Path) -> Result<Vec<PathBuf>, AuroraCliError> {
	let mut files = Vec::new();
	for entry in WalkDir::new(root).follow_links(false) {
		let entry = match entry {
			Ok(value) => value,
			Err(err) => {
				let path = err.path().unwrap_or(root).to_path_buf();
				let message = err.to_string();
				let source = err
					.into_io_error()
					.unwrap_or_else(|| std::io::Error::other(message));
				return Err(AuroraCliError::Io { path, source });
			}
		};
		if entry.file_type().is_file()
			&& entry.path().extension().and_then(|s| s.to_str()) == Some("json")
		{
			files.push(entry.into_path());
		}
	}
	files.sort_by(|lhs, rhs| relative_cmp(root, lhs, rhs));
	Ok(files)
}

fn relative_cmp(root: &Path, lhs: &Path, rhs: &Path) -> std::cmp::Ordering {
	let left = lhs.strip_prefix(root).unwrap_or(lhs);
	let right = rhs.strip_prefix(root).unwrap_or(rhs);
	path_to_string(left).cmp(&path_to_string(right))
}

fn path_to_string(path: &Path) -> String {
	path.components()
		.map(|c| match c {
			Component::Normal(seg) => seg.to_string_lossy().to_string(),
			Component::CurDir => String::from("."),
			Component::ParentDir => String::from(".."),
			Component::RootDir => String::from("/"),
			Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().to_string(),
		})
		.collect::<Vec<_>>()
		.join("/")
}

fn resolve_existing_path(input: &Path) -> Result<PathBuf, AuroraCliError> {
	if input.exists() {
		return canonicalize(input);
	}
	if let Some(repo_root) = repo_root() {
		let candidate = repo_root.join(input);
		if candidate.exists() {
			return canonicalize(&candidate);
		}
	}
	Err(AuroraCliError::MissingModelPath(input.to_path_buf()))
}

fn canonicalize(path: &Path) -> Result<PathBuf, AuroraCliError> {
	std::fs::canonicalize(path).map_err(|source| AuroraCliError::Canonicalize {
		path: path.to_path_buf(),
		source,
	})
}

fn repo_root() -> Option<PathBuf> {
	let mut current = std::env::current_dir().ok()?;
	for _ in 0..16 {
		if current.join("AGENT_PROGRESS.md").exists()
			|| current.join(".git").exists()
			|| current.join("README.md").exists()
		{
			return Some(current);
		}
		if !current.pop() {
			break;
		}
	}
	None
}
