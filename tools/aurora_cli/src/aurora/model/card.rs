use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
	fs::File,
	io::BufWriter,
	path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
	#[serde(skip)]
	path: PathBuf,
	#[serde(rename = "$schema")]
	pub schema: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attributes: Option<Map<String, Value>>,
	pub audit_trail: AuditTrail,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub card_subtype: Option<String>,
	pub card_type: String,
	pub description: String,
	pub id: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub links: Option<Vec<Link>>,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<String>,
}

impl Card {
	pub fn validate(path: &PathBuf, validator: &Validator) -> Result<Vec<String>, CardError> {
		let file = File::open(path)?;
		let reader = std::io::BufReader::new(file);
		let json: Value = serde_json::from_reader(reader)?;

		let evaluation = validator.evaluate(&json);

		let mut results: Vec<String> = Vec::new();
		for error in evaluation.iter_errors() {
			results.push(error.error.to_string())
		}
		for annotation in evaluation.iter_annotations() {
			results.push(format!(
				"Annotation at {}: {:?}",
				annotation.instance_location, annotation.annotations
			))
		}

		Ok(results)
	}

	pub fn load(path: &Path) -> Result<Self, CardError> {
		let file = File::open(path)?;
		let reader = std::io::BufReader::new(file);
		let mut card: Card = serde_json::from_reader(reader)?;
		card.path = path.to_path_buf();
		Ok(card)
	}

	pub fn bump_patch(&mut self, editor: &str) -> Result<(), CardError> {
		let mut sv = semver::Version::parse(&self.audit_trail.version)?;
		sv.patch += 1;
		self.audit_trail.version = sv.to_string();
		self.audit_trail.history.push(HistoryEntry {
			editor: editor.to_string(),
			event: HistoryEvent::Edited,
			timestamp: chrono::Utc::now().to_rfc3339(),
		});
		self.write()?;

		Ok(())
	}

	pub fn bump_minor(&mut self, editor: &str) -> Result<(), CardError> {
		let mut sv = semver::Version::parse(&self.audit_trail.version)?;
		sv.minor += 1;
		sv.patch = 0;
		self.audit_trail.version = sv.to_string();
		self.audit_trail.history.push(HistoryEntry {
			editor: editor.to_string(),
			event: HistoryEvent::Edited,
			timestamp: chrono::Utc::now().to_rfc3339(),
		});
		self.write()?;

		Ok(())
	}

	pub fn bump_major(&mut self, editor: &str) -> Result<(), CardError> {
		let mut sv = semver::Version::parse(&self.audit_trail.version)?;
		sv.major += 1;
		sv.minor = 0;
		sv.patch = 0;
		self.audit_trail.version = sv.to_string();
		self.audit_trail.history.push(HistoryEntry {
			editor: editor.to_string(),
			event: HistoryEvent::Edited,
			timestamp: chrono::Utc::now().to_rfc3339(),
		});
		self.write()?;

		Ok(())
	}

	fn write(&self) -> Result<(), CardError> {
		let file = File::create(&self.path)?;
		let mut writer = BufWriter::new(file);
		serde_json::to_writer_pretty(&mut writer, self)?;
		Ok(())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
	pub relationship: String,
	pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
	pub hash: Option<String>,
	pub history: Vec<HistoryEntry>,
	pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistoryEvent {
	Created,
	Edited,
	Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
	pub editor: String,
	pub event: HistoryEvent,
	pub timestamp: String,
}

#[derive(Debug, Error)]
pub enum CardError {
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),
	#[error("JSON error: {0}")]
	Json(#[from] serde_json::Error),
	#[error("Regex error: {0}")]
	RegexError(#[from] regex::Error),
	#[error("Invalid card file name")]
	InvalidCardFileName,
	#[error("Schema validation error: {0}")]
	SchemaValidationError(String),
	#[error("SemVer error: {0}")]
	SemVerError(#[from] semver::Error),
}
