use aurora_cli::{
	OutputPaths, export_compact_model, load_model, render_cards, render_views, validate_model,
};
use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

fn load_design_model() -> aurora_cli::Model {
	load_model(Path::new("docs/design/aurora/")).expect("failed to load design model")
}

#[test]
fn validation_passes_for_design_model() {
	let model = load_design_model();
	let outcome = validate_model(&model).expect("validation failed unexpectedly");
	assert!(
		outcome.is_success(),
		"expected validation success for design model"
	);
}

#[test]
fn render_cards_produces_markdown_files() {
	let model = load_design_model();
	let temp = tempdir().expect("temp dir");
	let outputs = OutputPaths::new(temp.path()).expect("outputs");
	let summary = render_cards(&model, &outputs).expect("render cards");
	assert!(summary.cards_written > 0);
	let mission_md = outputs.cards_dir().join("MIS-001.md");
	assert!(mission_md.exists(), "mission markdown missing");
	assert!(outputs.index_readme().exists(), "index readme missing");
}

#[test]
fn render_views_creates_expected_files() {
	let model = load_design_model();
	let temp = tempdir().expect("temp dir");
	let outputs = OutputPaths::new(temp.path()).expect("outputs");
	let summary = render_views(&model, &outputs).expect("render views");
	assert!(summary.rendered > 0);
	let requirements = outputs.views_dir().join("requirements-view.md");
	assert!(requirements.exists(), "requirements view missing");
	let deployment = outputs.views_dir().join("deployment-topology.md");
	assert!(
		!deployment.exists(),
		"deployment view should be skipped for sample model"
	);
}

#[test]
fn compact_export_strips_audit_and_relationship() {
	let model = load_design_model();
	let temp = tempdir().expect("temp dir");
	let outputs = OutputPaths::new(temp.path()).expect("outputs");
	export_compact_model(&model, &outputs).expect("export compact model");
	let compact_path = outputs.compact_model_path();
	assert!(compact_path.exists(), "compact model file missing");
	let contents = std::fs::read_to_string(compact_path).expect("read compact model");
	let doc: Value = serde_json::from_str(&contents).expect("parse compact model");
	let cards = doc
		.get("cards")
		.and_then(|v| v.as_array())
		.expect("cards array missing");
	assert_eq!(cards.len(), model.len(), "card count mismatch");
	for card in cards {
		assert!(
			card.get("audit_trail").is_none(),
			"audit trail should be removed"
		);
		if let Some(links) = card.get("links").and_then(|v| v.as_array()) {
			for link in links {
				assert!(
					link.get("relationship").is_none(),
					"link relationship field should be removed"
				);
			}
		}
	}
}
