use serde_json::json;

use super::{Attributes, attributes_markdown};

#[test]
fn attributes_markdown_returns_empty_message() {
	let attributes = Attributes::new();
	let markdown = attributes_markdown(&attributes);
	assert_eq!(markdown, "_No attributes defined._");
}

#[test]
fn attributes_markdown_formats_entries() {
	let mut attributes = Attributes::new();
	attributes.insert("key".to_string(), json!("value"));
	attributes.insert("count".to_string(), json!(3));

	let markdown = attributes_markdown(&attributes);
	assert!(markdown.contains("**key**"));
	assert!(markdown.contains("value"));
	assert!(markdown.contains("**count**"));
	assert!(markdown.contains("3"));
}
