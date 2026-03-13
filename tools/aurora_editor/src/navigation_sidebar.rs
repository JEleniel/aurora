//! Left-sidebar navigation, search, and breadcrumb UI.

use std::sync::Arc;

use aurora_shared::CardRef;
use dioxus::prelude::*;

use crate::EditorSession;
use crate::app::SessionSummary;
use crate::navigation_model::{
	ALL_CARD_TYPES_FILTER, card_meta, card_type_options, filter_cards, resolve_card_type_filter,
};
use crate::settings::SettingsDraft;
use crate::shell::ShellLayout;
use crate::theme::ThemePalette;

#[component]
pub(crate) fn NavigationSidebar(
	session: Option<Arc<EditorSession>>,
	selected_card_id: Signal<String>,
	summary: Option<SessionSummary>,
	layout: ShellLayout,
	draft: Signal<SettingsDraft>,
	palette: ThemePalette,
) -> Element {
	let search_query = use_signal(String::new);
	let card_type_filter = use_signal(|| ALL_CARD_TYPES_FILTER.to_string());
	let state = build_navigation_state(
		session.as_deref(),
		search_query().as_str(),
		selected_card_id().as_str(),
	);
	let card_type_options = card_type_options(&state.root_cards, &state.search_results);
	let active_filter = resolve_card_type_filter(card_type_filter().as_str(), &card_type_options);

	use_effect({
		let card_type_options = card_type_options.clone();
		let mut card_type_filter = card_type_filter;
		move || {
			let current = card_type_filter();
			if current != ALL_CARD_TYPES_FILTER
				&& !card_type_options.iter().any(|value| value == &current)
			{
				card_type_filter.set(ALL_CARD_TYPES_FILTER.to_string());
			}
		}
	});

	let filtered_roots = filter_cards(&state.root_cards, active_filter.as_str());
	let filtered_results = filter_cards(&state.search_results, active_filter.as_str());
	let current_card_id = selected_card_id();

	rsx! {
		div { style: container_style(),
			div { style: "display: grid; gap: 16px;",
				h2 { style: "margin: 0;", "Navigation" }
				p { style: body_style(&palette),
					"Search the in-memory model index, filter by card type, and jump the graph to any matching card."
				}
				SearchControls {
					search_query,
					card_type_filter,
					card_type_options,
					palette,
				}
				if let Some(message) = state.search_error {
					p { role: "alert", style: error_text_style(&palette), "{message}" }
				}
				QuickRootsSection {
					cards: filtered_roots,
					selected_card_id: current_card_id.clone(),
					on_select: move |card_id| selected_card_id.set(card_id),
					palette,
				}
				SearchResultsSection {
					query: search_query(),
					cards: filtered_results,
					selected_card_id: current_card_id.clone(),
					on_select: move |card_id| selected_card_id.set(card_id),
					palette,
				}
				SessionDetails {
					summary,
					layout,
					draft,
					palette,
				}
			}
			BreadcrumbSection {
				cards: state.breadcrumb,
				error: state.breadcrumb_error,
				selected_card_id: current_card_id,
				on_select: move |card_id| selected_card_id.set(card_id),
				palette,
			}
		}
	}
}

#[component]
fn SearchControls(
	search_query: Signal<String>,
	card_type_filter: Signal<String>,
	card_type_options: Vec<String>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		div { style: "display: grid; gap: 12px;",
			label { style: label_style(),
				span { "Search cards" }
				input {
					value: search_query(),
					placeholder: "Find cards by ID, type, name, link, or attribute",
					oninput: move |event| search_query.set(event.value()),
					aria_label: "Search cards",
					style: input_style(&palette),
				}
			}
			label { style: label_style(),
				span { "Card type filter" }
				select {
					value: card_type_filter(),
					oninput: move |event| card_type_filter.set(event.value()),
					aria_label: "Filter results by card type",
					style: input_style(&palette),
					option { value: ALL_CARD_TYPES_FILTER, "{ALL_CARD_TYPES_FILTER}" }
					for card_type in card_type_options {
						option { key: "{card_type}", value: "{card_type}", "{card_type}" }
					}
				}
			}
		}
	}
}

