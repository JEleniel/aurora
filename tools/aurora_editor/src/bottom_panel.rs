//! Bottom-panel UI for audit-log and diagnostics tabs.

use std::sync::Arc;

use dioxus::prelude::*;

use crate::EditorSession;
use crate::bottom_panel_model::{
	AuditEntrySummary, BottomPanelState, BottomPanelTab, DiagnosticMessage, load_bottom_panel_state,
};
use crate::theme::ThemePalette;

#[component]
pub(crate) fn BottomPanelTabs(
	session: Option<Arc<EditorSession>>,
	selected_card_id: Signal<String>,
	palette: ThemePalette,
) -> Element {
	let mut active_tab = use_signal(|| BottomPanelTab::AuditLog);
	let state = load_bottom_panel_state(session.as_deref(), selected_card_id().as_str());

	rsx! {
		div { style: container_style(),
			div { style: header_style(&palette),
				div { style: "display: grid; gap: 4px;",
					h2 { style: "margin: 0;", "Bottom panel" }
					p { style: muted_text_style(&palette),
						"Inspect the selected card's audit trail or switch to model diagnostics."
					}
				}
				div {
					role: "tablist",
					aria_label: "Bottom panel views",
					style: tablist_style(),
					for tab in [BottomPanelTab::AuditLog, BottomPanelTab::Diagnostics] {
						button {
							key: "bottom-tab-{tab.label()}",
							role: "tab",
							aria_selected: (active_tab() == tab).to_string(),
							style: tab_button_style(active_tab() == tab, &palette),
							onclick: move |_| active_tab.set(tab),
							"{tab.label()}"
						}
					}
				}
			}
			BottomPanelBody {
				state,
				active_tab: active_tab(),
				selected_card_id: selected_card_id(),
				on_select: move |card_id| selected_card_id.set(card_id),
				palette,
			}
		}
	}
}

#[component]
fn BottomPanelBody(
	state: BottomPanelState,
	active_tab: BottomPanelTab,
	selected_card_id: String,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	match state {
		BottomPanelState::Unavailable(message)
		| BottomPanelState::Missing(message)
		| BottomPanelState::Error(message) => rsx! {
			p { role: "alert", style: alert_style(&palette), "{message}" }
		},
		BottomPanelState::EmptySelection => rsx! {
			p { style: muted_text_style(&palette),
				"Select a card from the graph or sidebar to load its audit history and model diagnostics."
			}
		},
		BottomPanelState::Loaded(model) => {
			let model = *model;
			rsx! {
				div { style: panel_body_style(),
					p { style: muted_text_style(&palette), "Model root: {model.root_card_id}" }
					if active_tab == BottomPanelTab::AuditLog {
						AuditLogTab {
							entries: model.audit_entries,
							selected_card_id,
							on_select,
							palette,
						}
					} else {
						DiagnosticsTab {
							errors: model.validation_errors,
							warnings: model.validation_warnings,
							on_select,
							palette,
						}
					}
				}
			}
		}
	}
}

#[component]
fn AuditLogTab(
	entries: Vec<AuditEntrySummary>,
	selected_card_id: String,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	if entries.is_empty() {
		return rsx! {
			p { style: muted_text_style(&palette),
				"No audit-log entries mention {selected_card_id} yet."
			}
		};
	}

	rsx! {
		section { style: section_style(),
			h3 { style: "margin: 0;", "Reverse-chronological entries" }
			ul { style: list_style(),
				for (index , entry) in entries.into_iter().enumerate() {
					li {
						key: "audit-entry-{index}",
						style: item_style(&palette),
						div { style: "display: grid; gap: 4px;",
							strong { "{entry.change_summary}" }
							span { style: muted_text_style(&palette),
								"{entry.timestamp_label} · {entry.editor}"
							}
						}
						button {
							style: action_button_style(&palette),
							onclick: {
								let card_id = entry.navigation_target.clone();
								move |_| on_select.call(card_id.clone())
							},
							"Jump to card"
						}
					}
				}
			}
		}
	}
}

#[component]
fn DiagnosticsTab(
	errors: Vec<DiagnosticMessage>,
	warnings: Vec<DiagnosticMessage>,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		div { style: diagnostics_grid_style(),
			DiagnosticSection {
				title: "Validation errors",
				empty_message: "No validation errors for the current model state.",
				messages: errors,
				on_select: move |card_id| on_select.call(card_id),
				palette,
			}
			DiagnosticSection {
				title: "Validation warnings",
				empty_message: "No validation warnings for the current model state.",
				messages: warnings,
				on_select: move |card_id| on_select.call(card_id),
				palette,
			}
		}
	}
}

#[component]
fn DiagnosticSection(
	title: &'static str,
	empty_message: &'static str,
	messages: Vec<DiagnosticMessage>,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		section { style: section_style(),
			h3 { style: "margin: 0;", "{title}" }
			if messages.is_empty() {
				p { style: muted_text_style(&palette), "{empty_message}" }
			} else {
				ul { style: list_style(),
					for (index , message) in messages.into_iter().enumerate() {
						li {
							key: "diagnostic-{title}-{index}",
							style: item_style(&palette),
							p { style: message_text_style(&palette), "{message.message}" }
							button {
								style: action_button_style(&palette),
								onclick: {
									let card_id = message.navigation_target.clone();
									move |_| on_select.call(card_id.clone())
								},
								"Jump to {message.navigation_target}"
							}
						}
					}
				}
			}
		}
	}
}

fn container_style() -> &'static str {
	"display: grid; gap: 16px; min-height: 100%; align-content: start;"
}

fn header_style(palette: &ThemePalette) -> String {
	format!(
		"display: flex; justify-content: space-between; gap: 12px; align-items: start; flex-wrap: wrap; padding-bottom: 12px; border-bottom: 1px solid {};",
		palette.border,
	)
}

fn tablist_style() -> &'static str {
	"display: flex; gap: 10px; flex-wrap: wrap;"
}

fn tab_button_style(selected: bool, palette: &ThemePalette) -> String {
	if selected {
		return format!(
			"background: {}; color: {}; border-color: {};",
			palette.accent, palette.button_foreground, palette.accent,
		);
	}
	format!(
		"background: {}; color: {}; border-color: {};",
		palette.button_background, palette.button_foreground, palette.border,
	)
}

fn panel_body_style() -> &'static str {
	"display: grid; gap: 14px; align-content: start;"
}

fn section_style() -> &'static str {
	"display: grid; gap: 10px;"
}

fn list_style() -> &'static str {
	"display: grid; gap: 10px; margin: 0; padding-left: 0; list-style: none;"
}

fn item_style(palette: &ThemePalette) -> String {
	format!(
		"display: flex; justify-content: space-between; gap: 12px; align-items: center; flex-wrap: wrap; padding: 12px; border-radius: 10px; border: 1px solid {}; background: {};",
		palette.border, palette.background,
	)
}

fn action_button_style(palette: &ThemePalette) -> String {
	format!(
		"background: {}; color: {}; border-color: {};",
		palette.button_background, palette.button_foreground, palette.border,
	)
}

fn diagnostics_grid_style() -> &'static str {
	"display: grid; gap: 16px; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); align-items: start;"
}

fn alert_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; padding: 12px 14px; border-radius: 10px; background: {}; color: {};",
		palette.error_background, palette.error_foreground,
	)
}

fn muted_text_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; line-height: 1.5; color: {};",
		palette.muted_foreground
	)
}

fn message_text_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; line-height: 1.5; color: {};",
		palette.foreground
	)
}
