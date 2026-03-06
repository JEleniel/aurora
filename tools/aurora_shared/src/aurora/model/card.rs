use crate::registry::CardRegistry;
use jsonschema::{CompilationError, Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use tracing::debug;

use super::{Attributes, Link, attributes_markdown};

const CARD_MARKDOWN_TEMPLATE: &str = include_str!("card.template.md");

#[path = "card_persistence.rs"]
mod card_persistence;

pub use card_persistence::NewCard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
	#[serde(rename = "$schema", default)]
	pub schema: Option<String>,
	pub id: String,
	pub card_type: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub card_subtype: Option<String>,
	pub name: String,
	pub description: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub status: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub boundary: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub notes: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub icon: Option<String>,
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub attributes: Attributes,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub external_references: Vec<String>,
	pub links: Vec<Link>,
	#[serde(skip)]
	pub source_path: PathBuf,
	#[serde(skip)]
	pub validation_errors: Vec<String>,
	#[serde(skip)]
	pub validation_warnings: Vec<String>,
}

impl Card {
	pub fn try_load(path: &Path, card_schema: &serde_json::Value) -> Result<Self, CardError> {
		debug!("Loading card from {}", path.display());

		let data = std::fs::read_to_string(path)?;

		let card_json: serde_json::Value = serde_json::from_str(&data)
			.map_err(|e| CardError::ParseError(path.display().to_string(), e))?;
		let mut validation_errors = Self::validate_against_schema(&card_json, card_schema)?;
		let mut validation_warnings: Vec<String> = Vec::new();
		Self::validate_schema_reference(
			path,
			&card_json,
			"Aurora.card.schema.json",
			&mut validation_errors,
			&mut validation_warnings,
		);

		let mut card: Card = serde_json::from_str(&data)
			.map_err(|e| CardError::ParseError(path.display().to_string(), e))?;
		card.source_path = path.to_path_buf();
		card.validation_errors = validation_errors;
		card.validation_warnings = validation_warnings;
		Ok(card)
	}

	pub fn check_registry(&self, registry: &CardRegistry) -> Vec<String> {
		let mut warnings: Vec<String> = Vec::new();

		if !registry.check(&self.card_type) {
			warnings.push(format!("Card has unknown card type: {}", self.card_type));
		}

		if let Some(icon) = &self.icon
			&& !registry.has_icon(icon)
		{
			warnings.push(format!(
				"Card {} uses unknown icon override '{}'",
				self.id, icon
			));
		}

		for link in self.links.iter() {
			let target_acronym = link
				.target
				.split('-')
				.next()
				.map(str::trim)
				.unwrap_or_default();
			if target_acronym.len() != 3 {
				warnings.push(format!(
					"Card {} has link with invalid target id format: {}",
					self.id, link.target
				));
				continue;
			}

			let target_card_def = match registry.try_get_by_acronym(target_acronym) {
				Ok(target_card_def) => target_card_def,
				Err(_) => {
					warnings.push(format!(
						"Card links to unknown target card type: {}",
						link.target
					));
					continue;
				}
			};

			if !registry.check_link(
				&self.card_type,
				&link.relationship,
				&target_card_def.card_type,
			) {
				warnings.push(format!(
					"Card {} has unknown relationship '{}' to target card {}",
					self.id, link.relationship, link.target
				));
			}
		}

		warnings
	}

