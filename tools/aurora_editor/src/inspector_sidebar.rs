//! Right-sidebar inspector UI for selected cards.

use std::sync::Arc;

use dioxus::prelude::*;

use crate::EditorSession;
use crate::app::EditorAppBootstrap;
use crate::app_settings::{
	GeneralSection, IdentitySection, ProviderSection, SaveFeedback, SettingsActions,
};
use crate::inspector_model::{
	CardInspectorModel, CardInspectorState, format_attribute_value, load_card_inspector_state,
};
use crate::secrets::AgentProvider;
use crate::settings::SettingsDraft;
use crate::theme::ThemePalette;

#[component]
pub(crate) fn InspectorSidebar(
	bootstrap: EditorAppBootstrap,
	session: Option<Arc<EditorSession>>,
	selected_card_id: Signal<String>,
	draft: Signal<SettingsDraft>,
	baseline: Signal<SettingsDraft>,
	status: Signal<Option<SaveFeedback>>,
	palette: ThemePalette,
) -> Element {
	let refresh_nonce = use_signal(|| 0_u64);
	let state = load_card_inspector_state(session.as_deref(), selected_card_id().as_str());

	rsx! {
		div { style: container_style(),
			section { style: card_style(&palette),
				h2 { style: heading_style(), "Inspector" }
				p { style: muted_text_style(&palette),
					"Task 16 has started with a read-only card detail view. Inline editing lands next; for now this panel surfaces the live model state and keeps existing settings controls close at hand."
				}
				InspectorCardSection {
					state,
					session: session.clone(),
					refresh_nonce,
					on_select: move |card_id| selected_card_id.set(card_id),
					palette,
				}
			}
			section { style: card_style(&palette),
				h2 { style: heading_style(), "Workspace settings" }
				GeneralSection { draft, palette }
				IdentitySection { draft, palette }
				for provider in AgentProvider::all() {
					ProviderSection {
						key: "settings-{provider.as_str()}",
						draft,
						provider,
						palette,
					}
				}
				SettingsActions {
					bootstrap,
					draft,
					baseline,
					status,
				}
			}
		}
	}
}

