//! Right-sidebar inspector UI for selected cards.

use std::sync::Arc;

use dioxus::prelude::*;

use crate::EditorSession;
use crate::app::EditorAppBootstrap;
use crate::app_settings::{GeneralSection, IdentitySection, SaveFeedback, SettingsActions};
use crate::inspector_model::{
	CardInspectorModel, CardInspectorState, format_attribute_value, load_card_inspector_state,
};
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
                    on_select: move |card_id| selected_card_id.set(card_id),
                    palette,
                }
            }
            section { style: card_style(&palette),
                h2 { style: heading_style(), "Workspace settings" }
                GeneralSection { draft, palette }
                IdentitySection { draft, palette }
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
            CardInspectorDetails { model: *model, on_select, palette }
        },
	}
}

#[component]
fn CardInspectorDetails(
	model: CardInspectorModel,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	let card = model.card;
	let card_meta = card_meta(
		card.id.as_str(),
		card.card_type.as_str(),
		card.card_subtype.as_deref(),
	);

	rsx! {
        div { style: section_column_style(),
            div { style: "display: grid; gap: 6px;",
                h3 { style: "margin: 0;", "{card.name}" }
                p { style: muted_text_style(&palette), "{card_meta}" }
            }
            FieldGrid {
                pairs: vec![
                    (
                        "Version".to_string(),
                        card.version.clone().unwrap_or_else(|| "—".to_string()),
                    ),
                    ("Status".to_string(), card.status.clone().unwrap_or_else(|| "—".to_string())),
                    (
                        "Boundary".to_string(),
                        card.boundary.clone().unwrap_or_else(|| "—".to_string()),
                    ),
                    ("Icon".to_string(), card.icon.clone().unwrap_or_else(|| "—".to_string())),
                ],
                palette,
            }
            TextBlockSection {
                title: "Description",
                body: card.description.clone(),
                palette,
            }
            if let Some(notes) = card.notes.clone() {
                TextBlockSection { title: "Notes", body: notes, palette }
            }
            ListSection {
                title: "Attributes",
                empty_message: "No attributes defined.",
                palette,
                items: card
                     .attributes
                     .iter()
                     .map(|(key, value)| {
                      rsx! {
                    .iter()
                    .map(|(key, value)| {
                        rsx! {
                            li { key: "attribute-{key}", style: item_card_style(&palette),
                                strong { "{key}" }
                                pre { style: code_block_style(&palette), "{format_attribute_value(value)}" }
                            }
                        }
                    })
                    .collect::<Vec<_>>(),
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
fn FieldGrid(pairs: Vec<(String, String)>, palette: ThemePalette) -> Element {
	rsx! {
        section { style: field_grid_style(),
            for (label , value) in pairs {
                div { key: "field-{label}", style: item_card_style(&palette),
                    strong { "{label}" }
                    span { style: muted_text_style(&palette), "{value}" }
                }
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

fn field_grid_style() -> &'static str {
	"display: grid; gap: 10px; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));"
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
