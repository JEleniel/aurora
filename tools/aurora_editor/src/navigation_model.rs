//! Pure navigation-sidebar state and filtering helpers.

use std::collections::BTreeSet;

use aurora_shared::CardRef;

pub(crate) const ALL_CARD_TYPES_FILTER: &str = "All card types";

pub(crate) fn card_type_options(root_cards: &[CardRef], search_results: &[CardRef]) -> Vec<String> {
	root_cards
		.iter()
		.chain(search_results.iter())
		.map(|card| card.card_type.clone())
		.collect::<BTreeSet<_>>()
		.into_iter()
		.collect()
}

pub(crate) fn resolve_card_type_filter(current: &str, options: &[String]) -> String {
	if current == ALL_CARD_TYPES_FILTER || options.iter().any(|value| value == current) {
		return current.to_string();
	}
	ALL_CARD_TYPES_FILTER.to_string()
}

pub(crate) fn filter_cards(cards: &[CardRef], card_type_filter: &str) -> Vec<CardRef> {
	if card_type_filter == ALL_CARD_TYPES_FILTER {
		return cards.to_vec();
	}
	cards
		.iter()
		.filter(|card| card.card_type == card_type_filter)
		.cloned()
		.collect()
}

pub(crate) fn card_meta(card: &CardRef) -> String {
	match card.card_subtype.as_deref() {
		Some(card_subtype) if !card_subtype.is_empty() => {
			format!("{} ({}) · {}", card.card_type, card_subtype, card.id)
		}
		_ => format!("{} · {}", card.card_type, card.id),
	}
}

#[cfg(test)]
mod tests {
	use super::{
		ALL_CARD_TYPES_FILTER, card_meta, card_type_options, filter_cards, resolve_card_type_filter,
	};
	use aurora_shared::CardRef;

	fn card(id: &str, card_type: &str, card_subtype: Option<&str>, name: &str) -> CardRef {
		CardRef {
			id: id.to_string(),
			card_type: card_type.to_string(),
			card_subtype: card_subtype.map(ToString::to_string),
			name: name.to_string(),
		}
	}

	#[test]
	fn card_type_options_are_unique_and_sorted() {
		let options = card_type_options(
			&[card("MIS-001", "Mission", None, "Mission")],
			&[
				card("ACT-001", "Activity", None, "Alpha"),
				card("ACT-002", "Activity", None, "Beta"),
				card("CAP-001", "Capability", None, "Gamma"),
			],
		);

		assert_eq!(options, vec!["Activity", "Capability", "Mission"]);
	}

	#[test]
	fn filtering_keeps_rank_order_within_selected_type() {
		let filtered = filter_cards(
			&[
				card("ACT-001", "Activity", None, "Alpha"),
				card("CAP-001", "Capability", None, "Gamma"),
				card("ACT-002", "Activity", None, "Beta"),
			],
			"Activity",
		);

		assert_eq!(
			filtered
				.iter()
				.map(|card| card.id.as_str())
				.collect::<Vec<_>>(),
			vec!["ACT-001", "ACT-002"]
		);
	}

	#[test]
	fn invalid_card_type_filter_falls_back_to_all() {
		let resolved =
			resolve_card_type_filter("Unknown", &["Activity".to_string(), "Mission".to_string()]);
		assert_eq!(resolved, ALL_CARD_TYPES_FILTER);
	}

	#[test]
	fn card_meta_includes_subtype_when_present() {
		assert_eq!(
			card_meta(&card("ACT-001", "Activity", Some("Workflow"), "Alpha")),
			"Activity (Workflow) · ACT-001"
		);
	}
}
