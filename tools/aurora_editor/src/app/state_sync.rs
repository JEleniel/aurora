//! State synchronization helpers for selection and loading.

use std::path::PathBuf;

use dioxus::prelude::*;

use crate::state::AppState;

use super::types::EditorDraft;

/// Syncs the current selection into the editor draft.
pub(crate) fn sync_draft_with_selection(state: &AppState, mut draft: Signal<EditorDraft>) {
	let selected_card = state
		.selected_card_id
		.as_deref()
		.and_then(|card_id| state.current_mission().map(|mission| (card_id, mission)))
		.and_then(|(card_id, mission)| mission.model.card(card_id));
	if let Some(card) = selected_card {
		*draft.write() = EditorDraft::from_card(card);
	}
}

/// Selects a card and updates the editor draft with its fields.
pub(crate) fn select_card_and_sync(
	mut state: Signal<AppState>,
	draft: Signal<EditorDraft>,
	card_id: String,
) {
	{
		let mut state = state.write();
		state.select_card(card_id);
	}
	let current_state = state.read();
	sync_draft_with_selection(&current_state, draft);
}

/// Loads the model from the provided path and updates the draft state.
pub(crate) fn load_model_from_path(
	mut state: Signal<AppState>,
	draft: Signal<EditorDraft>,
	path: String,
) {
	let result = {
		let mut state = state.write();
		let path = PathBuf::from(path);
		state.load_from_path(&path)
	};
	match result {
		Ok(()) => {
			let current_state = state.read();
			sync_draft_with_selection(&current_state, draft);
		}
		Err(error) => {
			state.write().set_error(error);
		}
	}
}
