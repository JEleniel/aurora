//! Shared types for the editor UI.

use aurora_shared::Card;
use serde_json::Value;

/// Tabs displayed in the inspector pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InspectorTab {
	Preview,
	Audit,
}

/// Logo asset metadata for responsive images.
#[derive(Debug, Clone)]
pub(crate) struct LogoAsset {
	pub(crate) src: String,
	pub(crate) srcset: String,
}

/// Draft state for editing a card.
#[derive(Debug, Clone, Default)]
pub(crate) struct EditorDraft {
	pub(crate) id: String,
	pub(crate) card_type: String,
	pub(crate) card_subtype: String,
	pub(crate) name: String,
	pub(crate) status: String,
	pub(crate) description: String,
	pub(crate) attributes: String,
}

impl EditorDraft {
	/// Builds a draft from the provided card snapshot.
	pub(crate) fn from_card(card: &Card) -> Self {
		Self {
			id: card.id.clone(),
			card_type: card.card_type.clone(),
			card_subtype: card.card_subtype.clone().unwrap_or_default(),
			name: card.name.clone(),
			status: card.status.clone().unwrap_or_default(),
			description: card.description.clone(),
			attributes: format_attributes(&card.attributes),
		}
	}
}

fn format_attributes(attributes: &Value) -> String {
	if attributes.is_null() {
		return String::new();
	}
	match serde_json::to_string_pretty(attributes) {
		Ok(serialized) => serialized,
		Err(_) => String::new(),
	}
}
