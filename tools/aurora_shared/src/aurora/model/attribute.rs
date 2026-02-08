use serde_json::Value;
use std::collections::BTreeMap;

/// Arbitrary per-card metadata for agents and tools.
///
/// Matches the Aurora v2 schema shape: an object with arbitrary keys.
pub type Attributes = BTreeMap<String, Value>;

pub fn attributes_markdown(attributes: &Attributes) -> String {
	if attributes.is_empty() {
		return "_No attributes defined._".to_string();
	}

	let mut md = String::new();
	for (key, value) in attributes {
		md.push_str(&format!("- **{}**: {}\n", key, value));
	}
	md
}

#[cfg(test)]
#[path = "attribute_tests.rs"]
mod attribute_tests;