#[component]
fn QuickRootsSection(
	cards: Vec<CardRef>,
	selected_card_id: String,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		section { style: "display: grid; gap: 10px;",
			h3 { style: "margin: 0;", "Mission roots" }
			if cards.is_empty() {
				p { style: muted_text_style(&palette), "No root cards match the current filter." }
			} else {
				div { style: list_column_style(),
					for card in cards {
						CardButton {
							key: "root-{card.id}",
							card,
							selected_card_id: selected_card_id.clone(),
							on_select: move |card_id| on_select.call(card_id),
							palette,
						}
					}
				}
			}
		}
	}
}

#[component]
fn SearchResultsSection(
	query: String,
	cards: Vec<CardRef>,
	selected_card_id: String,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	if query.trim().is_empty() {
		return rsx! {
			section { style: "display: grid; gap: 10px;",
				h3 { style: "margin: 0;", "Search results" }
				p { style: muted_text_style(&palette), "Type to search the live model index." }
			}
		};
	}

	rsx! {
		section { style: "display: grid; gap: 10px;",
			h3 { style: "margin: 0;", "Search results" }
			p { style: muted_text_style(&palette), "{cards.len()} matching cards for \"{query}\"." }
			if cards.is_empty() {
				p { style: muted_text_style(&palette), "No cards match the current query and filter." }
			} else {
				div { style: list_column_style(),
					for card in cards {
						CardButton {
							key: "search-{card.id}",
							card,
							selected_card_id: selected_card_id.clone(),
							on_select: move |card_id| on_select.call(card_id),
							palette,
						}
					}
				}
			}
		}
	}
}

#[component]
fn SessionDetails(
	summary: Option<SessionSummary>,
	layout: ShellLayout,
	draft: Signal<SettingsDraft>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		section { style: "display: grid; gap: 8px;",
			h3 { style: "margin: 0;", "Session" }
			if let Some(summary) = summary {
				p { style: muted_text_style(&palette), "Model home: {summary.model_home_display}" }
				p { style: muted_text_style(&palette), "Root missions loaded: {summary.root_count}" }
			}
			p { style: muted_text_style(&palette),
				"Left {layout.left_sidebar_percent()}% · Center {layout.center_column_percent()}% · Right {layout.right_sidebar_percent()}% · Bottom {layout.bottom_panel_percent()}%"
			}
			p { style: muted_text_style(&palette),
				"Theme: {draft().theme_preference.as_str()} · Base font: {draft().base_font_size_px}px"
			}
		}
	}
}

#[component]
fn BreadcrumbSection(
	cards: Vec<CardRef>,
	error: Option<String>,
	selected_card_id: String,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		section { style: breadcrumb_section_style(&palette),
			h3 { style: "margin: 0;", "Breadcrumb" }
			if let Some(message) = error {
				p { role: "alert", style: error_text_style(&palette), "{message}" }
			} else if cards.is_empty() {
				p { style: muted_text_style(&palette),
					"Select a card to trace its path from the mission root."
				}
			} else {
				nav {
					aria_label: "Card breadcrumb",
					style: breadcrumb_nav_style(),
					for (index , card) in cards.into_iter().enumerate() {
						span {
							key: "crumb-{card.id}",
							style: "display: contents;",
							if index > 0 {
								span { style: muted_text_style(&palette), "›" }
							}
							button {
								style: breadcrumb_button_style(selected_card_id.as_str(), card.id.as_str(), &palette),
								onclick: {
									let next_id = card.id.clone();
									move |_| on_select.call(next_id.clone())
								},
								"{card.name}"
							}
						}
					}
				}
			}
		}
	}
}

