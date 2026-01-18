use crate::error::AuroraCliError;
use crate::json_utils::sort_value;
use chrono::{SecondsFormat, Utc};
use semver::Version;
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Semantic levels for bump operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
	Major,
	Minor,
	Patch,
}

fn write_value_pretty_tabs(path: &Path, v: &Value) -> std::result::Result<(), AuroraCliError> {
	// Serialize with sorted keys already applied by caller
	let file = path.with_extension("json.tmp");
	let f = fs::File::create(&file).map_err(|source| AuroraCliError::Io {
		path: file.clone(),
		source,
	})?;
	let mut ser = serde_json::Serializer::with_formatter(
		f,
		serde_json::ser::PrettyFormatter::with_indent(b"\t"),
	);
	serde::Serialize::serialize(v, &mut ser).map_err(|source| AuroraCliError::Json {
		path: path.to_path_buf(),
		source,
	})?;
	fs::rename(&file, path).map_err(|source| AuroraCliError::Io {
		path: path.to_path_buf(),
		source,
	})?;
	Ok(())
}

fn derive_editor(editor_opt: &Option<String>) -> String {
	if let Some(e) = editor_opt {
		return e.clone();
	}
	if let Ok(u) = env::var("USER") {
		return u;
	}
	// whoami::username returns String
	let uname = whoami::username();
	if uname.is_empty() {
		"unknown".to_string()
	} else {
		uname
	}
}

/// Bump the audit_trail.version and append history entries for all given paths.
pub fn bump_paths(
	paths: &[PathBuf],
	level: Level,
	editor: Option<String>,
	_create_dirs: bool,
) -> std::result::Result<(), AuroraCliError> {
	// Collect files
	let mut files: Vec<PathBuf> = Vec::new();
	for p in paths {
		if p.is_file() {
			if let Some(name) = p.file_name().and_then(|s| s.to_str())
				&& name.eq_ignore_ascii_case("Aurora.schema.json")
			{
				continue;
			}
			files.push(p.clone());
		} else if p.is_dir() {
			for entry in WalkDir::new(p).into_iter().filter_map(|e| e.ok()) {
				let ep = entry.path();
				if ep.is_file()
					&& let Some(ext) = ep.extension().and_then(|s| s.to_str())
					&& ext.eq_ignore_ascii_case("json")
				{
					if let Some(name) = ep.file_name().and_then(|s| s.to_str())
						&& name.eq_ignore_ascii_case("Aurora.schema.json")
					{
						continue;
					}
					files.push(ep.to_path_buf());
				}
			}
		}
	}
	// Deterministic order
	files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

	let editor_name = derive_editor(&editor);
	for file in files {
		let s = fs::read_to_string(&file).map_err(|source| AuroraCliError::Io {
			path: file.clone(),
			source,
		})?;
		let mut v: Value = serde_json::from_str(&s).map_err(|source| AuroraCliError::Json {
			path: file.clone(),
			source,
		})?;

		// Ensure audit_trail exists and is an object
		let mut audit = v
			.get("audit_trail")
			.cloned()
			.unwrap_or(Value::Object(Map::new()));
		if !audit.is_object() {
			audit = Value::Object(Map::new());
		}
		// version
		let version_str = audit
			.get("version")
			.and_then(|x| x.as_str())
			.unwrap_or("0.0.0");
		let mut ver = Version::parse(version_str).unwrap_or_else(|_| Version::new(0, 0, 0));
		match level {
			Level::Major => {
				ver.major += 1;
				ver.minor = 0;
				ver.patch = 0;
			}
			Level::Minor => {
				ver.minor += 1;
				ver.patch = 0;
			}
			Level::Patch => {
				ver.patch += 1;
			}
		}
		// history
		let history = audit
			.get("history")
			.cloned()
			.unwrap_or(Value::Array(vec![]));
		let is_empty = match &history {
			Value::Array(arr) => arr.is_empty(),
			_ => true,
		};
		let event = if is_empty { "created" } else { "edited" };
		let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
		let mut entry = Map::new();
		entry.insert("editor".to_string(), Value::String(editor_name.clone()));
		entry.insert("timestamp".to_string(), Value::String(timestamp));
		entry.insert("event".to_string(), Value::String(event.to_string()));

		// build new history array
		let new_history = match history {
			Value::Array(mut arr) => {
				arr.push(Value::Object(entry));
				Value::Array(arr)
			}
			_ => Value::Array(vec![Value::Object(entry)]),
		};

		// set updated fields into audit
		if let Value::Object(mut amap) = audit {
			amap.insert("version".to_string(), Value::String(ver.to_string()));
			amap.insert("history".to_string(), new_history);
			audit = Value::Object(amap);
		}

		if let Some(mut_map) = v.as_object_mut() {
			mut_map.insert("audit_trail".to_string(), audit);
		}

		// sort object keys deterministically
		let sorted = sort_value(&v);
		// write back with tabs
		write_value_pretty_tabs(&file, &sorted)?;
	}

	Ok(())
}