	pub fn write_markdown<'a>(
		&self,
		path: &Path,
		markdown_paths_by_id: Option<&HashMap<String, PathBuf>>,
		audit_entries: impl Iterator<Item = &'a super::super::AuditLogEntry>,
	) {
		let mut markdown: String = String::from(CARD_MARKDOWN_TEMPLATE);

		let subtype = match &self.card_subtype {
			Some(subtype) => format!(" ({})", subtype.as_str()),
			None => "".to_string(),
		};

		let status = match &self.status {
			Some(status) => format!("**Status**: {}\n", status.as_str()),
			None => "".to_string(),
		};

		let boundary = match &self.boundary {
			Some(boundary) => format!("**Boundary**: {}\n", boundary),
			None => "".to_string(),
		};

		let notes = match &self.notes {
			Some(notes) => format!("## Notes\n\n{}\n", notes),
			None => "".to_string(),
		};

		let attributes = attributes_markdown(&self.attributes);
		let references = references_markdown(&self.external_references, &self.source_path, path);

		let mut links: String = String::new();
		if self.links.is_empty() {
			links.push_str("_No links defined._");
		} else {
			for link in &self.links {
				let href = markdown_paths_by_id
					.and_then(|map| map.get(&link.target))
					.and_then(|target_path| {
						let from_dir = path.parent()?;
						Some(relative_href(from_dir, target_path))
					});
				match href {
					Some(href) => links.push_str(link.markdown_with_href(href.as_str()).as_str()),
					None => links.push_str(&format!("- {} {}\n", link.relationship, link.target)),
				}
			}
		}

		let mut history = String::new();
		history.push_str("| Timestamp | Editor | Change |\n");
		history.push_str("|-----------|--------|--------|\n");
		let mut any = false;
		for entry in audit_entries {
			any = true;
			let change_summary = super::super::AuditLog::change_summary_for_target(entry, &self.id);
			history.push_str(&format!(
				"| {} | {} | {} |\n",
				entry
					.timestamp
					.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
				entry.editor,
				change_summary,
			));
		}
		if !any {
			history.push_str("| _No entries_ |  |  |\n");
		}

		let version = self.version.clone().unwrap_or_default();

		markdown = markdown
			.replace("{{card_type}}", &self.card_type)
			.replace("{{card_subtype}}", &subtype)
			.replace("{{id}}", &self.id)
			.replace("{{name}}", &self.name)
			.replace("{{description}}", &self.description)
			.replace("{{status}}", &status)
			.replace("{{boundary}}", &boundary)
			.replace("{{notes}}", &notes)
			.replace("{{attributes}}", &attributes)
			.replace("{{references}}", &references)
			.replace("{{links}}", &links)
			.replace("{{version}}", &version)
			.replace("{{history}}", &history)
			.replace("|\n\n", "|\n")
			.replace("\n\n\n\n", "\n\n");

		std::fs::write(path, markdown).unwrap();
	}

	pub fn get_compact(&self) -> Value {
		let mut map = serde_json::Map::new();
		map.insert("id".to_string(), Value::String(self.id.clone()));
		map.insert(
			"card_type".to_string(),
			Value::String(self.card_type.clone()),
		);
		if let Some(card_subtype) = &self.card_subtype {
			map.insert(
				"card_subtype".to_string(),
				Value::String(card_subtype.clone()),
			);
		}
		map.insert("name".to_string(), Value::String(self.name.clone()));
		if let Some(status) = &self.status {
			map.insert("status".to_string(), Value::String(status.clone()));
		}
		if let Some(boundary) = &self.boundary {
			map.insert("boundary".to_string(), Value::String(boundary.clone()));
		}
		if !self.external_references.is_empty() {
			map.insert(
				"external_references".to_string(),
				Value::Array(
					self.external_references
						.iter()
						.cloned()
						.map(Value::String)
						.collect(),
				),
			);
		}
		map.insert(
			"attributes".to_string(),
			Value::Object(
				self.attributes
					.iter()
					.map(|(key, value)| (key.clone(), value.clone()))
					.collect(),
			),
		);
		map.insert(
			"links".to_string(),
			Value::Array(
				self.links
					.iter()
					.map(|link| {
						serde_json::json!({
							"relationship": link.relationship,
							"target": link.target,
						})
					})
					.collect(),
			),
		);

		Value::Object(map)
	}

	fn validate_against_schema(
		card_json: &Value,
		card_schema: &Value,
	) -> Result<Vec<String>, CardError> {
		let compiled_schema = JSONSchema::options()
			.with_draft(Draft::Draft7)
			.compile(card_schema)?;

		let result = compiled_schema.validate(card_json);
		match result {
			Ok(_) => Ok(vec![]),
			Err(errors) => {
				let mut error_messages = Vec::new();
				for error in errors {
					error_messages.push(error.to_string());
				}
				Ok(error_messages)
			}
		}
	}

	fn validate_schema_reference(
		path: &Path,
		card_json: &Value,
		expected_schema_file_name: &str,
		validation_errors: &mut Vec<String>,
		validation_warnings: &mut Vec<String>,
	) {
		let schema_ref = card_json.get("$schema").and_then(Value::as_str);
		let Some(schema_ref) = schema_ref else {
			validation_errors.push("Missing required $schema property.".to_string());
			return;
		};

		let Some(parent) = path.parent() else {
			validation_warnings.push("Could not resolve $schema path for validation.".to_string());
			return;
		};

		let resolved_path = parent.join(schema_ref);
		match std::fs::canonicalize(&resolved_path) {
			Ok(canonicalized) => {
				let schema_file_name = canonicalized
					.file_name()
					.and_then(|name| name.to_str())
					.unwrap_or_default();
				if schema_file_name != expected_schema_file_name {
					validation_warnings.push(format!(
						"$schema reference resolves to '{}' but '{}' is expected.",
						schema_file_name, expected_schema_file_name
					));
				}
			}
			Err(_) => validation_warnings.push(format!(
				"$schema reference '{}' could not be resolved; validating against '{}' anyway.",
				schema_ref, expected_schema_file_name
			)),
		}
	}
}