#[component]
fn InspectorCardSection(
	state: CardInspectorState,
	session: Option<Arc<EditorSession>>,
	refresh_nonce: Signal<u64>,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	match state {
		CardInspectorState::Unavailable(message)
		| CardInspectorState::Missing(message)
		| CardInspectorState::Error(message) => rsx! {
			p { role: "alert", style: alert_style(&palette), "{message}" }
		},
		CardInspectorState::EmptySelection => rsx! {
			p { style: muted_text_style(&palette),
				"Select a card from the graph or navigation sidebar to inspect its current details."
			}
		},
		CardInspectorState::Loaded(model) => rsx! {
			CardInspectorDetails {
				key: "{model.card.id}-{refresh_nonce()}",
				model: *model,
				session,
				refresh_nonce,
				on_select,
				palette,
			}
		},
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CardEditDraft {
	name: String,
	description: String,
	version: String,
	status: String,
	boundary: String,
	notes: String,
	icon: String,
}

impl CardEditDraft {
	fn from_card(card: &aurora_shared::Card) -> Self {
		Self {
			name: card.name.clone(),
			description: card.description.clone(),
			version: card.version.clone().unwrap_or_default(),
			status: card.status.clone().unwrap_or_default(),
			boundary: card.boundary.clone().unwrap_or_default(),
			notes: card.notes.clone().unwrap_or_default(),
			icon: card.icon.clone().unwrap_or_default(),
		}
	}

	fn apply_to_card(&self, card: &mut aurora_shared::Card) {
		card.name = self.name.trim().to_string();
		card.description = self.description.trim().to_string();
		card.version = normalize_optional_text(self.version.as_str());
		card.status = normalize_optional_text(self.status.as_str());
		card.boundary = normalize_optional_text(self.boundary.as_str());
		card.notes = normalize_optional_text(self.notes.as_str());
		card.icon = normalize_optional_text(self.icon.as_str());
	}
}

fn normalize_optional_text(value: &str) -> Option<String> {
	let trimmed = value.trim();
	if trimmed.is_empty() {
		None
	} else {
		Some(trimmed.to_string())
	}
}

fn validate_card_draft(draft: &CardEditDraft) -> Option<String> {
	let mut problems = Vec::new();
	if draft.name.trim().is_empty() {
		problems.push("Name is required.");
	}
	if draft.description.trim().is_empty() {
		problems.push("Description is required.");
	}
	if problems.is_empty() {
		None
	} else {
		Some(problems.join(" "))
	}
}

#[component]
fn CardInspectorDetails(
	model: CardInspectorModel,
	session: Option<Arc<EditorSession>>,
	refresh_nonce: Signal<u64>,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	let card = model.card;
	let draft = use_signal(|| CardEditDraft::from_card(&card));
	let save_error = use_signal(|| Option::<String>::None);
	let card_meta = card_meta(
		card.id.as_str(),
		card.card_type.as_str(),
		card.card_subtype.as_deref(),
	);
	let attribute_items = card
		.attributes
		.iter()
		.map(|(key, value)| {
			rsx! {
				li { key: "attribute-{key}", style: item_card_style(&palette),
					strong { "{key}" }
					pre { style: code_block_style(&palette), "{format_attribute_value(value)}" }
				}
			}
		})
		.collect::<Vec<_>>();
	let on_save = {
		let session = session.clone();
		let mut save_error = save_error;
		let mut refresh_nonce = refresh_nonce;
		let card = card.clone();
		move |_| {
			let Some(session) = session.as_ref() else {
				save_error.set(Some("Editor session is unavailable.".to_string()));
				return;
			};
			let next = draft();
			if let Some(message) = validate_card_draft(&next) {
				save_error.set(Some(message));
				return;
			}
			let mut updated_card = card.clone();
			next.apply_to_card(&mut updated_card);
			match session.save_card(&updated_card) {
				Ok(()) => {
					save_error.set(None);
					refresh_nonce.set(refresh_nonce() + 1);
				}
				Err(error) => save_error.set(Some(error.to_string())),
			}
		}
	};

	rsx! {
		div { style: section_column_style(),
			div { style: "display: grid; gap: 6px;",
				h3 { style: "margin: 0;", "{card.name}" }
				p { style: muted_text_style(&palette), "{card_meta}" }
			}
			if let Some(message) = save_error() {
				p { role: "alert", style: alert_style(&palette), "{message}" }
			}
			EditableCardFields { draft, on_save, palette }
			ListSection {
				title: "Attributes",
				empty_message: "No attributes defined.",
				palette,
				items: attribute_items,
			}
			LinkSection {
				title: "Outbound links",
				empty_message: "No outbound links defined.",
				palette,
				items: model
					.outbound_links
					.iter()
					.map(|link| {
						rsx! {
							li {
								key: "outbound-{link.relationship}-{link.target_id}",
								style: item_card_style(&palette),
								button {
									style: relation_button_style(&palette),
									onclick: {
										let target_id = link.target_id.clone();
										move |_| on_select.call(target_id.clone())
									},
									"{link.target_name.as_deref().unwrap_or(link.target_id.as_str())}"
								}
								span { style: muted_text_style(&palette), "{link.relationship} → {link.target_id}" }
							}
						}
					})
					.collect::<Vec<_>>(),
			}
			LinkSection {
				title: "Linked from",
				empty_message: "No inbound links found.",
				palette,
				items: model
					.inbound_links
					.iter()
					.map(|card_ref| {
						rsx! {
							li { key: "inbound-{card_ref.id}", style: item_card_style(&palette),
								button {
									style: relation_button_style(&palette),
									onclick: {
										let card_id = card_ref.id.clone();
										move |_| on_select.call(card_id.clone())
									},
									"{card_ref.name}"
								}
								span { style: muted_text_style(&palette), "{card_ref.card_type} · {card_ref.id}" }
							}
						}
					})
					.collect::<Vec<_>>(),
			}
			StringListSection {
				title: "External references",
				empty_message: "No external references defined.",
				items: card.external_references.clone(),
				palette,
			}
			StringListSection {
				title: "Validation errors",
				empty_message: "No validation errors.",
				items: card.validation_errors.clone(),
				palette,
			}
			StringListSection {
				title: "Validation warnings",
				empty_message: "No validation warnings.",
				items: card.validation_warnings.clone(),
				palette,
			}
			StringListSection {
				title: "Registry warnings",
				empty_message: "No registry warnings.",
				items: model.registry_warnings,
				palette,
			}
			TextBlockSection {
				title: "Source path",
				body: card.source_path.display().to_string(),
				palette,
			}
		}
	}
}

#[component]
fn EditableCardFields(
	draft: Signal<CardEditDraft>,
	on_save: EventHandler<MouseEvent>,
	palette: ThemePalette,
) -> Element {
	let current = draft();
	rsx! {
		section { style: section_column_style(),
			h4 { style: subheading_style(), "Editable fields" }
			div { style: editable_grid_style(),
				label { style: field_label_style(),
					"Name"
					input {
						value: current.name.clone(),
						oninput: move |evt| update_draft(draft, |state| state.name = evt.value()),
						style: input_style(&palette),
					}
				}
				label { style: field_label_style(),
					"Version"
					input {
						value: current.version.clone(),
						oninput: move |evt| update_draft(draft, |state| state.version = evt.value()),
						style: input_style(&palette),
					}
				}
				label { style: field_label_style(),
					"Status"
					input {
						value: current.status.clone(),
						oninput: move |evt| update_draft(draft, |state| state.status = evt.value()),
						style: input_style(&palette),
					}
				}
				label { style: field_label_style(),
					"Boundary"
					input {
						value: current.boundary.clone(),
						oninput: move |evt| update_draft(draft, |state| state.boundary = evt.value()),
						style: input_style(&palette),
					}
				}
				label { style: field_label_style(),
					"Icon"
					input {
						value: current.icon.clone(),
						oninput: move |evt| update_draft(draft, |state| state.icon = evt.value()),
						style: input_style(&palette),
					}
				}
				label { style: field_label_style(),
					"Description"
					textarea {
						value: current.description.clone(),
						rows: "4",
						oninput: move |evt| update_draft(draft, |state| state.description = evt.value()),
						style: textarea_style(&palette),
					}
				}
				label { style: field_label_style(),
					"Notes"
					textarea {
						value: current.notes.clone(),
						rows: "4",
						oninput: move |evt| update_draft(draft, |state| state.notes = evt.value()),
						style: textarea_style(&palette),
					}
				}
			}
			div { style: action_row_style(),
				button { onclick: move |event| on_save.call(event), "Save changes" }
			}
		}
	}
}

#[component]
fn TextBlockSection(title: &'static str, body: String, palette: ThemePalette) -> Element {
	rsx! {
		section { style: section_column_style(),
			h4 { style: subheading_style(), "{title}" }
			p { style: text_block_style(&palette), "{body}" }
		}
	}
}

#[component]
fn StringListSection(
	title: &'static str,
	empty_message: &'static str,
	items: Vec<String>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		section { style: section_column_style(),
			h4 { style: subheading_style(), "{title}" }
			if items.is_empty() {
				p { style: muted_text_style(&palette), "{empty_message}" }
			} else {
				ul { style: list_style(),
					for (index , item) in items.into_iter().enumerate() {
						li {
							key: "string-item-{title}-{index}",
							style: item_card_style(&palette),
							"{item}"
						}
					}
				}
			}
		}
	}
}

#[component]
fn ListSection(
	title: &'static str,
	empty_message: &'static str,
	items: Vec<Element>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		section { style: section_column_style(),
			h4 { style: subheading_style(), "{title}" }
			if items.is_empty() {
				p { style: muted_text_style(&palette), "{empty_message}" }
			} else {
				ul { style: list_style(),
					for item in items {
						{item}
					}
				}
			}
		}
	}
}

