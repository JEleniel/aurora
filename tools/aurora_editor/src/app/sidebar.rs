//! Sidebar navigation rendering.

use dioxus::prelude::*;

use crate::state::AppState;
use crate::{model::CardSummary, state::ValidationSummary};

use super::state_sync::{load_model_from_path, select_card_and_sync, sync_draft_with_selection};
use super::types::EditorDraft;

/// Renders the navigation sidebar with model controls and search.
pub(crate) fn render_sidebar(
	state: Signal<AppState>,
	snapshot: &AppState,
	draft: Signal<EditorDraft>,
	validation: Option<ValidationSummary>,
) -> Element {
	let selected_id = snapshot.selected_card_id.clone().unwrap_or_default();
	let search_query = snapshot.search_query.clone();
	let current_mission = snapshot.current_mission();
	let entries = current_mission.map(|mission| mission.model.tree_entries());
	let search_results = current_mission.map(|mission| mission.model.search_cards(&search_query));
	let card_count = state
		.read()
		.current_mission()
		.map(|mission| mission.model.tree_entries().len())
		.unwrap_or(0);

	let on_load_default = {
		let mut state = state.clone();
		let draft = draft.clone();
		move |_| {
			let path = {
				let mut state = state.write();
				state.load_default();
				state.load_path.clone()
			};
			load_model_from_path(state.clone(), draft.clone(), path);
		}
	};

	let on_load_path = {
		let state = state.clone();
		let draft = draft.clone();
		move |_| {
			let path = state.read().load_path.clone();
			load_model_from_path(state.clone(), draft.clone(), path);
		}
	};

	let on_path_input = {
		let mut state = state.clone();
		move |event: Event<FormData>| {
			state.write().load_path = event.value();
		}
	};

	let on_search_input = {
		let mut state = state.clone();
		move |event: Event<FormData>| {
			state.write().search_query = event.value();
		}
	};

	let on_select_mission = {
		let mut state = state.clone();
		let draft = draft.clone();
		move |event: Event<FormData>| {
			if let Ok(index) = event.value().parse::<usize>() {
				{
					let mut state = state.write();
					state.select_mission(index);
				}
				let current_state = state.read();
				sync_draft_with_selection(&current_state, draft.clone());
			}
		}
	};

	rsx! {
		div { class: "panel nav-panel",
			div { class: "status-pill", "Cards: {card_count}" }
			if let Some(summary) = validation {
				div { class: "status-pill", "Errors: {summary.errors}" }
				div { class: "status-pill", "Warnings: {summary.warnings}" }
				div { class: "status-pill", "Info: {summary.infos}" }
			}

			h2 { "Navigation" }
			div { class: "control-row",
				input {
					r#type: "text",
					value: snapshot.load_path.clone(),
					placeholder: "Model home path",
					oninput: on_path_input,
				}
				button { class: "primary", onclick: on_load_path, "Load" }
				button { onclick: on_load_default, "Load Default" }
			}
			if snapshot.missions.len() > 1 {
				div { class: "control-row",
					span { class: "field-label", "Model" }
					select { onchange: on_select_mission,
						for (index , mission) in snapshot.missions.iter().enumerate() {
							option { value: index.to_string(),
								"{mission.mission_id} — {mission.mission_name}"
							}
						}
					}
				}
			}
			div { class: "control-row",
				input {
					r#type: "text",
					value: search_query.clone(),
					placeholder: "Search cards",
					oninput: on_search_input,
				}
			}
			if let Some(error) = snapshot.last_error.as_ref() {
				div { class: "error-banner", "{error}" }
			}
			if !search_query.trim().is_empty() {
				if let Some(results) = search_results {
					div { class: "inline-muted", "Search results" }
					for entry in results {
						{render_sidebar_entry(&entry, 0, &selected_id, state.clone(), draft.clone())}
					}
				}
			} else if let Some(entries) = entries {
				for entry in entries {
					{
						render_sidebar_entry(
							&entry.summary,
							entry.depth,
							&selected_id,
							state.clone(),
							draft.clone(),
						)
					}
				}
			}
		}
	}
}

fn render_sidebar_entry(
	entry: &CardSummary,
	depth: usize,
	selected_id: &str,
	state: Signal<AppState>,
	draft: Signal<EditorDraft>,
) -> Element {
	let is_selected = entry.id == selected_id;
	let indent = format!("padding-left: 0.4rem");
	let gutter_style = if depth == 0 {
		"width: 0; border-left: 0;".to_string()
	} else {
		format!("width: {}rem", depth as f32 * 0.6)
	};
	let branch_style = if depth == 0 {
		"width: 0; border-bottom: 0;".to_string()
	} else {
		"width: 0.6rem".to_string()
	};
	let class_name = if is_selected {
		"tree-entry selected"
	} else {
		"tree-entry"
	};
	let card_id = entry.id.clone();
	let state = state.clone();
	let draft = draft.clone();

	rsx! {
		div {
			class: "{class_name}",
			style: "{indent}",
			onclick: move |_| select_card_and_sync(state.clone(), draft.clone(), card_id.clone()),
			span { class: "tree-gutter", style: "{gutter_style}" }
			span { class: "tree-branch", style: "{branch_style}" }
			span { "{entry.id}" }
			span { class: "inline-muted", "{entry.card_type}" }
			span { "{entry.name}" }
		}
	}
}
