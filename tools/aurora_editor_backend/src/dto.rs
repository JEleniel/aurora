use std::path::Path;

use aurora_shared::{AuroraModel, Card, Link, ModelHome, ValidationDiagnostic, ValidationReport};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::session::ModelSession;

#[derive(Debug, Serialize)]
pub struct ModelSummary {
	pub mission_id: Option<String>,
	pub mission_name: Option<String>,
	pub card_count: usize,
	pub model_path: String,
}

impl ModelSummary {
	pub fn from_model(home: &ModelHome, model: &AuroraModel) -> Self {
		let mission = find_mission(model);
		ModelSummary {
			mission_id: mission.map(|card| card.id.clone()),
			mission_name: mission.map(|card| card.name.clone()),
			card_count: model.len(),
			model_path: path_string(home.root()),
		}
	}
}

#[derive(Debug, Serialize)]
pub struct LinkView {
	pub target: String,
	pub relationship: String,
}

impl From<&Link> for LinkView {
	fn from(link: &Link) -> Self {
		Self {
			target: link.target.clone(),
			relationship: link.relationship.clone(),
		}
	}
}

#[derive(Debug, Serialize)]
pub struct CardView {
	pub id: String,
	pub card_type: String,
	pub card_subtype: Option<String>,
	pub name: String,
	pub description: String,
	pub status: Option<String>,
	pub links: Vec<LinkView>,
	pub attributes: Value,
	pub source_path: Option<String>,
}

impl From<&Card> for CardView {
	fn from(card: &Card) -> Self {
		Self {
			id: card.id.clone(),
			card_type: card.card_type.clone(),
			card_subtype: card.card_subtype.clone(),
			name: card.name.clone(),
			description: card.description.clone(),
			status: card.status.clone(),
			links: card.links.iter().map(LinkView::from).collect(),
			attributes: card.attributes.clone(),
			source_path: card.source_path().map(path_string),
		}
	}
}

#[derive(Debug, Serialize)]
pub struct LoadModelResponse {
	pub summary: ModelSummary,
	pub cards: Vec<CardView>,
	pub validation: ValidationPayload,
}

impl LoadModelResponse {
	pub fn build(home: &ModelHome, model: &AuroraModel, validation: &ValidationReport) -> Self {
		Self {
			summary: ModelSummary::from_model(home, model),
			cards: model.iter_cards().map(CardView::from).collect(),
			validation: ValidationPayload::from_report(validation),
		}
	}
}

#[derive(Debug, Serialize)]
pub struct ValidationPayload {
	pub has_errors: bool,
	pub diagnostics: Vec<ValidationMessage>,
}

impl ValidationPayload {
	pub fn from_report(report: &ValidationReport) -> Self {
		let diagnostics = report
			.diagnostics
			.iter()
			.map(ValidationMessage::from)
			.collect::<Vec<_>>();
		let has_errors = diagnostics.iter().any(|diag| diag.severity == "error");
		Self {
			has_errors,
			diagnostics,
		}
	}
}

#[derive(Debug, Serialize)]
pub struct ValidationMessage {
	pub severity: String,
	pub code: String,
	pub message: String,
	pub card_id: Option<String>,
	pub path: Option<String>,
}

impl From<&ValidationDiagnostic> for ValidationMessage {
	fn from(diag: &ValidationDiagnostic) -> Self {
		Self {
			severity: match diag.severity {
				aurora_shared::DiagnosticSeverity::Error => "error",
				aurora_shared::DiagnosticSeverity::Warning => "warning",
				aurora_shared::DiagnosticSeverity::Info => "info",
			}
			.to_string(),
			code: diag.code.to_string(),
			message: diag.message.clone(),
			card_id: diag.card_id.clone(),
			path: diag.path.clone(),
		}
	}
}

#[derive(Debug, Serialize)]
pub struct RenderReport {
	pub output_dir: String,
	pub cards_written: usize,
	pub views_written: usize,
}

#[derive(Debug, Serialize)]
pub struct CompactResult {
	pub output_path: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCardRequest {
	pub id: String,
	pub editor: String,
	pub name: Option<String>,
	pub description: Option<String>,
	pub card_subtype: Option<Option<String>>,
	pub status: Option<Option<String>>,
	pub attributes: Option<Option<Value>>,
	pub links: Option<Vec<LinkInput>>,
}

#[derive(Debug, Deserialize)]
pub struct LinkInput {
	pub target: String,
	pub relationship: String,
}

#[derive(Debug, Serialize)]
pub struct ModelClosedResponse {
	pub message: String,
}

pub fn path_string(path: &Path) -> String {
	path.to_path_buf().display().to_string()
}

fn find_mission(model: &AuroraModel) -> Option<&Card> {
	model.iter_cards().find(|card| card.card_type == "Mission")
}

pub fn card_by_id<'a>(model: &'a AuroraModel, id: &str) -> Option<&'a Card> {
	model.iter_cards().find(|card| card.id == id)
}

pub fn collect_card_views(model: &AuroraModel) -> Vec<CardView> {
	model.iter_cards().map(CardView::from).collect()
}

pub fn render_report(output_dir: &Path, summary: &aurora_shared::RenderSummary) -> RenderReport {
	RenderReport {
		output_dir: path_string(output_dir),
		cards_written: summary.cards_written,
		views_written: summary.views_written,
	}
}

pub fn compact_result(path: &Path) -> CompactResult {
	CompactResult {
		output_path: path_string(path),
	}
}

pub fn load_model_response(home: &ModelHome, model: &AuroraModel) -> LoadModelResponse {
	let validation = aurora_shared::validate_model(model);
	LoadModelResponse::build(home, model, &validation)
}

pub fn validation_payload(model: &AuroraModel) -> ValidationPayload {
	let report = aurora_shared::validate_model(model);
	ValidationPayload::from_report(&report)
}

pub fn summary_from_session(session: &ModelSession) -> LoadModelResponse {
	let validation = aurora_shared::validate_model(session.model());
	LoadModelResponse::build(session.home(), session.model(), &validation)
}
