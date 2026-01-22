use std::{
	collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
	fs,
	path::{Path, PathBuf},
};

use aurora_lib::{AuroraCard, AuroraLibError, read_card_from_path};
use serde::Serialize;

const TMP_DIR_NAME: &str = "tmp";

const VIEW_FILTER_DEFINITIONS: &[ViewFilterDefinition] = &[
	ViewFilterDefinition {
		id: "all",
		label: "All Cards",
		description: "Display every card discovered in the model home.",
		root_card_types: &[],
		include_card_types: &[],
	},
	ViewFilterDefinition {
		id: "requirements",
		label: "Requirements",
		description: "Mission roots with drivers, requirements, capabilities, features, constraints, actors, stories, and tests.",
		root_card_types: &["Mission"],
		include_card_types: &[
			"Driver",
			"Requirement",
			"Capability",
			"Feature",
			"Constraint",
			"Actor",
			"Story",
			"Test",
		],
	},
	ViewFilterDefinition {
		id: "component",
		label: "Component",
		description: "Systems, applications, components, interfaces, data stores, artifacts, and tests.",
		root_card_types: &["System", "Application"],
		include_card_types: &["Component", "Artifact", "Data Store", "Interface", "Test"],
	},
	ViewFilterDefinition {
		id: "deployment",
		label: "Deployment",
		description: "Deployments with nodes, node instances, data stores, and hosted components.",
		root_card_types: &["Deployment"],
		include_card_types: &["Node", "Node Instance", "Data Store", "Component"],
	},
	ViewFilterDefinition {
		id: "process",
		label: "Process",
		description: "Processes with their actors, activities, conditions, and events.",
		root_card_types: &["Process"],
		include_card_types: &["Actor", "Activity", "Condition", "Event"],
	},
	ViewFilterDefinition {
		id: "state-machine",
		label: "State Machine",
		description: "State machines along with the states, events, and guard conditions they reference.",
		root_card_types: &["State Machine"],
		include_card_types: &["State", "Condition", "Event"],
	},
	ViewFilterDefinition {
		id: "threat-model",
		label: "Threat Model",
		description: "Threat actors, risks, controls, and impacted assets.",
		root_card_types: &["Actor", "Threat"],
		include_card_types: &["Asset", "Risk", "Control"],
	},
];

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModelHomeSummary {
	pub root: String,
	pub missions: Vec<MissionSummary>,
	pub card_count: usize,
	pub filters: Vec<ViewFilterSummary>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViewFilterSummary {
	pub id: String,
	pub label: String,
	pub description: String,
	pub root_card_types: Vec<String>,
	pub include_card_types: Vec<String>,
	pub card_count: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissionSummary {
	pub id: String,
	pub name: String,
	pub cards: Vec<CardSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSummary {
	pub id: String,
	pub mission_id: String,
	pub card_type: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub card_subtype: Option<String>,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<String>,
	pub relative_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilteredCards {
	pub filter_id: String,
	pub filter_label: String,
	pub card_count: usize,
	pub cards: Vec<CardSummary>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdgeSummary {
	pub source_id: String,
	pub target_id: String,
	pub relationship: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GraphNodeSummary {
	pub id: String,
	pub mission_id: String,
	pub card_type: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub card_subtype: Option<String>,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<String>,
	pub relative_path: String,
	pub distance: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNeighborhood {
	pub filter_id: String,
	pub filter_label: String,
	pub max_depth: usize,
	pub center: GraphNodeSummary,
	pub nodes: Vec<GraphNodeSummary>,
	pub edges: Vec<GraphEdgeSummary>,
}

pub(crate) struct OpenedCard {
	pub absolute_path: PathBuf,
	pub card: AuroraCard,
}

pub(crate) struct LoadedModel {
	root: PathBuf,
	summary: ModelHomeSummary,
	cards_by_id: BTreeMap<String, CardRecord>,
	filters: BTreeMap<String, FilterContext>,
}

impl LoadedModel {
	pub fn root_path(&self) -> &Path {
		&self.root
	}

	pub fn summary(&self) -> &ModelHomeSummary {
		&self.summary
	}

	pub fn filter_cards(&self, filter_id: Option<&str>) -> FilteredCards {
		let filter = self.resolve_filter(filter_id);
		let mut cards = Vec::new();
		for mission in &self.summary.missions {
			for card in &mission.cards {
				if filter.allows(&card.card_type, false) {
					cards.push(card.clone());
				}
			}
		}
		FilteredCards {
			filter_id: filter.summary.id.clone(),
			filter_label: filter.summary.label.clone(),
			card_count: cards.len(),
			cards,
		}
	}

	pub fn graph_neighborhood(
		&self,
		card_id: &str,
		filter_id: Option<&str>,
		generations: usize,
	) -> Result<GraphNeighborhood, AuroraLibError> {
		let filter = self.resolve_filter(filter_id);
		let Some(center) = self.cards_by_id.get(card_id) else {
			return Err(validation_error(format!(
				"card {card_id} is not available in the loaded model",
			)));
		};
		let max_depth = generations.max(1);
		let mut nodes: BTreeMap<String, GraphNodeSummary> = BTreeMap::new();
		let mut edges: BTreeSet<GraphEdgeSummary> = BTreeSet::new();
		self.collect_outgoing(center, filter, max_depth, &mut nodes, &mut edges);
		self.collect_incoming(center, filter, max_depth, &mut nodes, &mut edges);
		let mut node_list: Vec<GraphNodeSummary> = nodes.into_values().collect();
		node_list.sort_by(|a, b| a.distance.cmp(&b.distance).then_with(|| a.id.cmp(&b.id)));
		let edge_list: Vec<GraphEdgeSummary> = edges.into_iter().collect();
		Ok(GraphNeighborhood {
			filter_id: filter.summary.id.clone(),
			filter_label: filter.summary.label.clone(),
			max_depth,
			center: center.as_graph_node(0),
			nodes: node_list,
			edges: edge_list,
		})
	}

	fn collect_outgoing(
		&self,
		start: &CardRecord,
		filter: &FilterContext,
		max_depth: usize,
		nodes: &mut BTreeMap<String, GraphNodeSummary>,
		edges: &mut BTreeSet<GraphEdgeSummary>,
	) {
		let mut queue: VecDeque<(String, usize)> = VecDeque::new();
		let mut best_distance: HashMap<String, usize> = HashMap::new();
		queue.push_back((start.card.id.clone(), 0));
		best_distance.insert(start.card.id.clone(), 0);
		while let Some((current_id, depth)) = queue.pop_front() {
			if depth >= max_depth {
				continue;
			}
			let Some(current) = self.cards_by_id.get(&current_id) else {
				continue;
			};
			for link in &current.outgoing {
				let Some(target) = self.cards_by_id.get(&link.target) else {
					continue;
				};
				if !filter.allows(&target.card.card_type, false) {
					continue;
				}
				let next_depth = depth + 1;
				insert_graph_node(nodes, target, next_depth);
				edges.insert(GraphEdgeSummary {
					source_id: current.card.id.clone(),
					target_id: target.card.id.clone(),
					relationship: link.relationship.clone(),
				});
				let should_visit = match best_distance.get(&target.card.id) {
					Some(existing) if *existing <= next_depth => false,
					_ => true,
				};
				if should_visit {
					best_distance.insert(target.card.id.clone(), next_depth);
					queue.push_back((target.card.id.clone(), next_depth));
				}
			}
		}
	}

	fn collect_incoming(
		&self,
		start: &CardRecord,
		filter: &FilterContext,
		max_depth: usize,
		nodes: &mut BTreeMap<String, GraphNodeSummary>,
		edges: &mut BTreeSet<GraphEdgeSummary>,
	) {
		let mut queue: VecDeque<(String, usize)> = VecDeque::new();
		let mut best_distance: HashMap<String, usize> = HashMap::new();
		queue.push_back((start.card.id.clone(), 0));
		best_distance.insert(start.card.id.clone(), 0);
		while let Some((current_id, depth)) = queue.pop_front() {
			if depth >= max_depth {
				continue;
			}
			let Some(current) = self.cards_by_id.get(&current_id) else {
				continue;
			};
			for link in &current.incoming {
				let Some(source) = self.cards_by_id.get(&link.source) else {
					continue;
				};
				if !filter.allows(&source.card.card_type, false) {
					continue;
				}
				let next_depth = depth + 1;
				insert_graph_node(nodes, source, next_depth);
				edges.insert(GraphEdgeSummary {
					source_id: source.card.id.clone(),
					target_id: current.card.id.clone(),
					relationship: link.relationship.clone(),
				});
				let should_visit = match best_distance.get(&source.card.id) {
					Some(existing) if *existing <= next_depth => false,
					_ => true,
				};
				if should_visit {
					best_distance.insert(source.card.id.clone(), next_depth);
					queue.push_back((source.card.id.clone(), next_depth));
				}
			}
		}
	}

	fn resolve_filter(&self, filter_id: Option<&str>) -> &FilterContext {
		let key = filter_id
			.map(normalize_filter_id)
			.unwrap_or_else(|| "all".to_string());
		self.filters
			.get(&key)
			.or_else(|| self.filters.get("all"))
			.expect("all filter must exist")
	}
}

pub(crate) fn load_model_home(root: impl AsRef<Path>) -> Result<LoadedModel, AuroraLibError> {
	let canonical_root = canonicalize_model_home(root.as_ref())?;
	let mut missions: BTreeMap<String, MissionAccumulator> = BTreeMap::new();
	let mut cards_by_id: BTreeMap<String, CardRecord> = BTreeMap::new();

	index_root_missions(&canonical_root, &mut missions, &mut cards_by_id)?;
	index_mission_directories(&canonical_root, &mut missions, &mut cards_by_id)?;

	if missions.is_empty() {
		return Err(validation_error(
			"model home does not contain any mission cards (REQ-001)",
		));
	}

	hydrate_incoming_links(&mut cards_by_id);
	let filter_contexts = build_filter_contexts(&cards_by_id);
	let filters_for_summary: Vec<ViewFilterSummary> = filter_contexts
		.iter()
		.map(|ctx| ctx.summary.clone())
		.collect();
	let filter_lookup: BTreeMap<String, FilterContext> = filter_contexts
		.into_iter()
		.map(|ctx| (ctx.summary.id.clone(), ctx))
		.collect();

	let mut mission_summaries: Vec<MissionSummary> = missions
		.into_values()
		.map(MissionAccumulator::into_summary)
		.collect();
	mission_summaries.sort_by(|a, b| a.id.cmp(&b.id));
	let card_count = mission_summaries
		.iter()
		.map(|mission| mission.cards.len())
		.sum();

	let summary = ModelHomeSummary {
		root: display_path(&canonical_root),
		missions: mission_summaries,
		card_count,
		filters: filters_for_summary,
	};

	Ok(LoadedModel {
		root: canonical_root,
		summary,
		cards_by_id,
		filters: filter_lookup,
	})
}

pub(crate) fn canonicalize_model_home(path: &Path) -> Result<PathBuf, AuroraLibError> {
	if !path.exists() {
		return Err(validation_error(format!(
			"model home {} does not exist",
			path.display()
		)));
	}

	let canonical = path
		.canonicalize()
		.map_err(|err| io_error(path.to_path_buf(), err))?;
	if !canonical.is_dir() {
		return Err(validation_error(format!(
			"model home must be a directory: {}",
			canonical.display()
		)));
	}
	Ok(canonical)
}

fn index_root_missions(
	root: &Path,
	missions: &mut BTreeMap<String, MissionAccumulator>,
	cards_by_id: &mut BTreeMap<String, CardRecord>,
) -> Result<(), AuroraLibError> {
	let entries = fs::read_dir(root).map_err(|err| io_error(root.to_path_buf(), err))?;
	for entry in entries {
		let entry = entry.map_err(|err| io_error(root.to_path_buf(), err))?;
		let path = entry.path();
		let Ok(name) = entry.file_name().into_string() else {
			continue;
		};
		let file_type = entry
			.file_type()
			.map_err(|err| io_error(path.clone(), err))?;
		if file_type.is_file()
			&& path.extension().is_some_and(|ext| ext == "json")
			&& looks_like_mission_card(&name)
		{
			let card = read_card_with_context(&path)?;
			let path_info = relative_path_info(root, &path)?;
			register_card(missions, cards_by_id, card, path_info, None)?;
		}
	}
	Ok(())
}

fn index_mission_directories(
	root: &Path,
	missions: &mut BTreeMap<String, MissionAccumulator>,
	cards_by_id: &mut BTreeMap<String, CardRecord>,
) -> Result<(), AuroraLibError> {
	let entries = fs::read_dir(root).map_err(|err| io_error(root.to_path_buf(), err))?;
	for entry in entries {
		let entry = entry.map_err(|err| io_error(root.to_path_buf(), err))?;
		let path = entry.path();
		let Ok(name) = entry.file_name().into_string() else {
			continue;
		};
		let file_type = entry
			.file_type()
			.map_err(|err| io_error(path.clone(), err))?;
		if file_type.is_symlink() {
			continue;
		}
		if file_type.is_dir() {
			reject_tmp_directory(&name, &path)?;
			if name.starts_with("MIS-") {
				scan_card_tree(root, &path, &name, missions, cards_by_id)?;
			}
		}
	}
	Ok(())
}

fn scan_card_tree(
	root: &Path,
	dir: &Path,
	mission_id: &str,
	missions: &mut BTreeMap<String, MissionAccumulator>,
	cards_by_id: &mut BTreeMap<String, CardRecord>,
) -> Result<(), AuroraLibError> {
	let entries = fs::read_dir(dir).map_err(|err| io_error(dir.to_path_buf(), err))?;
	for entry in entries {
		let entry = entry.map_err(|err| io_error(dir.to_path_buf(), err))?;
		let path = entry.path();
		let file_type = entry
			.file_type()
			.map_err(|err| io_error(path.clone(), err))?;
		if file_type.is_symlink() {
			continue;
		}
		if file_type.is_dir() {
			let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
				continue;
			};
			reject_tmp_directory(name, &path)?;
			scan_card_tree(root, &path, mission_id, missions, cards_by_id)?;
		} else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "json") {
			let card = read_card_with_context(&path)?;
			let path_info = relative_path_info(root, &path)?;
			register_card(
				missions,
				cards_by_id,
				card,
				path_info,
				Some(mission_id.to_string()),
			)?;
		}
	}
	Ok(())
}

fn register_card(
	missions: &mut BTreeMap<String, MissionAccumulator>,
	cards_by_id: &mut BTreeMap<String, CardRecord>,
	card: AuroraCard,
	path_info: RelativePathInfo,
	mission_hint: Option<String>,
) -> Result<(), AuroraLibError> {
	let mission_id = derive_mission_id(&card, mission_hint)?;
	let entry = missions
		.entry(mission_id.clone())
		.or_insert_with(|| MissionAccumulator::new(mission_id.clone()));
	let mut summary = CardSummary {
		id: card.id.clone(),
		mission_id: mission_id.clone(),
		card_type: card.card_type.clone(),
		card_subtype: card.card_subtype.clone(),
		name: card.name.clone(),
		status: card.status.clone(),
		relative_path: path_info.relative.clone(),
	};
	if card.card_type.eq_ignore_ascii_case("Mission") {
		entry.name = Some(card.name.clone());
		// Mission summaries should appear ahead of other card types.
		summary.card_type = "Mission".into();
	}
	entry.cards.push(summary.clone());
	if cards_by_id.contains_key(&card.id) {
		return Err(validation_error(format!(
			"duplicate card id {} discovered while indexing the model home",
			card.id
		)));
	}
	cards_by_id.insert(
		card.id.clone(),
		CardRecord {
			mission_id,
			relative_path: path_info.relative,
			card,
			outgoing: Vec::new(),
			incoming: Vec::new(),
		},
	);
	Ok(())
}

fn derive_mission_id(
	card: &AuroraCard,
	mission_hint: Option<String>,
) -> Result<String, AuroraLibError> {
	if card.card_type.eq_ignore_ascii_case("Mission") {
		return Ok(card.id.clone());
	}
	if let Some(hint) = mission_hint {
		return Ok(hint);
	}
	Err(validation_error(format!(
		"card {} is not stored inside a mission directory",
		card.id
	)))
}

fn looks_like_mission_card(name: &str) -> bool {
	let Some(remainder) = name.strip_prefix("MIS-") else {
		return false;
	};
	let mut digits_seen = 0;
	for ch in remainder.chars() {
		if ch.is_ascii_digit() {
			digits_seen += 1;
			continue;
		}
		return digits_seen > 0 && ch == '-';
	}
	false
}

fn reject_tmp_directory(name: &str, path: &Path) -> Result<(), AuroraLibError> {
	if name.eq_ignore_ascii_case(TMP_DIR_NAME) {
		return Err(validation_error(format!(
			"model home contains forbidden tmp directory at {} (REQ-006/CNS-004)",
			path.display()
		)));
	}
	Ok(())
}

struct RelativePathInfo {
	relative: String,
}

fn relative_path_info(root: &Path, path: &Path) -> Result<RelativePathInfo, AuroraLibError> {
	let canonical = path
		.canonicalize()
		.map_err(|err| io_error(path.to_path_buf(), err))?;
	let relative = canonical.strip_prefix(root).map_err(|_| {
		validation_error(format!(
			"path {} is outside the model home",
			canonical.display()
		))
	})?;
	Ok(RelativePathInfo {
		relative: path_to_unix_string(relative),
	})
}

pub(crate) fn open_card(
	root_path: impl AsRef<Path>,
	relative_path: impl AsRef<Path>,
) -> Result<OpenedCard, AuroraLibError> {
	let canonical_root = canonicalize_model_home(root_path.as_ref())?;
	let resolved = resolve_relative_path(&canonical_root, relative_path.as_ref())?;
	let card = read_card_with_context(&resolved)?;
	Ok(OpenedCard {
		absolute_path: resolved,
		card,
	})
}

fn read_card_with_context(path: &Path) -> Result<AuroraCard, AuroraLibError> {
	read_card_from_path(path).map_err(|err| annotate_card_error(path, err))
}

fn annotate_card_error(path: &Path, err: AuroraLibError) -> AuroraLibError {
	match err {
		AuroraLibError::Serialization(source) => AuroraLibError::Validation(format!(
			"card serialization error at {}: {source}",
			path.display()
		)),
		AuroraLibError::Validation(message) => {
			AuroraLibError::Validation(format!("{}: {message}", path.display()))
		}
		other => other,
	}
}

fn resolve_relative_path(root: &Path, relative: &Path) -> Result<PathBuf, AuroraLibError> {
	let joined = root.join(relative);
	let canonical = joined
		.canonicalize()
		.map_err(|err| io_error(joined.clone(), err))?;
	if !canonical.starts_with(root) {
		return Err(validation_error(
			"requested path escapes the selected model home (REQ-030)",
		));
	}
	Ok(canonical)
}

fn path_to_unix_string(path: &Path) -> String {
	let display = path.to_string_lossy();
	display.replace('\\', "/")
}

fn display_path(path: &Path) -> String {
	path_to_unix_string(path)
}

fn validation_error(message: impl Into<String>) -> AuroraLibError {
	AuroraLibError::Validation(message.into())
}

fn io_error(path: impl Into<PathBuf>, err: std::io::Error) -> AuroraLibError {
	AuroraLibError::Io {
		path: path.into(),
		source: err,
	}
}

fn hydrate_incoming_links(cards_by_id: &mut BTreeMap<String, CardRecord>) {
	let mut incoming: HashMap<String, Vec<IncomingEdge>> = HashMap::new();
	for (source_id, record) in cards_by_id.iter_mut() {
		let mut outgoing_edges = Vec::new();
		for link in &record.card.links {
			outgoing_edges.push(LinkEdge {
				target: link.target.clone(),
				relationship: link.relationship.clone(),
			});
			incoming
				.entry(link.target.clone())
				.or_default()
				.push(IncomingEdge {
					source: source_id.clone(),
					relationship: link.relationship.clone(),
				});
		}
		record.outgoing = outgoing_edges;
	}
	for (target_id, edges) in incoming {
		if let Some(record) = cards_by_id.get_mut(&target_id) {
			record.incoming = edges;
		}
	}
}

fn build_filter_contexts(cards_by_id: &BTreeMap<String, CardRecord>) -> Vec<FilterContext> {
	let cards: Vec<&CardRecord> = cards_by_id.values().collect();
	VIEW_FILTER_DEFINITIONS
		.iter()
		.map(|definition| {
			let allowed_types = definition.allowed_card_types();
			let count = match &allowed_types {
				Some(set) => cards
					.iter()
					.filter(|record| set.contains(record.card.card_type.as_str()))
					.count(),
				None => cards.len(),
			};
			FilterContext {
				summary: ViewFilterSummary {
					id: definition.id.to_string(),
					label: definition.label.to_string(),
					description: definition.description.to_string(),
					root_card_types: definition
						.root_card_types
						.iter()
						.map(|value| value.to_string())
						.collect(),
					include_card_types: definition
						.include_card_types
						.iter()
						.map(|value| value.to_string())
						.collect(),
					card_count: count,
				},
				allowed_types,
			}
		})
		.collect()
}

fn insert_graph_node(
	nodes: &mut BTreeMap<String, GraphNodeSummary>,
	record: &CardRecord,
	distance: usize,
) {
	nodes
		.entry(record.card.id.clone())
		.and_modify(|existing| {
			if distance < existing.distance {
				existing.distance = distance;
			}
		})
		.or_insert_with(|| record.as_graph_node(distance));
}

fn normalize_filter_id(input: &str) -> String {
	input.trim().to_ascii_lowercase().replace([' ', '_'], "-")
}

struct CardRecord {
	mission_id: String,
	relative_path: String,
	card: AuroraCard,
	outgoing: Vec<LinkEdge>,
	incoming: Vec<IncomingEdge>,
}

impl CardRecord {
	fn as_graph_node(&self, distance: usize) -> GraphNodeSummary {
		GraphNodeSummary {
			id: self.card.id.clone(),
			mission_id: self.mission_id.clone(),
			card_type: self.card.card_type.clone(),
			card_subtype: self.card.card_subtype.clone(),
			name: self.card.name.clone(),
			status: self.card.status.clone(),
			relative_path: self.relative_path.clone(),
			distance,
		}
	}
}

#[derive(Debug, Clone)]
struct LinkEdge {
	target: String,
	relationship: String,
}

#[derive(Debug, Clone)]
struct IncomingEdge {
	source: String,
	relationship: String,
}

#[derive(Clone)]
struct FilterContext {
	summary: ViewFilterSummary,
	allowed_types: Option<HashSet<String>>,
}

impl FilterContext {
	fn allows(&self, card_type: &str, is_center: bool) -> bool {
		if is_center {
			return true;
		}
		match &self.allowed_types {
			Some(set) => set.contains(card_type),
			None => true,
		}
	}
}

struct ViewFilterDefinition {
	id: &'static str,
	label: &'static str,
	description: &'static str,
	root_card_types: &'static [&'static str],
	include_card_types: &'static [&'static str],
}

impl ViewFilterDefinition {
	fn allowed_card_types(&self) -> Option<HashSet<String>> {
		if self.root_card_types.is_empty() && self.include_card_types.is_empty() {
			return None;
		}
		let mut set: HashSet<String> = HashSet::new();
		for value in self
			.root_card_types
			.iter()
			.chain(self.include_card_types.iter())
		{
			set.insert((*value).to_string());
		}
		Some(set)
	}
}

struct MissionAccumulator {
	id: String,
	name: Option<String>,
	cards: Vec<CardSummary>,
}

impl MissionAccumulator {
	fn new(id: String) -> Self {
		Self {
			id,
			name: None,
			cards: Vec::new(),
		}
	}

	fn into_summary(mut self) -> MissionSummary {
		self.cards.sort_by(|a, b| {
			a.card_type
				.cmp(&b.card_type)
				.then_with(|| {
					a.card_subtype
						.as_deref()
						.unwrap_or("")
						.cmp(b.card_subtype.as_deref().unwrap_or(""))
				})
				.then_with(|| a.id.cmp(&b.id))
		});
		MissionSummary {
			name: self.name.clone().unwrap_or_else(|| self.id.clone()),
			id: self.id,
			cards: self.cards,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use aurora_lib::{AuditEntry, AuditTrail, AuroraCard, AuroraLink, write_card_to_path};
	use std::{collections::HashSet, path::Path};
	use tempfile::tempdir;

	#[test]
	fn load_model_home_indexes_cards_and_filters() -> Result<(), AuroraLibError> {
		let temp_dir = tempdir().expect("create temp dir");
		let root = temp_dir.path();

		let mut mission = sample_card("MIS-001", "Mission", "Demo Mission");
		mission.links.push(link("DRI-001", "establishes"));
		write_card(root, "MIS-001-Demo_Mission.json", &mission)?;

		let mut driver = sample_card("DRI-001", "Driver", "Primary Driver");
		driver.links.push(link("REQ-001", "drives"));
		write_card(root, "MIS-001/Driver/DRI-001.json", &driver)?;

		let mut requirement = sample_card("REQ-001", "Requirement", "Deterministic Backend");
		requirement.links.push(link("CAP-001", "necessitates"));
		write_card(root, "MIS-001/Requirement/REQ-001.json", &requirement)?;

		let mut capability = sample_card("CAP-001", "Capability", "Authoring Capability");
		capability.links.push(link("FEA-001", "enables"));
		write_card(root, "MIS-001/Capability/CAP-001.json", &capability)?;

		let feature = sample_card("FEA-001", "Feature", "Filtering Feature");
		write_card(root, "MIS-001/Feature/FEA-001.json", &feature)?;

		let model = load_model_home(root)?;
		let summary = model.summary();
		assert_eq!(summary.root, display_path(root));
		assert_eq!(summary.missions.len(), 1);
		assert_eq!(summary.card_count, 5);
		assert_eq!(summary.filters.len(), VIEW_FILTER_DEFINITIONS.len());

		let mission_cards = &summary.missions[0].cards;
		assert_eq!(mission_cards.len(), 5);
		assert!(
			mission_cards
				.iter()
				.all(|card| card.mission_id == "MIS-001")
		);

		let req_view = model.filter_cards(Some("requirements"));
		assert_eq!(req_view.card_count, 5);
		let ids: HashSet<_> = req_view.cards.iter().map(|card| card.id.as_str()).collect();
		for expected in ["MIS-001", "DRI-001", "REQ-001", "CAP-001", "FEA-001"] {
			assert!(
				ids.contains(expected),
				"missing card {expected} in requirements view"
			);
		}

		let all_view = model.filter_cards(Some("all"));
		assert_eq!(all_view.card_count, summary.card_count);

		Ok(())
	}

	#[test]
	fn graph_neighborhood_respects_filters_and_generations() -> Result<(), AuroraLibError> {
		let temp_dir = tempdir().expect("create temp dir");
		let root = temp_dir.path();

		let mut mission = sample_card("MIS-100", "Mission", "Graph Mission");
		mission.links.push(link("DRI-100", "establishes"));
		mission.links.push(link("SYS-100", "necessitates"));
		write_card(root, "MIS-100-Graph_Mission.json", &mission)?;

		let mut driver = sample_card("DRI-100", "Driver", "Graph Driver");
		driver.links.push(link("REQ-100", "drives"));
		write_card(root, "MIS-100/Driver/DRI-100.json", &driver)?;

		let mut requirement = sample_card("REQ-100", "Requirement", "Graph Requirement");
		requirement.links.push(link("CAP-100", "necessitates"));
		write_card(root, "MIS-100/Requirement/REQ-100.json", &requirement)?;

		let mut capability = sample_card("CAP-100", "Capability", "Graph Capability");
		capability.links.push(link("FEA-100", "enables"));
		write_card(root, "MIS-100/Capability/CAP-100.json", &capability)?;

		let mut feature = sample_card("FEA-100", "Feature", "Graph Feature");
		feature.links.push(link("COM-100", "implements"));
		write_card(root, "MIS-100/Feature/FEA-100.json", &feature)?;

		let mut system = sample_card("SYS-100", "System", "Graph System");
		system.links.push(link("COM-100", "hosts"));
		write_card(root, "MIS-100/System/SYS-100.json", &system)?;

		let component = sample_card("COM-100", "Component", "Graph Component");
		write_card(root, "MIS-100/Component/COM-100.json", &component)?;

		let model = load_model_home(root)?;
		let neighborhood = model.graph_neighborhood("COM-100", Some("component"), 2)?;
		assert_eq!(neighborhood.filter_id, "component");
		assert_eq!(neighborhood.max_depth, 2);
		assert_eq!(neighborhood.center.id, "COM-100");
		assert_eq!(neighborhood.center.card_type, "Component");

		let node_ids: Vec<_> = neighborhood
			.nodes
			.iter()
			.map(|node| node.id.as_str())
			.collect();
		assert_eq!(
			node_ids,
			vec!["SYS-100"],
			"component filter should only include systems hosting the component"
		);
		assert_eq!(neighborhood.edges.len(), 1);
		let edge = &neighborhood.edges[0];
		assert_eq!(edge.source_id, "SYS-100");
		assert_eq!(edge.target_id, "COM-100");
		assert_eq!(edge.relationship, "hosts");

		Ok(())
	}

	#[test]
	fn graph_neighborhood_rejects_unknown_cards() -> Result<(), AuroraLibError> {
		let temp_dir = tempdir().expect("create temp dir");
		let root = temp_dir.path();

		let mission = sample_card("MIS-200", "Mission", "Validation Mission");
		write_card(root, "MIS-200-Validation_Mission.json", &mission)?;

		let model = load_model_home(root)?;
		let error = model
			.graph_neighborhood("UNKNOWN", None, 1)
			.expect_err("unknown card should produce validation error");
		match error {
			AuroraLibError::Validation(message) => {
				assert!(message.contains("UNKNOWN"));
			}
			other => panic!("unexpected error: {:?}", other),
		}

		Ok(())
	}

	fn sample_card(id: &str, card_type: &str, name: &str) -> AuroraCard {
		AuroraCard {
			schema: None,
			id: id.to_string(),
			card_type: card_type.to_string(),
			card_subtype: None,
			name: name.to_string(),
			description: format!("Description for {name}"),
			status: Some("Proposed".into()),
			links: Vec::new(),
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: vec![AuditEntry {
					editor: "tester".into(),
					timestamp: "2024-01-01T00:00:00Z".into(),
					event: "created".into(),
					hash: None,
				}],
			},
			attributes: Default::default(),
		}
	}

	fn link(target: &str, relationship: &str) -> AuroraLink {
		AuroraLink {
			target: target.to_string(),
			relationship: relationship.to_string(),
		}
	}

	fn write_card(
		root: &Path,
		relative_path: &str,
		card: &AuroraCard,
	) -> Result<(), AuroraLibError> {
		let path = root.join(relative_path);
		write_card_to_path(&path, card)
	}
}
