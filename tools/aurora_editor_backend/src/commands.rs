use std::path::{Path, PathBuf};

use aurora_shared::{
	AuroraModel, RenderSummary, discover_model_homes, load_model, render_all, render_markdown,
	render_views, validate_model, write_compact_model,
};
use chrono::Utc;
use serde_json::Value;
use tauri::State;

use crate::app_state::AppState;
use crate::dto::{
	CardView, LoadModelResponse, ModelClosedResponse, ModelSummary, RenderReport,
	UpdateCardRequest, ValidationPayload, compact_result, load_model_response, render_report,
	summary_from_session, validation_payload,
};
use crate::errors::{BackendError, BackendResult};
use crate::file_ops::write_atomic;
use crate::session::ModelSession;

pub type CommandResult<T> = BackendResult<T>;

#[tauri::command]
pub fn discover_models(input_path: Option<String>) -> CommandResult<Vec<ModelSummary>> {
	let target = input_path
		.map(PathBuf::from)
		.unwrap_or_else(|| aurora_shared::ModelHome::default_input());
	let homes = discover_model_homes(&target)?;
	let mut summaries = Vec::new();
	for home in homes {
		let model = load_model(&home)?;
		summaries.push(ModelSummary::from_model(&home, &model));
	}
	Ok(summaries)
}

#[tauri::command]
pub fn open_model(path: String, state: State<AppState>) -> CommandResult<LoadModelResponse> {
	let home = aurora_shared::ModelHome::new(path)?;
	let model = load_model(&home)?;
	let response = load_model_response(&home, &model);
	state.set_session(ModelSession::new(model));
	Ok(response)
}

#[tauri::command]
pub fn refresh_model(state: State<AppState>) -> CommandResult<LoadModelResponse> {
	state.with_session_mut(|session| {
		session.reload()?;
		Ok(summary_from_session(session))
	})
}

#[tauri::command]
pub fn current_model(state: State<AppState>) -> CommandResult<LoadModelResponse> {
	state.with_session(|session| Ok(summary_from_session(session)))
}

#[tauri::command]
pub fn validate_active_model(state: State<AppState>) -> CommandResult<ValidationPayload> {
	state.with_session(|session| Ok(validation_payload(session.model())))
}

#[tauri::command]
pub fn close_model(state: State<AppState>) -> CommandResult<ModelClosedResponse> {
	state.clear();
	Ok(ModelClosedResponse {
		message: "Model session closed".to_string(),
	})
}

#[tauri::command]
pub fn update_card(request: UpdateCardRequest, state: State<AppState>) -> CommandResult<CardView> {
	state.with_session_mut(|session| update_card_impl(session, request))
}

#[tauri::command]
pub fn render_markdown_docs(
	output_dir: String,
	state: State<AppState>,
) -> CommandResult<RenderReport> {
	render_with_mode(output_dir, RenderMode::Markdown, state)
}

#[tauri::command]
pub fn render_views_docs(
	output_dir: String,
	state: State<AppState>,
) -> CommandResult<RenderReport> {
	render_with_mode(output_dir, RenderMode::Views, state)
}

#[tauri::command]
pub fn render_all_docs(output_dir: String, state: State<AppState>) -> CommandResult<RenderReport> {
	render_with_mode(output_dir, RenderMode::All, state)
}

#[tauri::command]
pub fn write_compact_model_file(
	output_path: Option<String>,
	state: State<AppState>,
) -> CommandResult<crate::dto::CompactResult> {
	state.with_session(|session| {
		let path = output_path.map(PathBuf::from);
		let written = write_compact_model(session.model(), path)?;
		Ok(compact_result(&written))
	})
}

fn update_card_impl(
	session: &mut ModelSession,
	request: UpdateCardRequest,
) -> CommandResult<CardView> {
	let existing = session
		.model()
		.get(&request.id)
		.cloned()
		.ok_or_else(|| BackendError::CardNotFound(request.id.clone()))?;
	let source_path = existing
		.source_path()
		.map(|path| path.to_path_buf())
		.ok_or_else(|| BackendError::MissingCardSource(existing.id.clone()))?;
	let mut updated = existing.clone();
	apply_updates(&mut updated, &request);
	persist_card(&source_path, &updated)?;
	session.reload()?;
	let fresh = session
		.model()
		.get(&request.id)
		.cloned()
		.ok_or_else(|| BackendError::CardNotFound(request.id.clone()))?;
	Ok(CardView::from(&fresh))
}

fn apply_updates(card: &mut aurora_shared::Card, request: &UpdateCardRequest) {
	if let Some(name) = &request.name {
		card.name = name.clone();
	}
	if let Some(description) = &request.description {
		card.description = description.clone();
	}
	if let Some(subtype) = &request.card_subtype {
		card.card_subtype = subtype.clone();
	}
	if let Some(status) = &request.status {
		card.status = status.clone();
	}
	if let Some(attributes) = &request.attributes {
		card.attributes = attributes.clone().unwrap_or(Value::Null);
	}
	if let Some(links) = &request.links {
		card.links = links
			.iter()
			.map(|link| aurora_shared::Link {
				target: link.target.clone(),
				relationship: link.relationship.clone(),
			})
			.collect();
	}
	bump_audit(card, &request.editor);
}

fn persist_card(path: &Path, card: &aurora_shared::Card) -> CommandResult<()> {
	let json = serde_json::to_string_pretty(card)?;
	write_atomic(path, &json)
}

fn bump_audit(card: &mut aurora_shared::Card, editor: &str) {
	let mut parts: Vec<u64> = card
		.audit_trail
		.version
		.split('.')
		.filter_map(|part| part.parse::<u64>().ok())
		.collect();
	if parts.len() != 3 {
		parts = vec![1, 0, 0];
	}
	if let Some(last) = parts.last_mut() {
		*last += 1;
	}
	card.audit_trail.version = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
	card.audit_trail.history.push(aurora_shared::AuditEvent {
		editor: editor.to_string(),
		timestamp: Utc::now().to_rfc3339(),
		event: "edited".to_string(),
		hash: None,
	});
}

fn render_with_mode(
	output_dir: String,
	mode: RenderMode,
	state: State<AppState>,
) -> CommandResult<RenderReport> {
	let out = PathBuf::from(output_dir);
	state.with_session(|session| {
		let summary = mode.render(session.model(), &out)?;
		Ok(render_report(&out, &summary))
	})
}

enum RenderMode {
	Markdown,
	Views,
	All,
}

impl RenderMode {
	fn render(&self, model: &AuroraModel, output: &Path) -> CommandResult<RenderSummary> {
		match self {
			RenderMode::Markdown => render_markdown(model, output).map_err(BackendError::from),
			RenderMode::Views => render_views(model, output).map_err(BackendError::from),
			RenderMode::All => render_all(model, output).map_err(BackendError::from),
		}
	}
}
