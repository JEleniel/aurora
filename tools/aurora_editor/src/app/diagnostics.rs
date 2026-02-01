//! Diagnostics list rendering.

use dioxus::prelude::*;

use aurora_shared::{DiagnosticSeverity, ValidationDiagnostic, ValidationReport};

use crate::state::AppState;

use super::state_sync::select_card_and_sync;
use super::types::EditorDraft;

/// Renders diagnostics for the current mission, optionally filtered by card.
pub(crate) fn render_diagnostics(
	validation: Option<&ValidationReport>,
	focused_id: Option<&str>,
	state: Signal<AppState>,
	draft: Signal<EditorDraft>,
) -> Element {
	let diagnostics = validation
		.map(|report| report.diagnostics.as_slice())
		.unwrap_or(&[]);
	let mut entries = Vec::new();
	for diagnostic in diagnostics {
		if let Some(card_id) = focused_id {
			if diagnostic.card_id.as_deref() != Some(card_id) {
				continue;
			}
		}
		entries.push(diagnostic);
	}

	if entries.is_empty() {
		return rsx! { div { class: "inline-muted", "No diagnostics available." } };
	}

	rsx! {
		div {
			for diagnostic in entries {
				{render_diagnostic_entry(diagnostic, state.clone(), draft.clone())}
			}
		}
	}
}

fn render_diagnostic_entry(
	diagnostic: &ValidationDiagnostic,
	state: Signal<AppState>,
	draft: Signal<EditorDraft>,
) -> Element {
	let (label, class_name) = diagnostic_style(diagnostic.severity);
	let card_id = diagnostic.card_id.clone();
	let state = state.clone();
	let draft = draft.clone();
	let class_value = format!("diagnostic-entry {class_name}");
	let onclick = move |_| {
		if let Some(card_id) = card_id.clone() {
			select_card_and_sync(state.clone(), draft.clone(), card_id);
		}
	};
	rsx! {
		div {
			class: "{class_value}",
			onclick: onclick,
			div { "{label}: {diagnostic.code} — {diagnostic.message}" }
			if let Some(card_id) = diagnostic.card_id.as_ref() {
				div { class: "inline-muted", "Card: {card_id}" }
			}
			if let Some(path) = diagnostic.path.as_ref() {
				div { class: "inline-muted", "Path: {path}" }
			}
		}
	}
}

fn diagnostic_style(severity: DiagnosticSeverity) -> (&'static str, &'static str) {
	match severity {
		DiagnosticSeverity::Error => ("Error", "error"),
		DiagnosticSeverity::Warning => ("Warning", "warning"),
		DiagnosticSeverity::Info => ("Info", "info"),
	}
}
