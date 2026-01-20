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

#[derive(Debug, Serialize)]
pub struct CompactCardBorrowed<'a> {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attributes: Option<&'a Map<String, Value>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub card_subtype: Option<&'a str>,
	pub card_type: &'a str,
	pub description: &'a str,
	pub id: &'a str,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub links: Option<&'a Vec<Link>>,
	pub name: &'a str,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<&'a str>,
}

impl<'a> From<&'a Card> for CompactCardBorrowed<'a> {
	fn from(card: &'a Card) -> Self {
		Self {
			attributes: card.attributes.as_ref(),
			card_subtype: card.card_subtype.as_deref(),
			card_type: &card.card_type,
			description: &card.description,
			id: &card.id,
			links: card.links.as_ref(),
			name: &card.name,
			status: card.status.as_deref(),
		}
	}
}
