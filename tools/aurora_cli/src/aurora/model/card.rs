pub mod prefixes;
mod template;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
	fs::File,
	io::{BufWriter, Write},
	path::{Path, PathBuf},
};
use thiserror::Error;

use crate::aurora::model::view_definitions;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
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
	pub fn load(path: &Path) -> Result<Self, CardError> {
		let file = File::open(path)?;
		let reader = std::io::BufReader::new(file);
		let value: serde_json::Value = serde_json::from_reader(reader)?;
		Self::from_value(path, value)
	}

	pub(crate) fn from_value(path: &Path, value: serde_json::Value) -> Result<Self, CardError> {
		let mut card: Card = serde_json::from_value(value)?;
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

	pub fn render(&self, path: &PathBuf) -> Result<(), CardError> {
		let mut rendered = template::CARD_TEMPLATE.to_string();
		rendered = rendered.replace("{{card_type}}", &self.card_type);
		let card_subtype = match &self.card_subtype {
			Some(subtype) => format!(" ({})", subtype.as_str()),
			None => "".to_string(),
		};
		rendered = rendered.replace("{{card_subtype}}", &card_subtype);
		rendered = rendered.replace("{{name}}", &self.name);
		rendered = rendered.replace("{{description}}", &self.description);

		let attributes = match &self.attributes {
			Some(attrs) => {
				let mut attrs_str = String::new();
				for (key, value) in attrs {
					attrs_str.push_str(&format!("- **{}**: {}\n", key, value));
				}
				attrs_str
			}
			None => "None".to_string(),
		};
		rendered = rendered.replace("{{attributes}}", &attributes);

		let links = match &self.links {
			Some(links_vec) => {
				let mut links_str = String::new();
				for link in links_vec {
					links_str.push_str(&format!(
						"- **{}**: {}\n",
						link.relationship,
						format!(
							"../{}/{}.json",
							prefixes::get_card_type(&link.target[..3])
								.ok_or(CardError::InvalidLinkTarget(link.target.clone()))?,
							link.target
						)
					));
				}
				links_str
			}
			None => "None".to_string(),
		};

		rendered = rendered.replace("{{links}}", &links);

		for history in &self.audit_trail.history {
			rendered = rendered.replace(
				"{{history}}",
				&format!(
					"- **{}**: {} by {} \n{{{{history}}}}",
					history.timestamp,
					format!("{:?}", history.event),
					history.editor
				),
			);
		}
		rendered = rendered.replace("{{history}}", "");

		rendered = rendered.replace("{{version}}", &self.audit_trail.version);

		rendered = rendered.replace("{{id}}", &self.id);

		let file = File::create(path)?;
		let mut writer = BufWriter::new(file);
		writer.write_all(rendered.as_bytes())?;

		Ok(())
	}

	pub fn render_view_node(&self) -> String {
		let mut rendered: String = view_definitions::get_card_wrapper(&self.card_type).to_string();
		rendered = rendered.replace("{{id}}", &self.id);
		rendered = rendered.replace("{{name}}", &self.name);
		rendered = rendered.replace("{{card_type}}", &self.card_type);
		rendered
	}

	fn write(&self) -> Result<(), CardError> {
		let file = File::create(&self.path)?;
		let mut writer = BufWriter::new(file);
		serde_json::to_writer_pretty(&mut writer, self)?;
		Ok(())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Link {
	pub relationship: String,
	pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct AuditTrail {
	pub hash: Option<String>,
	pub history: Vec<HistoryEntry>,
	pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HistoryEvent {
	Created,
	Edited,
	Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
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
	#[error("Invalid link target: {0}")]
	InvalidLinkTarget(String),
}
