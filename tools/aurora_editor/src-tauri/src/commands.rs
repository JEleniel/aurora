//! Command handlers exposed to the Aurora Editor webview.

use std::fs;
use std::path::{Path, PathBuf};

use aurora_shared::{
	AuroraModel, Card, ModelHome, discover_model_homes, load_model, render_all, render_markdown,
	render_views, validate_model, write_compact_model,
};
use tauri::State;
use tracing::error;

use crate::audit::{AuditEventKind, apply_audit_event, new_audit_trail};
use crate::state::{EditorState, WorkspaceState};
use crate::types::{
	AuditBump, CardDraft, CardRecord, CompactRequest, CreateCardRequest, DeleteCardRequest,
	ModelHomeInfo, ModelSnapshot, RenderRequest, RenderSummaryDto, UpdateCardRequest,
	WorkspaceInfo,
};
use crate::workspace::{
	canonicalize_dir, resolve_workspace_path, schema_relative_path, workspace_relative,
	write_json_atomic,
};

/// Simple command that allows the UI to verify connectivity with the backend.
#[tauri::command]
pub fn health_check() -> Result<String, String> {
	Ok("ok".to_string())
}

/// Register or update the current workspace root and trust state.
#[tauri::command]
pub fn set_workspace(
	state: State<'_, EditorState>,
	root: String,
	trusted: bool,
) -> Result<WorkspaceInfo, String> {
	let root_path = canonicalize_dir(&root).map_err(|err| log_error("set_workspace", err))?;
	let workspace = WorkspaceState {
		root: root_path,
		trusted,
	};
	let stored = state
		.set_workspace(workspace)
		.map_err(|err| log_error("set_workspace", err))?;
	Ok(WorkspaceInfo {
		root: stored.root.to_string_lossy().to_string(),
		trusted: stored.trusted,
	})
}

/// Return the current workspace configuration.
#[tauri::command]
pub fn workspace_status(state: State<'_, EditorState>) -> Result<WorkspaceInfo, String> {
	let workspace = require_workspace(&state)?;
	Ok(WorkspaceInfo {
		root: workspace.root.to_string_lossy().to_string(),
		trusted: workspace.trusted,
	})
}

/// Discover model homes rooted in the workspace.
#[tauri::command]
pub fn discover_models(state: State<'_, EditorState>) -> Result<Vec<ModelHomeInfo>, String> {
	let workspace = require_workspace(&state)?;
	let homes = discover_model_homes(&workspace.root)
		.map_err(|err| log_error("discover_models", err.to_string()))?;
	let results = homes
		.iter()
		.map(|home| ModelHomeInfo {
			root: home.root().to_string_lossy().to_string(),
			has_schema: home.has_schema(),
		})
		.collect();
	Ok(results)
}

/// Load the specified model home and return a full snapshot.
#[tauri::command]
pub fn load_model_snapshot(
	state: State<'_, EditorState>,
	model_home: String,
) -> Result<ModelSnapshot, String> {
	let workspace = require_workspace(&state)?;
	let model = load_model_from_workspace(&workspace, &model_home)?;
	model_snapshot(&model, &workspace)
}

/// Validate the specified model home and return diagnostics.
#[tauri::command]
pub fn validate_model_snapshot(
	state: State<'_, EditorState>,
	model_home: String,
) -> Result<aurora_shared::ValidationReport, String> {
	let workspace = require_workspace(&state)?;
	let model = load_model_from_workspace(&workspace, &model_home)?;
	Ok(validate_model(&model))
}

/// Render card markdown for the specified model home.
#[tauri::command]
pub fn render_card_markdown(
	state: State<'_, EditorState>,
	request: RenderRequest,
) -> Result<RenderSummaryDto, String> {
	let workspace = require_trusted_workspace(&state, "render_card_markdown")?;
	let model = load_model_from_workspace(&workspace, &request.model_home)?;
	let output_dir = resolve_workspace_path(&workspace.root, &request.output_dir)
		.map_err(|err| log_error("render_card_markdown", err))?;
	let summary = render_markdown(&model, &output_dir)
		.map_err(|err| log_error("render_card_markdown", err.to_string()))?;
	to_render_summary(&workspace, summary)
}

