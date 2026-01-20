use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::aurora::model::{Card, Link};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactCard {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attributes: Option<Map<String, Value>>,
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

impl From<&Card> for CompactCard {
	fn from(card: &Card) -> Self {
		Self {
			attributes: card.attributes.clone(),
			card_subtype: card.card_subtype.clone(),
			card_type: card.card_type.clone(),
			description: card.description.clone(),
			id: card.id.clone(),
			links: card.links.clone(),
			name: card.name.clone(),
			status: card.status.clone(),
		}
	}
}
