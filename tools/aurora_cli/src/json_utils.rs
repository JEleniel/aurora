use serde_json::{Map, Value};

/// Recursively sort JSON objects to provide deterministic key ordering.
pub(crate) fn sort_value(value: &Value) -> Value {
	match value {
		Value::Object(map) => {
			let mut sorted = Map::new();
			let mut keys: Vec<_> = map.keys().cloned().collect();
			keys.sort();
			for key in keys {
				if let Some(val) = map.get(&key) {
					sorted.insert(key, sort_value(val));
				}
			}
			Value::Object(sorted)
		}
		Value::Array(items) => Value::Array(items.iter().map(sort_value).collect()),
		other => other.clone(),
	}
}
