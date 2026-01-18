use crate::error::AuroraCliError;
use crate::json_utils::sort_value;
use crate::model::Model;
use crate::output::OutputPaths;
use serde::Serialize;
use serde_json::{Value, json};

/// Export a compact model snapshot that removes audit trails and link relationships.
pub fn export_compact_model(model: &Model, outputs: &OutputPaths) -> Result<(), AuroraCliError> {
	let mut cards: Vec<(String, Value)> = model
		.cards()
		.map(|card| {
			let compact = compact_card(card);
			(card.id.clone(), sort_value(&compact))
		})
		.collect();

	cards.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
	let card_values: Vec<Value> = cards.into_iter().map(|(_, value)| value).collect();
	let payload = json!({ "cards": card_values });
	let sorted_payload = sort_value(&payload);

	let mut buffer = Vec::new();
	let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
	{
		let mut serializer = serde_json::Serializer::with_formatter(&mut buffer, formatter);
		sorted_payload
			.serialize(&mut serializer)
			.map_err(|source| AuroraCliError::Json {
				path: outputs.compact_model_path().to_path_buf(),
				source,
			})?;
	}
	let json_string = String::from_utf8(buffer).expect("serializer produced invalid UTF-8");
	outputs.write(outputs.compact_model_path(), &json_string)
}

fn compact_card(card: &crate::model::Card) -> Value {
	let mut value = card.raw.clone();
	if let Some(obj) = value.as_object_mut() {
		obj.remove("audit_trail");
		if let Some(links) = obj.get_mut("links") {
			if let Some(arr) = links.as_array_mut() {
				for link in arr.iter_mut() {
					if let Some(link_obj) = link.as_object_mut() {
						link_obj.remove("relationship");
					}
				}
			}
		}
	}
	value
}