/// Render view assets (DOT, SVG, Markdown) for the specified model home.
#[tauri::command]
pub fn render_views_bundle(
	state: State<'_, EditorState>,
	request: RenderRequest,
) -> Result<RenderSummaryDto, String> {
	let workspace = require_trusted_workspace(&state, "render_views_bundle")?;
	let model = load_model_from_workspace(&workspace, &request.model_home)?;
	let output_dir = resolve_workspace_path(&workspace.root, &request.output_dir)
		.map_err(|err| log_error("render_views_bundle", err))?;
	let summary = render_views(&model, &output_dir)
		.map_err(|err| log_error("render_views_bundle", err.to_string()))?;
	to_render_summary(&workspace, summary)
}

/// Render card markdown and view assets for the specified model home.
#[tauri::command]
pub fn render_all_assets(
	state: State<'_, EditorState>,
	request: RenderRequest,
) -> Result<RenderSummaryDto, String> {
	let workspace = require_trusted_workspace(&state, "render_all_assets")?;
	let model = load_model_from_workspace(&workspace, &request.model_home)?;
	let output_dir = resolve_workspace_path(&workspace.root, &request.output_dir)
		.map_err(|err| log_error("render_all_assets", err))?;
	let summary = render_all(&model, &output_dir)
		.map_err(|err| log_error("render_all_assets", err.to_string()))?;
	to_render_summary(&workspace, summary)
}

/// Write or refresh the compact model export.
#[tauri::command]
pub fn write_compact_export(
	state: State<'_, EditorState>,
	request: CompactRequest,
) -> Result<String, String> {
	let workspace = require_trusted_workspace(&state, "write_compact_export")?;
	let model = load_model_from_workspace(&workspace, &request.model_home)?;
	let output_path = match &request.output_path {
		Some(path) => Some(
			resolve_workspace_path(&workspace.root, path)
				.map_err(|err| log_error("write_compact_export", err))?,
		),
		None => None,
	};
	let path = write_compact_model(&model, output_path)
		.map_err(|err| log_error("write_compact_export", err.to_string()))?;
	workspace_relative(&workspace.root, &path).map_err(|err| log_error("write_compact_export", err))
}

/// Create a new card file within the model home.
#[tauri::command]
pub fn create_card(
	state: State<'_, EditorState>,
	request: CreateCardRequest,
) -> Result<CardRecord, String> {
	let workspace = require_trusted_workspace(&state, "create_card")?;
	ensure_editor(&request.editor, "create_card")?;
	validate_card_fields(&request.card, "create_card")?;
	let model_home = resolve_model_home(&workspace, &request.model_home)?;
	let card_path = resolve_card_path(&workspace, model_home.root(), &request.relative_path)?;
	ensure_card_extension(&card_path, "create_card")?;
	if card_path.exists() {
		return Err(log_error(
			"create_card",
			"Card path already exists; choose a new filename".to_string(),
		));
	}
	let schema = schema_relative_path(model_home.root(), &card_path)
		.map_err(|err| log_error("create_card", err))?;
	let audit_trail = new_audit_trail(&request.editor);
	let card = card_from_draft(&request.card, schema, audit_trail);
	write_card(&card_path, &card, "create_card")?;
	card_record(&workspace, card, &card_path)
}

