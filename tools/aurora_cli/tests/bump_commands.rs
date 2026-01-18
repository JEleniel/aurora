use std::fs::{read_to_string, write};
use std::process::Command;
use tempfile::tempdir;

fn write_card(path: &std::path::Path, id: &str, version: &str, history_empty: bool) {
	let history = if history_empty {
		"[]"
	} else {
		"[{\"editor\": \"orig\", \"timestamp\": \"2025-01-01T00:00:00.000Z\", \"event\": \"created\"}]"
	};
	let content = format!(
		r#"{{
  "$schema": "../Aurora.schema.json",
  "id": "{}",
  "card_type": "Note",
  "name": "Test Card",
  "audit_trail": {{
    "version": "{}",
    "hash": null,
    "history": {}
  }}
}}"#,
		id, version, history
	);
	write(path, content).expect("write card");
}

fn run_cli(args: &[&str]) -> std::process::Output {
	let bin = format!("{}/target/debug/aurora_cli", env!("CARGO_MANIFEST_DIR"));
	Command::new(bin)
		.args(args)
		.output()
		.expect("failed to run aurora_cli")
}

#[test]
fn bump_patch_creates_history_when_empty() {
	let dir = tempdir().unwrap();
	let file = dir.path().join("card.json");
	write_card(&file, "TEST-001", "0.0.0", true);

	let out = run_cli(&["bump-patch", file.to_str().unwrap()]);
	assert!(
		out.status.success(),
		"cli exited unsuccessfully: {}",
		String::from_utf8_lossy(&out.stderr)
	);

	let s = read_to_string(&file).expect("read back");
	let v: serde_json::Value = serde_json::from_str(&s).expect("parse json");
	let version = v["audit_trail"]["version"].as_str().unwrap();
	assert_eq!(version, "0.0.1");
	let history = v["audit_trail"]["history"].as_array().unwrap();
	assert_eq!(history.len(), 1);
	let event = history[0]["event"].as_str().unwrap();
	assert_eq!(event, "created");
}

#[test]
fn bump_minor_appends_edited_when_history_present() {
	let dir = tempdir().unwrap();
	let file = dir.path().join("card2.json");
	write_card(&file, "TEST-002", "1.2.3", false);

	let out = run_cli(&["bump-minor", file.to_str().unwrap()]);
	assert!(
		out.status.success(),
		"cli exited unsuccessfully: {}",
		String::from_utf8_lossy(&out.stderr)
	);

	let s = read_to_string(&file).expect("read back");
	let v: serde_json::Value = serde_json::from_str(&s).expect("parse json");
	let version = v["audit_trail"]["version"].as_str().unwrap();
	assert_eq!(version, "1.3.0");
	let history = v["audit_trail"]["history"].as_array().unwrap();
	assert!(history.len() >= 2);
	let event = history.last().unwrap()["event"].as_str().unwrap();
	assert_eq!(event, "edited");
}

#[test]
fn bump_with_editor_flag_sets_editor() {
	let dir = tempdir().unwrap();
	let file = dir.path().join("card3.json");
	write_card(&file, "TEST-003", "0.1.0", true);

	let out = run_cli(&["bump-patch", file.to_str().unwrap(), "--editor", "CI Bot"]);
	assert!(
		out.status.success(),
		"cli exited unsuccessfully: {}",
		String::from_utf8_lossy(&out.stderr)
	);

	let s = read_to_string(&file).expect("read back");
	let v: serde_json::Value = serde_json::from_str(&s).expect("parse json");
	let history = v["audit_trail"]["history"].as_array().unwrap();
	let editor = history.last().unwrap()["editor"].as_str().unwrap();
	assert_eq!(editor, "CI Bot");
}
