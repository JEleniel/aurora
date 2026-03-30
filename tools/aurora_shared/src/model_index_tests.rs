use super::{CardRef, ModelIndex};
use serde_json::Value;
use serde_json::json;
use std::path::Path;
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn open_builds_index_from_existing_cards() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let model_home = seed_model_home(temp.path())?;
	write_card(
		&model_home.join("MIS-001-Root.json"),
		json!({
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Aurora Mission",
			"links": [{"target": "ACT-001", "relationship": "tracks"}],
			"attributes": {"priority": "high"}
		}),
	)?;
	write_card(
		&model_home
			.join("MIS-001")
			.join("Activity")
			.join("ACT-001-Alpha.json"),
		json!({
			"id": "ACT-001",
			"card_type": "Activity",
			"card_subtype": "Workflow",
			"name": "Alpha Workflow",
			"links": [],
			"attributes": {"owner": "ops"}
		}),
	)?;

	let index = ModelIndex::open(&model_home)?;
	let results = index.search("alpha")?;
	assert_eq!(
		results.first().map(|card| card.id.as_str()),
		Some("ACT-001")
	);
	assert_eq!(
		results
			.first()
			.and_then(|card| card.card_subtype.as_deref()),
		Some("Workflow")
	);
	Ok(())
}

#[test]
fn watcher_indexes_new_card_files() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let model_home = seed_model_home(temp.path())?;
	write_card(
		&model_home.join("MIS-001-Root.json"),
		json!({
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Aurora Mission",
			"links": [],
			"attributes": {}
		}),
	)?;

	let index = ModelIndex::open(&model_home)?;
	assert!(index.search("observer")?.is_empty());

	write_card(
		&model_home
			.join("MIS-001")
			.join("Activity")
			.join("ACT-002-Observer.json"),
		json!({
			"id": "ACT-002",
			"card_type": "Activity",
			"name": "Observer Activity",
			"links": [{"target": "MIS-001", "relationship": "observes"}],
			"attributes": {"channel": "alerts"}
		}),
	)?;

	let result = wait_for_search(&index, "observer", |results| !results.is_empty())?;
	assert_eq!(result[0].id, "ACT-002");
	Ok(())
}

#[test]
fn find_cards_linking_to_returns_exact_inbound_neighbors() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let model_home = seed_model_home(temp.path())?;
	write_card(
		&model_home.join("MIS-001-Root.json"),
		json!({
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Aurora Mission",
			"links": [
				{"target": "ACT-001", "relationship": "tracks"},
				{"target": "ACT-002", "relationship": "tracks"}
			],
			"attributes": {}
		}),
	)?;
	write_card(
		&model_home.join("MIS-001").join("ACT-001.json"),
		json!({
			"id": "ACT-001",
			"card_type": "Activity",
			"name": "Focus",
			"links": [],
			"attributes": {}
		}),
	)?;
	write_card(
		&model_home.join("MIS-001").join("ACT-002.json"),
		json!({
			"id": "ACT-002",
			"card_type": "Activity",
			"name": "Peer",
			"links": [{"target": "ACT-001", "relationship": "relates"}],
			"attributes": {}
		}),
	)?;

	let index = ModelIndex::open(&model_home)?;
	let inbound = index.find_cards_linking_to("ACT-001")?;

	assert_eq!(inbound.len(), 2);
	assert_eq!(inbound[0].id, "ACT-002");
	assert_eq!(inbound[1].id, "MIS-001");
	Ok(())
}

#[test]
fn watcher_removes_deleted_cards() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let model_home = seed_model_home(temp.path())?;
	let root_path = model_home.join("MIS-001-Root.json");
	let child_path = model_home
		.join("MIS-001")
		.join("Activity")
		.join("ACT-003-Delete.json");
	write_card(
		&root_path,
		json!({
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Aurora Mission",
			"links": [{"target": "ACT-003", "relationship": "contains"}],
			"attributes": {}
		}),
	)?;
	write_card(
		&child_path,
		json!({
			"id": "ACT-003",
			"card_type": "Activity",
			"name": "Delete Me",
			"links": [],
			"attributes": {}
		}),
	)?;

	let index = ModelIndex::open(&model_home)?;
	assert_eq!(
		index.search("delete")?.first().map(|card| card.id.as_str()),
		Some("ACT-003")
	);

	std::fs::remove_file(&child_path)?;
	let result = wait_for_search(&index, "delete", |results| results.is_empty())?;
	assert!(result.is_empty());
	Ok(())
}

#[test]
fn search_prefers_name_matches_over_attribute_only_matches() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let model_home = seed_model_home(temp.path())?;
	write_card(
		&model_home.join("MIS-001-Root.json"),
		json!({
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Aurora Mission",
			"links": [{"target": "ACT-010", "relationship": "contains"}, {"target": "ACT-011", "relationship": "contains"}],
			"attributes": {}
		}),
	)?;
	write_card(
		&model_home
			.join("MIS-001")
			.join("Activity")
			.join("ACT-010-Alpha.json"),
		json!({
			"id": "ACT-010",
			"card_type": "Activity",
			"name": "Alpha Control",
			"links": [],
			"attributes": {"owner": "ops"}
		}),
	)?;
	write_card(
		&model_home
			.join("MIS-001")
			.join("Activity")
			.join("ACT-011-Beta.json"),
		json!({
			"id": "ACT-011",
			"card_type": "Activity",
			"name": "Beta Control",
			"links": [],
			"attributes": {"alpha": true}
		}),
	)?;

	let index = ModelIndex::open(&model_home)?;
	let results = index.search("alpha")?;
	assert!(results.len() >= 2);
	assert_eq!(results[0].id, "ACT-010");
	assert!(results.iter().any(|card| card.id == "ACT-011"));
	Ok(())
}

#[test]
fn resolve_card_path_returns_relative_path_for_indexed_card() -> Result<()> {
	let temp = tempfile::tempdir()?;
	let model_home = seed_model_home(temp.path())?;
	write_card(
		&model_home.join("MIS-001-Root.json"),
		json!({
			"id": "MIS-001",
			"card_type": "Mission",
			"name": "Aurora Mission",
			"links": [],
			"attributes": {}
		}),
	)?;
	write_card(
		&model_home.join("MIS-001").join("ACT-012-Path.json"),
		json!({
			"id": "ACT-012",
			"card_type": "Activity",
			"name": "Path Lookup",
			"links": [],
			"attributes": {}
		}),
	)?;

	let index = ModelIndex::open(&model_home)?;
	assert_eq!(
		index.resolve_card_path("ACT-012")?,
		Some(Path::new("MIS-001/ACT-012-Path.json").to_path_buf())
	);
	Ok(())
}

fn seed_model_home(root: &Path) -> Result<std::path::PathBuf> {
	let model_home = root.join("aurora");
	std::fs::create_dir_all(model_home.join("MIS-001").join("Activity"))?;
	Ok(model_home)
}

fn write_card(path: &Path, value: Value) -> Result<()> {
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}
	std::fs::write(path, serde_json::to_string_pretty(&value)?)?;
	Ok(())
}

fn wait_for_search(
	index: &ModelIndex,
	query: &str,
	predicate: impl Fn(&[CardRef]) -> bool,
) -> Result<Vec<CardRef>> {
	let deadline = Instant::now() + Duration::from_secs(3);
	loop {
		let results = index.search(query)?;
		if predicate(&results) {
			return Ok(results);
		}
		if Instant::now() >= deadline {
			return Err(format!("timed out waiting for search results for query '{query}'").into());
		}
		std::thread::sleep(Duration::from_millis(50));
	}
}