/// Update an existing card with a new payload.
#[tauri::command]
pub fn update_card(
	state: State<'_, EditorState>,
	request: UpdateCardRequest,
) -> Result<CardRecord, String> {
	let workspace = require_trusted_workspace(&state, "update_card")?;
	ensure_editor(&request.editor, "update_card")?;
	validate_card_fields(&request.card, "update_card")?;
	let model_home = resolve_model_home(&workspace, &request.model_home)?;
	let card_path = resolve_card_path(&workspace, model_home.root(), &request.relative_path)?;
	ensure_card_extension(&card_path, "update_card")?;
	let mut existing = read_card(&card_path, "update_card")?;
	if existing.id != request.card.id {
		return Err(log_error(
			"update_card",
			"Card id changes require creating a new card".to_string(),
		));
	}
	if existing.card_type != request.card.card_type {
		return Err(log_error(
			"update_card",
			"Card type changes require creating a new card".to_string(),
		));
	}
	let schema = schema_relative_path(model_home.root(), &card_path)
		.map_err(|err| log_error("update_card", err))?;
	existing.schema = Some(schema);
	existing.card_subtype = request.card.card_subtype.clone();
	existing.name = request.card.name.clone();
	existing.description = request.card.description.clone();
	existing.status = request.card.status.clone();
	existing.links = request.card.links.clone();
	existing.attributes = request.card.attributes.clone();
	existing.extra = request.card.extra.clone();
	let bump = request.bump.unwrap_or(AuditBump::Patch);
	apply_audit_event(
		&mut existing.audit_trail,
		bump,
		&request.editor,
		AuditEventKind::Edited,
	)
	.map_err(|err| log_error("update_card", err))?;
	write_card(&card_path, &existing, "update_card")?;
	card_record(&workspace, existing, &card_path)
}

/// Mark a card as deleted by updating its status and audit trail.
#[tauri::command]
pub fn delete_card(
	state: State<'_, EditorState>,
	request: DeleteCardRequest,
) -> Result<CardRecord, String> {
	let workspace = require_trusted_workspace(&state, "delete_card")?;
	ensure_editor(&request.editor, "delete_card")?;
	let model_home = resolve_model_home(&workspace, &request.model_home)?;
	let card_path = resolve_card_path(&workspace, model_home.root(), &request.relative_path)?;
	ensure_card_extension(&card_path, "delete_card")?;
	let mut existing = read_card(&card_path, "delete_card")?;
	existing.status = Some("Deleted".to_string());
	let bump = request.bump.unwrap_or(AuditBump::Major);
	apply_audit_event(
		&mut existing.audit_trail,
		bump,
		&request.editor,
		AuditEventKind::Deleted,
	)
	.map_err(|err| log_error("delete_card", err))?;
	write_card(&card_path, &existing, "delete_card")?;
	card_record(&workspace, existing, &card_path)
}

fn require_workspace(state: &EditorState) -> Result<WorkspaceState, String> {
	state
		.workspace()
		.map_err(|err| log_error("workspace", err))?
		.ok_or_else(|| log_error("workspace", "Workspace is not configured".to_string()))
}

fn require_trusted_workspace(state: &EditorState, context: &str) -> Result<WorkspaceState, String> {
	let workspace = require_workspace(state)?;
	if !workspace.trusted {
		return Err(log_error(
			context,
			"Workspace is untrusted; operation is blocked".to_string(),
		));
	}
	Ok(workspace)
}

fn resolve_model_home(workspace: &WorkspaceState, model_home: &str) -> Result<ModelHome, String> {
	let resolved = resolve_workspace_path(&workspace.root, model_home)
		.map_err(|err| log_error("model_home", err))?;
	let home = ModelHome::new(&resolved).map_err(|err| log_error("model_home", err.to_string()))?;
	if !home.root().starts_with(&workspace.root) {
		return Err(log_error(
			"model_home",
			"Model home is outside the workspace root".to_string(),
		));
	}
	if !home.has_schema() {
		return Err(log_error(
			"model_home",
			"Model home is missing Aurora.schema.jsjson or Aurora.schema.json".to_string(),
		));
	}
	Ok(home)
}

fn load_model_from_workspace(
	workspace: &WorkspaceState,
	model_home: &str,
) -> Result<AuroraModel, String> {
	let home = resolve_model_home(workspace, model_home)?;
	load_model(&home).map_err(|err| log_error("load_model", err.to_string()))
}

