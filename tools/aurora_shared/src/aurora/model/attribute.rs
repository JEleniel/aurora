use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
	pub name: String,
	pub value: Value,
}

impl Attribute {
	pub fn get_markdown(&self) -> String {
		format!("- **{}**: {}\n", self.name, self.value)
	}
}
