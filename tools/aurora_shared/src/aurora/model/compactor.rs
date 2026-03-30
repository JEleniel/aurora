use serde_json::Value;

use super::Model;

impl Model {
	pub fn compact(&self, schema_ref: Option<String>) -> Value {
		let mut cards: Vec<Value> = Vec::new();
		cards.push(self.root_card.get_compact());
		for card in &self.cards {
			cards.push(card.get_compact());
		}

		let mut root = serde_json::Map::new();
		if let Some(schema_ref) = schema_ref {
			root.insert("$schema".to_string(), Value::String(schema_ref));
		}
		root.insert("cards".to_string(), Value::Array(cards));
		Value::Object(root)
	}
}