#[component]
fn LinkSection(
	title: &'static str,
	empty_message: &'static str,
	items: Vec<Element>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		ListSection {
			title,
			empty_message,
			items,
			palette,
		}
	}
}

fn card_meta(card_id: &str, card_type: &str, card_subtype: Option<&str>) -> String {
	match card_subtype {
		Some(card_subtype) => format!("{card_type} · {card_subtype} · {card_id}"),
		None => format!("{card_type} · {card_id}"),
	}
}

fn container_style() -> &'static str {
	"display: grid; gap: 16px;"
}

fn card_style(palette: &ThemePalette) -> String {
	format!(
		"display: grid; gap: 16px; padding: 16px; border: 1px solid {}; border-radius: 12px; background: {};",
		palette.border, palette.panel_background,
	)
}

fn section_column_style() -> &'static str {
	"display: grid; gap: 10px;"
}

fn editable_grid_style() -> &'static str {
	"display: grid; gap: 10px; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));"
}

fn field_label_style() -> &'static str {
	"display: grid; gap: 6px; font-weight: 600;"
}

fn input_style(palette: &ThemePalette) -> String {
	format!(
		"width: 100%; background: {}; color: {}; border-color: {};",
		palette.input_background, palette.input_foreground, palette.border
	)
}

fn textarea_style(palette: &ThemePalette) -> String {
	format!(
		"width: 100%; min-height: 96px; resize: vertical; background: {}; color: {}; border-color: {};",
		palette.input_background, palette.input_foreground, palette.border
	)
}

