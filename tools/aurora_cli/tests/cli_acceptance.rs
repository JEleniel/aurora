use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::Compression;
use flate2::write::GzEncoder;

fn write_model_home_scaffold(model_home: &Path) {
	let schemas = model_home.join("schemas");
	let reference = model_home.join("reference");
	fs::create_dir_all(&schemas).expect("create schemas dir");
	fs::create_dir_all(&reference).expect("create reference dir");

	fs::write(
		schemas.join("Aurora.card.schema.json"),
		include_str!("../../../.github/agents/aurora/schemas/Aurora.card.schema.json"),
	)
	.expect("write card schema");
	fs::write(
		schemas.join("Aurora.compact.schema.json"),
		include_str!("../../../.github/agents/aurora/schemas/Aurora.compact.schema.json"),
	)
	.expect("write compact schema");
	fs::write(
		schemas.join("Aurora.audit.schema.json"),
		include_str!("../../../.github/agents/aurora/schemas/Aurora.audit.schema.json"),
	)
	.expect("write audit schema");
	fs::write(
		schemas.join("Aurora.modelconfiguration.schema.json"),
		include_str!(
			"../../../.github/agents/aurora/schemas/Aurora.modelconfiguration.schema.json"
		),
	)
	.expect("write modelconfiguration schema");
	fs::write(
		reference.join("Aurora.modelconfiguration.json"),
		include_str!("../../../.github/agents/aurora/reference/Aurora.modelconfiguration.json"),
	)
	.expect("write modelconfiguration reference");
	write_svg_template_svgz(&reference);
}

fn write_svg_template_svgz(reference_dir: &Path) {
	let svg_template = include_str!("../../../assets/masters/SVGTemplate.svg");
	let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
	encoder
		.write_all(svg_template.as_bytes())
		.expect("gzip svg template");
	let data = encoder.finish().expect("finish gzip");
	fs::write(reference_dir.join("SVGTemplate.svgz"), data).expect("write svgz template");
}

fn write_card(path: &Path, value: serde_json::Value) {
	let data = serde_json::to_string_pretty(&value).expect("serialize card");
	fs::create_dir_all(path.parent().expect("card parent")).expect("create card parent");
	fs::write(path, data).expect("write card");
}

fn write_empty_audit_log(mission_home: &Path) {
	fs::create_dir_all(mission_home).expect("create mission home");
	fs::write(mission_home.join("AuditLog.ndjson"), "").expect("write audit log");
}

fn run_cli(args: &[&str]) -> std::process::Output {
	Command::new(env!("CARGO_BIN_EXE_aurora_cli"))
		.args(args)
		.output()
		.expect("run aurora_cli")
}

#[test]
fn validate_warning_only_returns_success() {
	let temp = tempfile::tempdir().expect("tempdir");
	let model_home = temp.path().join("aurora");
	fs::create_dir_all(&model_home).expect("create model home");
	write_model_home_scaffold(&model_home);

	write_card(
		&model_home.join("MIS-001-Example.json"),
		serde_json::json!({
			"$schema": "./schemas/Aurora.card.schema.json",
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Mission",
			"description": "mission",
			"attributes": {},
			"links": [{"relationship": "rel", "target": "CUS-001"}]
		}),
	);
	write_card(
		&model_home
			.join("MIS-001")
			.join("Custom_Thing")
			.join("CUS-001-Custom.json"),
		serde_json::json!({
			"$schema": "../../schemas/Aurora.card.schema.json",
			"id": "CUS-001",
			"card_type": "Custom Thing",
			"name": "Custom",
			"description": "custom",
			"attributes": {},
			"links": []
		}),
	);
	write_empty_audit_log(&model_home.join("MIS-001"));

	let output = run_cli(&[
		"--input",
		model_home.to_str().expect("model home path"),
		"--log",
		"warn",
		"validate",
	]);
	assert!(
		output.status.success(),
		"expected success for warning-only validate. stderr:\n{}",
		String::from_utf8_lossy(&output.stderr)
	);
}

#[test]
fn render_views_fails_on_root_cycle_validation() {
	let temp = tempfile::tempdir().expect("tempdir");
	let model_home = temp.path().join("aurora");
	let output_dir = temp.path().join("out");
	fs::create_dir_all(&model_home).expect("create model home");
	write_model_home_scaffold(&model_home);

	write_card(
		&model_home.join("MIS-001-Example.json"),
		serde_json::json!({
			"$schema": "./schemas/Aurora.card.schema.json",
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Mission",
			"description": "mission",
			"attributes": {},
			"links": [{"relationship": "contains", "target": "CAP-001"}]
		}),
	);

	let cap_dir = model_home.join("MIS-001").join("Capability");
	write_card(
		&cap_dir.join("CAP-001-One.json"),
		serde_json::json!({
			"$schema": "../../schemas/Aurora.card.schema.json",
			"id": "CAP-001",
			"card_type": "Capability",
			"name": "Cap 1",
			"description": "cap",
			"attributes": {},
			"links": [{"relationship": "depends on", "target": "CAP-002"}]
		}),
	);
	write_card(
		&cap_dir.join("CAP-002-Two.json"),
		serde_json::json!({
			"$schema": "../../schemas/Aurora.card.schema.json",
			"id": "CAP-002",
			"card_type": "Capability",
			"name": "Cap 2",
			"description": "cap",
			"attributes": {},
			"links": [{"relationship": "depends on", "target": "CAP-001"}]
		}),
	);
	write_empty_audit_log(&model_home.join("MIS-001"));

	let output = run_cli(&[
		"--input",
		model_home.to_str().expect("model home path"),
		"--log",
		"error",
		"render-views",
		"--output",
		output_dir.to_str().expect("output path"),
	]);
	assert!(
		!output.status.success(),
		"expected render-views failure for root-cycle model"
	);
}

#[test]
fn compact_requires_valid_model() {
	let temp = tempfile::tempdir().expect("tempdir");
	let model_home = temp.path().join("aurora");
	let output_dir = temp.path().join("out");
	fs::create_dir_all(&model_home).expect("create model home");
	write_model_home_scaffold(&model_home);

	write_card(
		&model_home.join("MIS-001-Example.json"),
		serde_json::json!({
			"$schema": "./schemas/Aurora.card.schema.json",
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Mission",
			"description": "mission",
			"attributes": {},
			"links": [{"relationship": "contains", "target": "REQ-999"}]
		}),
	);
	write_empty_audit_log(&model_home.join("MIS-001"));

	let output = run_cli(&[
		"--input",
		model_home.to_str().expect("model home path"),
		"--log",
		"error",
		"compact",
		"--output",
		output_dir.to_str().expect("output path"),
	]);
	assert!(
		!output.status.success(),
		"expected compact failure for invalid model"
	);

	let compact_path: PathBuf = output_dir.join("MIS-001").join("Compact.json");
	assert!(
		!compact_path.exists(),
		"compact should not be written when validation fails"
	);
}
