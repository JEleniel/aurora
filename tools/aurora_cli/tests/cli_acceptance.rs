use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn read_testdata(rel_path: &str) -> String {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("src")
		.join("testdata")
		.join(rel_path);
	fs::read_to_string(&path)
		.unwrap_or_else(|e| panic!("failed to read testdata file {}: {e}", path.display()))
}

fn write_model_home_scaffold(model_home: &Path) {
	let schemas = model_home.join("schemas");
	let reference = model_home.join("reference");
	fs::create_dir_all(&schemas).expect("create schemas dir");
	fs::create_dir_all(&reference).expect("create reference dir");

	for schema_file in [
		"Aurora.card.schema.json",
		"Aurora.compact.schema.json",
		"Aurora.audit.schema.json",
		"Aurora.modelconfiguration.schema.json",
		"Aurora.viewconfiguration.schema.json",
	] {
		let contents = read_testdata(&format!("model_home/schemas/{schema_file}"));
		fs::write(schemas.join(schema_file), contents)
			.unwrap_or_else(|e| panic!("failed to write schema fixture {schema_file}: {e}"));
	}
	for reference_file in [
		"Aurora.modelconfiguration.json",
		"Aurora.viewconfiguration.json",
		"SVGTemplate.svg",
	] {
		let contents = read_testdata(&format!("model_home/reference/{reference_file}"));
		fs::write(reference.join(reference_file), contents)
			.unwrap_or_else(|e| panic!("failed to write reference fixture {reference_file}: {e}"));
	}
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
	fs::write(
		model_home
			.join("reference")
			.join("Aurora.modelconfiguration.json"),
		read_testdata("model_home/reference/Aurora.modelconfiguration.with_views.json"),
	)
	.expect("override modelconfiguration for render-views");
	fs::write(
		model_home
			.join("reference")
			.join("Aurora.viewconfiguration.json"),
		read_testdata("model_home/reference/Aurora.viewconfiguration.with_views.json"),
	)
	.expect("override viewconfiguration for render-views");

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
			"links": [{"relationship": "depends on", "target": "MIS-001"}]
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

#[test]
fn upgrade_migrates_attribute_references_into_external_references() {
	let temp = tempfile::tempdir().expect("tempdir");
	let model_home = temp.path().join("aurora");
	fs::create_dir_all(&model_home).expect("create model home");
	write_model_home_scaffold(&model_home);

	let mission_path = model_home.join("MIS-001-Example.json");
	write_card(
		&mission_path,
		serde_json::json!({
			"$schema": "./schemas/Aurora.card.schema.json",
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Mission",
			"description": "mission",
			"external_references": ["existing"],
			"attributes": {
				"keep": 1,
				"references": ["a.md", "b.md"],
				"external_reference": "https://example.com/spec"
			},
			"links": []
		}),
	);
	write_empty_audit_log(&model_home.join("MIS-001"));

	let output = run_cli(&[
		"--input",
		model_home.to_str().expect("model home path"),
		"--log",
		"error",
		"upgrade",
	]);
	assert!(
		output.status.success(),
		"upgrade should succeed. stderr:\n{}",
		String::from_utf8_lossy(&output.stderr)
	);

	let upgraded: serde_json::Value =
		serde_json::from_str(&fs::read_to_string(&mission_path).expect("read mission"))
			.expect("parse upgraded mission");
	let external = upgraded
		.get("external_references")
		.expect("external_references")
		.as_array()
		.expect("external_references array");
	assert_eq!(
		external,
		&vec![
			serde_json::Value::String("existing".to_string()),
			serde_json::Value::String("a.md".to_string()),
			serde_json::Value::String("b.md".to_string()),
			serde_json::Value::String("https://example.com/spec".to_string()),
		]
	);
	let attributes = upgraded
		.get("attributes")
		.expect("attributes")
		.as_object()
		.expect("attributes object");
	assert!(attributes.contains_key("keep"));
	assert!(!attributes.contains_key("references"));
	assert!(!attributes.contains_key("external_reference"));
}

#[test]
fn upgrade_bootstraps_required_assets_and_prunes_outdated_files() {
	let temp = tempfile::tempdir().expect("tempdir");
	let model_home = temp.path().join("aurora");
	let schemas_dir = model_home.join("schemas");
	let reference_dir = model_home.join("reference");
	fs::create_dir_all(&schemas_dir).expect("create schemas dir");
	fs::create_dir_all(&reference_dir).expect("create reference dir");
	fs::write(schemas_dir.join("Legacy.schema.json"), "{}").expect("write outdated schema fixture");
	fs::write(reference_dir.join("Legacy.reference.json"), "{}")
		.expect("write outdated reference fixture");
	fs::write(reference_dir.join("SVGTemplate.svg"), "<svg />")
		.expect("write outdated svg fixture");

	let mission_path = model_home.join("MIS-001-Example.json");
	write_card(
		&mission_path,
		serde_json::json!({
			"$schema": "./schemas/Aurora.card.schema.json",
			"id": "MIS-001", "card_type": "Mission", "name": "Mission", "description": "mission",
			"attributes": {"references": ["a.md"]}, "links": []
		}),
	);
	write_empty_audit_log(&model_home.join("MIS-001"));

	let output = run_cli(&[
		"--input",
		model_home.to_str().expect("model home path"),
		"upgrade",
	]);
	let load_error = aurora_shared::Aurora::try_load(&model_home)
		.err()
		.map(|error| error.to_string())
		.unwrap_or_else(|| "Aurora::try_load succeeded".to_string());
	assert!(
		output.status.success(),
		"upgrade should succeed. stderr:\n{}\npost-upgrade load: {}",
		String::from_utf8_lossy(&output.stderr),
		load_error
	);

	for file in [
		"Aurora.audit.schema.json",
		"Aurora.card.schema.json",
		"Aurora.compact.schema.json",
		"Aurora.modelconfiguration.schema.json",
		"Aurora.viewconfiguration.schema.json",
	] {
		assert!(schemas_dir.join(file).is_file(), "missing schema {}", file);
	}
	for file in [
		"Aurora.modelconfiguration.json",
		"Aurora.viewconfiguration.json",
		"SVGTemplate.svgz",
	] {
		assert!(
			reference_dir.join(file).is_file(),
			"missing reference {}",
			file
		);
	}
	assert!(!schemas_dir.join("Legacy.schema.json").exists());
	assert!(!reference_dir.join("Legacy.reference.json").exists());
	assert!(!reference_dir.join("SVGTemplate.svg").exists());
}

#[test]
fn upgrade_adds_default_domains_when_missing() {
	let temp = tempfile::tempdir().expect("tempdir");
	let model_home = temp.path().join("aurora");
	fs::create_dir_all(&model_home).expect("create model home");
	write_model_home_scaffold(&model_home);
	write_card(
		&model_home.join("MIS-001-Example.json"),
		serde_json::json!({
			"$schema": "./schemas/Aurora.card.schema.json",
			"id": "MIS-001", "card_type": "Mission", "name": "Mission", "description": "mission",
			"attributes": {}, "links": []
		}),
	);
	write_empty_audit_log(&model_home.join("MIS-001"));

	let output = run_cli(&[
		"--input",
		model_home.to_str().expect("model home path"),
		"upgrade",
	]);
	let load_error = aurora_shared::Aurora::try_load(&model_home)
		.err()
		.map(|error| error.to_string())
		.unwrap_or_else(|| "Aurora::try_load succeeded".to_string());
	assert!(
		output.status.success(),
		"upgrade should succeed. stderr:\n{}\npost-upgrade load: {}",
		String::from_utf8_lossy(&output.stderr),
		load_error
	);

	let upgraded: serde_json::Value = serde_json::from_str(
		&fs::read_to_string(
			model_home
				.join("reference")
				.join("Aurora.viewconfiguration.json"),
		)
		.expect("read upgraded view config"),
	)
	.expect("parse upgraded view config");
	assert_eq!(
		upgraded.get("version").and_then(|v| v.as_str()),
		Some("1.1.0")
	);
	assert!(
		upgraded
			.get("domains")
			.and_then(|v| v.as_object())
			.is_some()
	);
}