fn action_row_style() -> &'static str {
	"display: flex; gap: 8px; flex-wrap: wrap;"
}

fn update_draft(mut draft: Signal<CardEditDraft>, update: impl FnOnce(&mut CardEditDraft)) {
	let mut next = draft();
	update(&mut next);
	draft.set(next);
}

fn list_style() -> &'static str {
	"display: grid; gap: 8px; padding-left: 18px; margin: 0;"
}

fn item_card_style(palette: &ThemePalette) -> String {
	format!(
		"display: grid; gap: 6px; padding: 10px 12px; border: 1px solid {}; border-radius: 10px; background: {};",
		palette.border, palette.background,
	)
}

fn alert_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; padding: 12px 14px; border-radius: 10px; background: {}; color: {};",
		palette.error_background, palette.error_foreground,
	)
}

fn heading_style() -> &'static str {
	"margin: 0;"
}

fn subheading_style() -> &'static str {
	"margin: 0;"
}

fn relation_button_style(palette: &ThemePalette) -> String {
	format!(
		"justify-self: start; background: {}; color: {}; border-color: {};",
		palette.button_background, palette.button_foreground, palette.border,
	)
}

fn code_block_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; white-space: pre-wrap; word-break: break-word; font-family: ui-monospace, SFMono-Regular, SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace; color: {};",
		palette.foreground,
	)
}

fn text_block_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; white-space: pre-wrap; line-height: 1.6; color: {};",
		palette.foreground,
	)
}

fn muted_text_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; line-height: 1.5; color: {};",
		palette.muted_foreground
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::BTreeMap;
	use std::path::PathBuf;

	fn sample_card() -> aurora_shared::Card {
		aurora_shared::Card {
			schema: None,
			id: "ACT-001".to_string(),
			card_type: "Activity".to_string(),
			card_subtype: Some("Flow".to_string()),
			name: "Alpha Workflow".to_string(),
			description: "loaded lazily".to_string(),
			version: Some("1.0".to_string()),
			status: Some("draft".to_string()),
			boundary: Some("system".to_string()),
			notes: Some("notes".to_string()),
			icon: Some("rocket".to_string()),
			attributes: BTreeMap::new(),
			external_references: Vec::new(),
			links: Vec::new(),
			source_path: PathBuf::from("/model/ACT-001.json"),
			validation_errors: Vec::new(),
			validation_warnings: Vec::new(),
		}
	}

	#[test]
	fn draft_round_trips_scalar_fields() {
		let card = sample_card();
		let draft = CardEditDraft::from_card(&card);
		assert_eq!(draft.name, card.name);
		assert_eq!(draft.description, card.description);

		let mut updated = card.clone();
		CardEditDraft {
			name: " Updated Workflow ".to_string(),
			description: " Updated description ".to_string(),
			version: "2.0".to_string(),
			status: String::new(),
			boundary: String::new(),
			notes: String::new(),
			icon: " icon-star ".to_string(),
		}
		.apply_to_card(&mut updated);

		assert_eq!(updated.name, "Updated Workflow");
		assert_eq!(updated.description, "Updated description");
		assert_eq!(updated.version.as_deref(), Some("2.0"));
		assert_eq!(updated.status, None);
		assert_eq!(updated.boundary, None);
		assert_eq!(updated.notes, None);
		assert_eq!(updated.icon.as_deref(), Some("icon-star"));
	}

	#[test]
	fn draft_validation_rejects_blank_required_fields() {
		let draft = CardEditDraft {
			name: "".to_string(),
			description: "".to_string(),
			version: String::new(),
			status: String::new(),
			boundary: String::new(),
			notes: String::new(),
			icon: String::new(),
		};

		let message = validate_card_draft(&draft).expect("expected validation errors");
		assert!(message.contains("Name is required."));
		assert!(message.contains("Description is required."));
	}
}