fn relative_href(from_dir: &Path, target_path: &Path) -> String {
	let rel = relative_path(from_dir, target_path);
	rel.to_string_lossy().replace('\\', "/")
}

fn relative_path(from_dir: &Path, to: &Path) -> PathBuf {
	let from_components: Vec<_> = from_dir.components().collect();
	let to_components: Vec<_> = to.components().collect();

	let mut common_len = 0usize;
	while common_len < from_components.len()
		&& common_len < to_components.len()
		&& from_components[common_len] == to_components[common_len]
	{
		common_len += 1;
	}

	let mut out = PathBuf::new();
	for _ in common_len..from_components.len() {
		out.push("..");
	}
	for comp in &to_components[common_len..] {
		out.push(comp.as_os_str());
	}
	if out.as_os_str().is_empty() {
		out.push(".");
	}
	out
}

fn references_markdown(references: &[String], source_path: &Path, markdown_path: &Path) -> String {
	if references.is_empty() {
		return "_No references defined._".to_string();
	}

	let mut out = String::new();
	for reference in references {
		let href = resolve_reference_href(reference, source_path, markdown_path);
		out.push_str(format!("- [{}]({})\n", reference, href).as_str());
	}
	out
}

fn resolve_reference_href(reference: &str, source_path: &Path, markdown_path: &Path) -> String {
	if is_external_reference(reference) {
		return reference.to_string();
	}

	let Some(markdown_dir) = markdown_path.parent() else {
		return reference.to_string();
	};
	let Some(source_dir) = source_path.parent() else {
		return reference.to_string();
	};

	let target_path = normalize_path(source_dir.join(reference).as_path());
	relative_href(markdown_dir, target_path.as_path())
}

fn is_external_reference(reference: &str) -> bool {
	let trimmed = reference.trim();
	trimmed.starts_with("http://")
		|| trimmed.starts_with("https://")
		|| trimmed.starts_with("ftp://")
		|| trimmed.starts_with("ftps://")
		|| trimmed.starts_with("mailto:")
		|| trimmed.starts_with("file://")
}

fn normalize_path(path: &Path) -> PathBuf {
	let mut out = PathBuf::new();
	let mut has_root = false;

	for component in path.components() {
		match component {
			Component::Prefix(prefix) => out.push(prefix.as_os_str()),
			Component::RootDir => {
				has_root = true;
				out.push(component.as_os_str());
			}
			Component::CurDir => {}
			Component::ParentDir => {
				if !out.pop() && !has_root {
					out.push("..");
				}
			}
			Component::Normal(part) => out.push(part),
		}
	}

	if out.as_os_str().is_empty() {
		out.push(".");
	}
	out
}

#[derive(Debug, Error)]
pub enum CardError {
	#[error("I/O error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Parse error at {0}: {1}")]
	ParseError(String, serde_json::Error),
	#[error("Invalid card file: {0}")]
	InvalidCard(String),
	#[error("Invalid filename: {0}")]
	InvalidFilename(String),
	#[error("Could not resolve card schema reference: {0}")]
	SchemaReference(String),
	#[error("Card validation failed: {0:?}")]
	ValidationErrors(Vec<String>),
	#[error("Schema compilation error: {0}")]
	SchemaCompilationError(#[from] CompilationError),
	#[error("Card not found: {0}")]
	CardNotFound(String),
	#[error("Registry error: {0}")]
	RegistryError(#[from] crate::registry::RegistryError),
}

#[cfg(test)]
#[path = "card_tests.rs"]
mod card_tests;