#[component]
fn CardButton(
	card: CardRef,
	selected_card_id: String,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	let selected = selected_card_id == card.id;
	let meta = card_meta(&card);

	rsx! {
		button {
			style: card_button_style(selected, &palette),
			onclick: {
				let next_id = card.id.clone();
				move |_| on_select.call(next_id.clone())
			},
			aria_label: "Open {card.name}",
			title: "{card.id}",
			strong { style: "text-align: left;", "{card.name}" }
			span { style: muted_text_style(&palette), "{meta}" }
		}
	}
}

struct NavigationState {
	root_cards: Vec<CardRef>,
	search_results: Vec<CardRef>,
	breadcrumb: Vec<CardRef>,
	search_error: Option<String>,
	breadcrumb_error: Option<String>,
}

fn build_navigation_state(
	session: Option<&EditorSession>,
	query: &str,
	selected_card_id: &str,
) -> NavigationState {
	let Some(session) = session else {
		return NavigationState {
			root_cards: Vec::new(),
			search_results: Vec::new(),
			breadcrumb: Vec::new(),
			search_error: Some("Editor session is not available yet.".to_string()),
			breadcrumb_error: None,
		};
	};

	let (root_cards, root_error) = match session.root_cards() {
		Ok(cards) => (cards, None),
		Err(error) => (Vec::new(), Some(error.to_string())),
	};
	let (search_results, search_error) = if query.trim().is_empty() {
		(Vec::new(), root_error)
	} else {
		match session.search(query) {
			Ok(cards) => (cards, root_error),
			Err(error) => (Vec::new(), Some(error.to_string())),
		}
	};
	let (breadcrumb, breadcrumb_error) = if selected_card_id.trim().is_empty() {
		(Vec::new(), None)
	} else {
		match session.breadcrumb(selected_card_id) {
			Ok(cards) => (cards, None),
			Err(error) => (Vec::new(), Some(error.to_string())),
		}
	};

	NavigationState {
		root_cards,
		search_results,
		breadcrumb,
		search_error,
		breadcrumb_error,
	}
}

fn container_style() -> &'static str {
	"display: flex; flex-direction: column; gap: 18px; min-height: 100%;"
}

fn label_style() -> &'static str {
	"display: grid; gap: 6px; font-weight: 600;"
}

fn input_style(palette: &ThemePalette) -> String {
	format!(
		"width: 100%; background: {}; color: {}; border-color: {};",
		palette.input_background, palette.input_foreground, palette.border,
	)
}

fn list_column_style() -> &'static str {
	"display: grid; gap: 8px;"
}

fn body_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; line-height: 1.6; color: {};",
		palette.foreground
	)
}

fn muted_text_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; line-height: 1.5; color: {};",
		palette.muted_foreground
	)
}

fn error_text_style(palette: &ThemePalette) -> String {
	format!("margin: 0; color: {};", palette.error_background)
}

fn card_button_style(selected: bool, palette: &ThemePalette) -> String {
	let (background, border, color) = if selected {
		(palette.accent, palette.accent, palette.button_foreground)
	} else {
		(
			palette.button_background,
			palette.border,
			palette.button_foreground,
		)
	};
	format!(
		"display: grid; gap: 4px; width: 100%; text-align: left; padding: 12px; background: {}; color: {}; border-color: {};",
		background, color, border,
	)
}

fn breadcrumb_section_style(palette: &ThemePalette) -> String {
	format!(
		"display: grid; gap: 10px; margin-top: auto; padding-top: 12px; border-top: 1px solid {};",
		palette.border,
	)
}

fn breadcrumb_nav_style() -> &'static str {
	"display: flex; gap: 8px; flex-wrap: wrap; align-items: center;"
}

fn breadcrumb_button_style(
	selected_card_id: &str,
	card_id: &str,
	palette: &ThemePalette,
) -> String {
	if selected_card_id == card_id {
		return format!(
			"padding: 8px 10px; background: {}; color: {}; border-color: {};",
			palette.accent, palette.button_foreground, palette.accent,
		);
	}
	format!(
		"padding: 8px 10px; background: {}; color: {}; border-color: {};",
		palette.button_background, palette.button_foreground, palette.border,
	)
}
