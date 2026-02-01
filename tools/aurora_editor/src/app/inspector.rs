//! Inspector and editor form rendering.

use dioxus::prelude::*;

use aurora_shared::{Card, ValidationReport};

use crate::model::render_card_preview;
use crate::state::AppState;

use super::diagnostics::render_diagnostics;
use super::empty_state::render_empty_state;
use super::types::{EditorDraft, InspectorTab};

/// Renders the inspector pane with editor controls and preview/audit tabs.
pub(crate) fn render_inspector(
	focused_card: Option<Card>,
	focused_id: Option<String>,
	validation: Option<ValidationReport>,
	selected_tab: InspectorTab,
	inspector_tab: Signal<InspectorTab>,
	state: Signal<AppState>,
	draft: Signal<EditorDraft>,
) -> Element {
	let has_card = focused_card.is_some();
	let on_preview_tab = {
		let mut inspector_tab = inspector_tab.clone();
		move |_| {
			*inspector_tab.write() = InspectorTab::Preview;
		}
	};
	let on_audit_tab = {
		let mut inspector_tab = inspector_tab.clone();
		move |_| {
			*inspector_tab.write() = InspectorTab::Audit;
		}
	};

	rsx! {
		section { class: "panel inspector-panel",
			div { class: "inspector-shell",
				div { class: "editor-pane",
					h2 { "Editor" }
					{render_editor(draft, has_card)}
				}
				div { class: "inspector-pane",
					h2 { "Inspector" }
					if let Some(card) = focused_card.as_ref() {
						div { class: "control-row",
							button { class: "primary", onclick: on_preview_tab, "Preview" }
							button { onclick: on_audit_tab, "Audit" }
						}
						if selected_tab == InspectorTab::Preview {
							{render_inspector_preview(card)}
						} else {
							{render_inspector_audit(card)}
						}
					} else {
						{render_empty_state("Select a card to inspect.")}
					}
						h2 { "Diagnostics" }
						{render_diagnostics(validation.as_ref(), focused_id.as_deref(), state, draft.clone())}
				}
			}
		}
	}
}

fn render_editor(draft: Signal<EditorDraft>, has_card: bool) -> Element {
	let draft_state = draft.read().clone();
	let disabled = !has_card;

	let on_name = {
		let mut draft = draft.clone();
		move |event: Event<FormData>| {
			draft.write().name = event.value();
		}
	};
	let on_subtype = {
		let mut draft = draft.clone();
		move |event: Event<FormData>| {
			draft.write().card_subtype = event.value();
		}
	};
	let on_status = {
		let mut draft = draft.clone();
		move |event: Event<FormData>| {
			draft.write().status = event.value();
		}
	};
	let on_description = {
		let mut draft = draft.clone();
		move |event: Event<FormData>| {
			draft.write().description = event.value();
		}
	};
	let on_attributes = {
		let mut draft = draft.clone();
		move |event: Event<FormData>| {
			draft.write().attributes = event.value();
		}
	};

	rsx! {
		div { class: "editor-form",
			if !has_card {
				div { class: "inline-muted", "Select a card to edit." }
			} else {
				div { class: "inline-muted", "Draft mode (saving not yet implemented)." }
				div { class: "editor-field",
					label { "ID" }
					input { r#type: "text", value: draft_state.id, disabled: true }
				}
				div { class: "editor-field",
					label { "Type" }
					input { r#type: "text", value: draft_state.card_type, disabled: true }
				}
				div { class: "editor-field",
					label { "Subtype" }
					input {
						r#type: "text",
						value: draft_state.card_subtype,
						disabled: disabled,
						oninput: on_subtype,
					}
				}
				div { class: "editor-field",
					label { "Name" }
					input {
						r#type: "text",
						value: draft_state.name,
						disabled: disabled,
						oninput: on_name,
					}
				}
				div { class: "editor-field",
					label { "Status" }
					input {
						r#type: "text",
						value: draft_state.status,
						disabled: disabled,
						oninput: on_status,
					}
				}
				div { class: "editor-field",
					label { "Description" }
					textarea {
						value: draft_state.description,
						disabled: disabled,
						oninput: on_description,
					}
				}
				div { class: "editor-field",
					label { "Attributes (JSON)" }
					textarea {
						value: draft_state.attributes,
						disabled: disabled,
						oninput: on_attributes,
					}
				}
				div { class: "control-row",
					button { disabled: true, "Save" }
				}
			}
		}
	}
}

fn render_inspector_preview(card: &Card) -> Element {
	let preview = render_card_preview(card);
	rsx! {
		div { class: "preview-block", "{preview}" }
	}
}

fn render_inspector_audit(card: &Card) -> Element {
	rsx! {
		div { class: "control-row",
			div { "Version: {card.audit_trail.version}" }
			if let Some(hash) = card.audit_trail.hash.as_ref() {
				div { "Hash: {hash}" }
			}
		}
		ul { class: "meta-list",
			for event in &card.audit_trail.history {
				li { "{event.timestamp} — {event.editor} ({event.event})" }
			}
		}
	}
}