fn model_snapshot(
	model: &AuroraModel,
	workspace: &WorkspaceState,
) -> Result<ModelSnapshot, String> {
	let home_path = workspace_relative(&workspace.root, model.home().root())
		.map_err(|err| log_error("model_snapshot", err))?;
	let mut records = Vec::new();
	for card in model.iter_cards() {
		let source = card
			.source_path()
			.ok_or_else(|| log_error("model_snapshot", "Card missing source path".to_string()))?;
		let relative = workspace_relative(&workspace.root, source)
			.map_err(|err| log_error("model_snapshot", err))?;
		records.push(CardRecord {
			card: card.clone(),
			source_path: relative,
		});
	}
	Ok(ModelSnapshot {
		home: home_path,
		cards: records,
	})
}

fn to_render_summary(
	workspace: &WorkspaceState,
	summary: aurora_shared::RenderSummary,
) -> Result<RenderSummaryDto, String> {
	let output_dir = workspace_relative(&workspace.root, &summary.output_dir)
		.map_err(|err| log_error("render_summary", err))?;
	Ok(RenderSummaryDto {
		cards_written: summary.cards_written,
		views_written: summary.views_written,
		output_dir,
	})
}

fn card_from_draft(
	draft: &CardDraft,
	schema: String,
	audit_trail: aurora_shared::AuditTrail,
) -> Card {
	Card {
		schema: Some(schema),
		id: draft.id.clone(),
		card_type: draft.card_type.clone(),
		card_subtype: draft.card_subtype.clone(),
		name: draft.name.clone(),
		description: draft.description.clone(),
		status: draft.status.clone(),
		links: draft.links.clone(),
		audit_trail,
		attributes: draft.attributes.clone(),
		source_path: None,
		extra: draft.extra.clone(),
	}
}

fn read_card(path: &Path, context: &str) -> Result<Card, String> {
	let contents = fs::read_to_string(path)
		.map_err(|err| log_error(context, format!("Failed to read {}: {err}", path.display())))?;
	serde_json::from_str(&contents).map_err(|err| {
		log_error(
			context,
			format!("Failed to parse {}: {err}", path.display()),
		)
	})
}

fn write_card(path: &Path, card: &Card, context: &str) -> Result<(), String> {
	let value = serde_json::to_value(card)
		.map_err(|err| log_error(context, format!("Failed to serialize card: {err}")))?;
	write_json_atomic(path, &value).map_err(|err| log_error(context, err))
}

fn resolve_card_path(
	workspace: &WorkspaceState,
	model_home: &Path,
	relative_path: &str,
) -> Result<PathBuf, String> {
	let resolved = resolve_workspace_path(&workspace.root, relative_path)
		.map_err(|err| log_error("card_path", err))?;
	if !resolved.starts_with(model_home) {
		return Err(log_error(
			"card_path",
			"Card path must live within the model home".to_string(),
		));
	}
	Ok(resolved)
}

fn card_record(workspace: &WorkspaceState, card: Card, path: &Path) -> Result<CardRecord, String> {
	let source_path =
		workspace_relative(&workspace.root, path).map_err(|err| log_error("card_record", err))?;
	Ok(CardRecord { card, source_path })
}

fn validate_card_fields(card: &CardDraft, context: &str) -> Result<(), String> {
	ensure_non_empty(&card.id, context, "id")?;
	ensure_non_empty(&card.card_type, context, "card_type")?;
	ensure_non_empty(&card.name, context, "name")?;
	ensure_non_empty(&card.description, context, "description")?;
	Ok(())
}

fn ensure_editor(editor: &str, context: &str) -> Result<(), String> {
	ensure_non_empty(editor, context, "editor")
}

fn ensure_non_empty(value: &str, context: &str, field: &str) -> Result<(), String> {
	if value.trim().is_empty() {
		return Err(log_error(
			context,
			format!("Field '{field}' must not be empty"),
		));
	}
	Ok(())
}

fn ensure_card_extension(path: &Path, context: &str) -> Result<(), String> {
	let ext = path
		.extension()
		.and_then(|value| value.to_str())
		.unwrap_or("?");
	if ext != "json" && ext != "jsjson" {
		return Err(log_error(
			context,
			"Card files must end in .json or .jsjson".to_string(),
		));
	}
	Ok(())
}

fn log_error(context: &str, message: String) -> String {
	error!(context = context, error = %message, "editor backend error");
	message
}
