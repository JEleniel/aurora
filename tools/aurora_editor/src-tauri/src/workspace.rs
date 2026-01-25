//! Workspace path and file safety helpers for the editor backend.

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};

/// Canonicalize a directory path and ensure it exists.
pub fn canonicalize_dir(path: impl AsRef<Path>) -> Result<PathBuf, String> {
	let raw = path.as_ref().to_string_lossy();
	let trimmed = raw.trim();
	if trimmed.is_empty() {
		return Err("Workspace path must not be empty".to_string());
	}
	let path = expand_tilde(&PathBuf::from(trimmed))?;
	if !path.exists() {
		return Err(format!("Workspace path does not exist: {}", path.display()));
	}
	if !path.is_dir() {
		return Err(format!(
			"Workspace path is not a directory: {}",
			path.display()
		));
	}
	fs::canonicalize(&path)
		.map_err(|err| format!("Failed to resolve workspace path {}: {err}", path.display()))
}

/// Resolve a path into an absolute path under the workspace root.
pub fn resolve_workspace_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
	let trimmed = relative.trim();
	if trimmed.is_empty() {
		return Err("Path must not be empty".to_string());
	}
	let candidate = expand_tilde(&PathBuf::from(trimmed))?;
	let normalized = if candidate.is_absolute() {
		let normalized = normalize_path(&candidate);
		if !normalized.starts_with(root) {
			return Err("Path must live within the workspace root".to_string());
		}
		normalized
	} else {
		let joined = root.join(candidate);
		normalize_path(&joined)
	};
	if !normalized.starts_with(root) {
		return Err("Resolved path escapes the workspace root".to_string());
	}
	Ok(normalized)
}

/// Convert an absolute path into a workspace-relative string.
pub fn workspace_relative(root: &Path, path: &Path) -> Result<String, String> {
	let relative = path.strip_prefix(root).map_err(|_| {
		format!(
			"Path {} is not within workspace root {}",
			path.display(),
			root.display()
		)
	})?;
	Ok(relative.to_string_lossy().to_string())
}

/// Compute a schema path relative to a card file location.
pub fn schema_relative_path(model_home: &Path, card_path: &Path) -> Result<String, String> {
	let relative = card_path.strip_prefix(model_home).map_err(|_| {
		format!(
			"Card path {} is not within model home {}",
			card_path.display(),
			model_home.display()
		)
	})?;
	let parent = relative.parent();
	let depth = parent.map_or(0, |dir| dir.components().count());
	if depth == 0 {
		return Ok("./Aurora.schema.jsjson".to_string());
	}
	let mut prefix = String::new();
	for _ in 0..depth {
		prefix.push_str("../");
	}
	Ok(format!("{prefix}Aurora.schema.jsjson"))
}

/// Write a JSON value using tab-indented pretty formatting and atomic replacement.
pub fn write_json_atomic(path: &Path, value: &serde_json::Value) -> Result<(), String> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)
			.map_err(|err| format!("Failed to create directory {}: {err}", parent.display()))?;
	}
	let tmp_path = tmp_path(path);
	{
		let file = File::create(&tmp_path)
			.map_err(|err| format!("Failed to create {}: {err}", tmp_path.display()))?;
		let formatter = PrettyFormatter::with_indent(b"\t");
		let mut serializer = Serializer::with_formatter(file, formatter);
		value
			.serialize(&mut serializer)
			.map_err(|err| format!("Failed to serialize JSON for {}: {err}", path.display()))?;
	}
	append_newline(&tmp_path)?;
	fs::rename(&tmp_path, path)
		.map_err(|err| format!("Failed to finalize {}: {err}", path.display()))?;
	Ok(())
}

fn append_newline(path: &Path) -> Result<(), String> {
	let mut file = fs::OpenOptions::new()
		.append(true)
		.open(path)
		.map_err(|err| format!("Failed to open {}: {err}", path.display()))?;
	file.write_all(b"\n")
		.map_err(|err| format!("Failed to write {}: {err}", path.display()))?;
	Ok(())
}

fn normalize_path(path: &Path) -> PathBuf {
	let mut normalized = PathBuf::new();
	for component in path.components() {
		match component {
			Component::CurDir => {}
			Component::ParentDir => {
				let _ = normalized.pop();
			}
			Component::Prefix(prefix) => {
				normalized.push(prefix.as_os_str());
			}
			Component::RootDir => normalized.push(Component::RootDir.as_os_str()),
			Component::Normal(value) => normalized.push(value),
		}
	}
	normalized
}

fn expand_tilde(path: &Path) -> Result<PathBuf, String> {
	let raw = path.to_string_lossy();
	if raw == "~" {
		return resolve_home_dir();
	}
	if let Some(remainder) = raw.strip_prefix("~/") {
		return Ok(resolve_home_dir()?.join(remainder));
	}
	if raw.starts_with('~') {
		return Err("Tilde paths must use the form ~/path".to_string());
	}
	Ok(path.to_path_buf())
}

fn resolve_home_dir() -> Result<PathBuf, String> {
	let candidates = ["HOME", "USERPROFILE"];
	for key in candidates {
		if let Ok(value) = env::var(key) {
			let trimmed = value.trim();
			if !trimmed.is_empty() {
				return Ok(PathBuf::from(trimmed));
			}
		}
	}
	Err("Unable to resolve the home directory for ~ expansion".to_string())
}

fn tmp_path(path: &Path) -> PathBuf {
	let mut tmp_name = path
		.file_name()
		.and_then(|name| name.to_str())
		.map(|name| format!(".{name}.tmp"))
		.unwrap_or_else(|| ".aurora.tmp".to_string());
	if tmp_name
		== path
			.file_name()
			.and_then(|name| name.to_str())
			.unwrap_or("")
	{
		tmp_name.push_str(".tmp");
	}
	path.with_file_name(tmp_name)
}
