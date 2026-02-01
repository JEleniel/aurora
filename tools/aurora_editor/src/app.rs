//! Dioxus UI for the Aurora standalone editor.

mod assets;
mod context;
mod diagnostics;
mod empty_state;
mod header;
mod inspector;
mod sidebar;
mod state_sync;
mod types;

use dioxus::prelude::*;

use crate::constants::{APP_CSS, RESET_CSS};
use crate::state::AppState;

use self::context::render_context;
use self::header::render_header;
use self::inspector::render_inspector;
use self::sidebar::render_sidebar;
use self::types::{EditorDraft, InspectorTab};

#[component]
/// Root component for the editor UI.
pub fn App() -> Element {
	let state = use_signal(AppState::new);
	let inspector_tab = use_signal(|| InspectorTab::Preview);
	let draft = use_signal(EditorDraft::default);

	let current_state = state.read().clone();
	let current_mission = current_state.current_mission();
	let validation = current_state.validation_summary();
	let focused_card = current_state
		.selected_card_id
		.as_deref()
		.and_then(|card_id| current_mission.map(|mission| (card_id, mission)))
		.and_then(|(card_id, mission)| mission.model.card(card_id))
		.cloned();
	let validation_report = current_mission.map(|mission| mission.validation.clone());

	rsx! {
		style { "{RESET_CSS}" }
		style { "{APP_CSS}" }
		div { class: "app-shell",
			{render_header(&current_state)}
			div { class: "main-grid",
				{render_sidebar(state.clone(), &current_state, draft.clone(), validation)}
				{
					render_context(
						current_mission.map(|mission| mission.model.clone()),
						current_state.selected_card_id.clone(),
						state.clone(),
						draft.clone(),
					)
				}
				{
					render_inspector(
						focused_card,
						current_state.selected_card_id.clone(),
						validation_report,
						*inspector_tab.read(),
						inspector_tab.clone(),
						state.clone(),
						draft.clone(),
					)
				}
			}
		}
	}
}
