use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::json;

use crate::errors::{AuroraError, Result};
use crate::model::{AuroraModel, Card};

const INSTRUCTIONS_DIR_ENV: &str = "AURORA_INSTRUCTIONS_ROOT";
const DOT_COMMAND_ENV: &str = "AURORA_DOT_COMMAND";

/// Summary information produced by rendering helpers.
#[derive(Debug, Clone)]
pub struct RenderSummary {
	pub cards_written: usize,
	pub views_written: usize,
	pub output_dir: PathBuf,
}

/// Render per-card Markdown documentation.
pub fn render_markdown(model: &AuroraModel, output_dir: impl AsRef<Path>) -> Result<RenderSummary> {
	let output_dir = output_dir.as_ref();
	fs::create_dir_all(output_dir).map_err(|err| AuroraError::io(output_dir, err))?;
	let mission_id = mission_identifier(model);

	let mut cards_written = 0;
	for card in model.iter_cards() {
		let path = card_markdown_path(card, model, output_dir, mission_id.as_deref());
		let mut buffer = String::new();
		buffer.push_str(&format!("# {} ({})\n\n", card.id, card.card_type));
		buffer.push_str(&format!("**Name:** {}\\\n\n", card.name));
		buffer.push_str("## Description\n\n");
		buffer.push_str(&card.description);
		buffer.push_str("\n\n## Links\n\n");
		if card.links.is_empty() {
			buffer.push_str("_No outgoing links._\n");
		} else {
			for link in &card.links {
				buffer.push_str(&format!("- `{}` → `{}`\n", link.relationship, link.target));
			}
		}

		write_text(&path, buffer)?;
		cards_written += 1;
	}

	Ok(RenderSummary {
		cards_written,
		views_written: 0,
		output_dir: output_dir.to_path_buf(),
	})
}

/// Render the canonical view set (DOT → SVG) defined by the registry.
pub fn render_views(model: &AuroraModel, output_dir: impl AsRef<Path>) -> Result<RenderSummary> {
	let output_dir = output_dir.as_ref();
	fs::create_dir_all(output_dir).map_err(|err| AuroraError::io(output_dir, err))?;

	let instructions_root = resolve_instructions_root(model)?;
	let palette = CardPalette::load(&instructions_root.join("Card_Definitions.md"))?;
	let registry = ViewRegistry::load(&instructions_root.join("View_Definitions.md"))?;
	let graphviz = GraphvizConfig::from_model(model)?;

	let mut views_written = 0usize;
	let views_dir = output_dir.join("Views");
	fs::create_dir_all(&views_dir).map_err(|err| AuroraError::io(&views_dir, err))?;
	let view_source_dir = views_dir.join("source");
	fs::create_dir_all(&view_source_dir).map_err(|err| AuroraError::io(&view_source_dir, err))?;
	for view in &registry.views {
		let dot_path = view_source_dir.join(format!("{}.view.dot", view.slug));
		let svg_path = views_dir.join(format!("{}.view.svg", view.slug));

		let nodes = collect_view_nodes(model, view);
		validate_view_connectivity(view, &nodes)?;
		let dot = build_dot(view, &nodes, &palette, &registry.card_colors, &graphviz);
		write_text(&dot_path, dot)?;
		render_svg(&dot_path, &svg_path, &graphviz)?;
		views_written += 1;
	}

	Ok(RenderSummary {
		cards_written: 0,
		views_written,
		output_dir: output_dir.to_path_buf(),
	})
}

/// Convenience helper that renders both cards and the relationship view.
pub fn render_all(model: &AuroraModel, output_dir: impl AsRef<Path>) -> Result<RenderSummary> {
	let output_dir = output_dir.as_ref();
	let cards = render_markdown(model, output_dir)?;
	let views = render_views(model, output_dir)?;
	Ok(RenderSummary {
		cards_written: cards.cards_written,
		views_written: views.views_written,
		output_dir: output_dir.to_path_buf(),
	})
}

#[derive(Debug, Clone, Default)]
struct CardColor {
	fill: Option<String>,
	font: Option<String>,
	stroke: Option<String>,
}

#[derive(Debug, Clone)]
struct CardPalette {
	shapes: HashMap<String, String>,
	icons: HashMap<String, String>,
}

impl CardPalette {
	fn load(path: &Path) -> Result<Self> {
		let contents = fs::read_to_string(path).map_err(|err| AuroraError::io(path, err))?;
		let mut shapes = HashMap::new();
		let mut icons = HashMap::new();
		let mut in_table = false;
		for line in contents.lines() {
			let trimmed = line.trim();
			if trimmed.starts_with("| Card type |") {
				in_table = true;
				continue;
			}
			if !in_table {
				continue;
			}
			if trimmed.starts_with("| ---") {
				continue;
			}
			if trimmed.is_empty() {
				if !shapes.is_empty() {
					break;
				}
				continue;
			}
			if !trimmed.starts_with('|') {
				continue;
			}
			let cells: Vec<String> = trimmed
				.trim_matches('|')
				.split('|')
				.map(|cell| cell.trim().to_string())
				.collect();
			if cells.len() < 3 {
				continue;
			}
			let card_type = normalize_card_type(&cells[0]);
			if card_type.is_empty() {
				continue;
			}
			let shape = cells[1].trim();
			if !shape.is_empty() {
				shapes.insert(card_type.clone(), shape.to_string());
			}
			let icon = cells[2].trim();
			if !icon.is_empty() {
				let symbol = icon_symbol(icon).unwrap_or(icon);
				icons.insert(card_type.clone(), symbol.to_string());
			}
		}
		if shapes.is_empty() {
			return Err(AuroraError::InvalidInput {
				message: format!("failed to parse card palette from {}", path.display()),
			});
		}
		Ok(CardPalette { shapes, icons })
	}

	fn shape_for(&self, card_type: &str) -> Option<&str> {
		self.shapes.get(card_type).map(|value| value.as_str())
	}

	fn icon_for(&self, card_type: &str) -> Option<&str> {
		self.icons.get(card_type).map(|value| value.as_str())
	}
}

#[derive(Debug, Clone)]
struct ViewRegistry {
	views: Vec<ViewSpec>,
	card_colors: HashMap<String, CardColor>,
}

#[derive(Debug, Clone)]
struct ViewSpec {
	name: String,
	slug: String,
	root_card_types: BTreeSet<String>,
	card_types: BTreeSet<String>,
	include_all: bool,
}

impl ViewSpec {
	fn includes_card_type(&self, card_type: &str) -> bool {
		self.include_all || self.card_types.contains(card_type)
	}
}

impl ViewRegistry {
	fn load(path: &Path) -> Result<Self> {
		let contents = fs::read_to_string(path).map_err(|err| AuroraError::io(path, err))?;
		let views = parse_view_table(&contents, path)?;
		let card_colors = parse_color_section(&contents);
		Ok(ViewRegistry { views, card_colors })
	}
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ConstraintAttributes {
	#[serde(default)]
	dot: DotAttributes,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct DotAttributes {
	#[serde(default)]
	graph_defaults: BTreeMap<String, String>,
	#[serde(default)]
	node_defaults: BTreeMap<String, String>,
	#[serde(default)]
	edge_defaults: BTreeMap<String, String>,
	#[serde(default)]
	card_type_symbols: HashMap<String, DotSymbol>,
	#[serde(default)]
	boundary_cluster: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct DotSymbol {
	#[serde(default)]
	shape: Option<String>,
	#[serde(default)]
	fillcolor: Option<String>,
	#[serde(default)]
	fontcolor: Option<String>,
	#[serde(default)]
	color: Option<String>,
}

#[derive(Debug, Clone)]
struct GraphvizConfig {
	graph_defaults: BTreeMap<String, String>,
	node_defaults: BTreeMap<String, String>,
	edge_defaults: BTreeMap<String, String>,
	card_symbols: HashMap<String, DotSymbol>,
	boundary_cluster: BTreeMap<String, String>,
}

impl Default for GraphvizConfig {
	fn default() -> Self {
		GraphvizConfig {
			graph_defaults: BTreeMap::from([
				("rankdir".to_string(), "LR".to_string()),
				("bgcolor".to_string(), "#FFFFFF".to_string()),
				("splines".to_string(), "spline".to_string()),
				("fontname".to_string(), "Inter".to_string()),
			]),
			node_defaults: BTreeMap::from([
				("style".to_string(), "filled".to_string()),
				("shape".to_string(), "box".to_string()),
				("fontname".to_string(), "Inter".to_string()),
				("fontsize".to_string(), "11".to_string()),
				("color".to_string(), "#111827".to_string()),
				("fillcolor".to_string(), "#E5E7EB".to_string()),
				("fontcolor".to_string(), "#111827".to_string()),
			]),
			edge_defaults: BTreeMap::from([
				("fontname".to_string(), "Inter".to_string()),
				("fontsize".to_string(), "9".to_string()),
				("color".to_string(), "#000000".to_string()),
				("fontcolor".to_string(), "#000000".to_string()),
			]),
			card_symbols: HashMap::new(),
			boundary_cluster: BTreeMap::from([
				("style".to_string(), "dashed".to_string()),
				("color".to_string(), "#6B7280".to_string()),
			]),
		}
	}
}

impl GraphvizConfig {
	fn from_model(model: &AuroraModel) -> Result<Self> {
		let mut config = GraphvizConfig::default();
		if let Some(card) = model.get("CNS-005") {
			if !card.attributes.is_null() {
				if let Ok(attrs) =
					serde_json::from_value::<ConstraintAttributes>(card.attributes.clone())
				{
					if !attrs.dot.graph_defaults.is_empty() {
						config.graph_defaults = attrs.dot.graph_defaults;
					}
					if !attrs.dot.node_defaults.is_empty() {
						config.node_defaults = attrs.dot.node_defaults;
					}
					if !attrs.dot.edge_defaults.is_empty() {
						config.edge_defaults = attrs.dot.edge_defaults;
					}
					if !attrs.dot.card_type_symbols.is_empty() {
						config.card_symbols = attrs.dot.card_type_symbols;
					}
					if !attrs.dot.boundary_cluster.is_empty() {
						config.boundary_cluster = attrs.dot.boundary_cluster;
					}
				}
			}
		}
		Ok(config)
	}

	fn dot_command(&self) -> String {
		env::var(DOT_COMMAND_ENV).unwrap_or_else(|_| "dot".to_string())
	}

	fn shape_for(&self, card_type: &str) -> Option<&str> {
		self.card_symbols
			.get(card_type)
			.and_then(|symbol| symbol.shape.as_deref())
	}

	fn fill_for(&self, card_type: &str) -> Option<&str> {
		self.card_symbols
			.get(card_type)
			.and_then(|symbol| symbol.fillcolor.as_deref())
	}

	fn font_for(&self, card_type: &str) -> Option<&str> {
		self.card_symbols.get(card_type).and_then(|symbol| {
			symbol
				.fontcolor
				.as_deref()
				.or_else(|| symbol.color.as_deref())
		})
	}

	fn default_shape(&self) -> Option<&str> {
		self.node_defaults.get("shape").map(|value| value.as_str())
	}

	fn default_fill(&self) -> Option<&str> {
		self.node_defaults
			.get("fillcolor")
			.map(|value| value.as_str())
	}

	fn default_font_color(&self) -> Option<&str> {
		self.node_defaults
			.get("fontcolor")
			.map(|value| value.as_str())
	}
}

#[derive(Debug, Clone)]
struct CardTypeList {
	card_types: BTreeSet<String>,
	include_all: bool,
}

#[derive(Debug, Clone)]
struct EdgeSpec {
	source: String,
	target: String,
	label: String,
}

#[derive(Debug, Clone)]
struct BoundaryCluster {
	id: String,
	label: String,
	members: Vec<String>,
}

fn resolve_instructions_root(model: &AuroraModel) -> Result<PathBuf> {
	if let Some(path) = instructions_override()? {
		return Ok(path);
	}
	let mut current = Some(model.home().root().to_path_buf());
	while let Some(dir) = current {
		let candidate = dir.join(".github/instructions");
		if candidate.is_dir() {
			return Ok(candidate);
		}
		current = dir.parent().map(|parent| parent.to_path_buf());
	}
	Err(AuroraError::InvalidInput {
		message: format!(
			"Unable to locate .github/instructions above {}",
			model.home().root().display()
		),
	})
}

fn instructions_override() -> Result<Option<PathBuf>> {
	match env::var(INSTRUCTIONS_DIR_ENV) {
		Ok(value) => {
			if value.trim().is_empty() {
				return Ok(None);
			}
			let path = PathBuf::from(value);
			if path.is_dir() {
				Ok(Some(path))
			} else {
				Err(AuroraError::InvalidInput {
					message: format!(
						"{}={} is not a directory",
						INSTRUCTIONS_DIR_ENV,
						path.display()
					),
				})
			}
		}
		Err(env::VarError::NotPresent) => Ok(None),
		Err(env::VarError::NotUnicode(value)) => Err(AuroraError::InvalidInput {
			message: format!(
				"{} contains invalid UTF-8: {:?}",
				INSTRUCTIONS_DIR_ENV, value
			),
		}),
	}
}

fn normalize_card_type(value: &str) -> String {
	let trimmed = value.trim().trim_matches('`');
	let base = match trimmed.split_once('(') {
		Some((head, _)) => head.trim(),
		None => trimmed,
	};
	base.trim().to_string()
}

fn parse_view_table(contents: &str, source: &Path) -> Result<Vec<ViewSpec>> {
	let mut views = Vec::new();
	let mut in_table = false;
	for line in contents.lines() {
		let trimmed = line.trim();
		if trimmed.starts_with("| View |") {
			in_table = true;
			continue;
		}
		if !in_table {
			continue;
		}
		if trimmed.starts_with("| ---") {
			continue;
		}
		if trimmed.is_empty() {
			if !views.is_empty() {
				break;
			}
			continue;
		}
		if !trimmed.starts_with('|') {
			if !views.is_empty() {
				break;
			}
			continue;
		}
		let cells: Vec<String> = trimmed
			.trim_matches('|')
			.split('|')
			.map(|cell| cell.trim().to_string())
			.collect();
		if cells.len() < 6 {
			continue;
		}
		let name = cells[0].clone();
		let _description = cells[1].clone();
		let root = parse_card_type_cell(&cells[2]);
		let included = parse_card_type_cell(&cells[3]);
		let supporting = parse_card_type_cell(&cells[4]);
		let mut card_types = included.card_types;
		card_types.extend(supporting.card_types);
		let include_all = included.include_all || supporting.include_all;
		let slug = slugify_view_name(&name);
		views.push(ViewSpec {
			name,
			slug,
			root_card_types: root.card_types,
			card_types,
			include_all,
		});
	}
	if views.is_empty() {
		return Err(AuroraError::InvalidInput {
			message: format!("failed to parse view registry from {}", source.display()),
		});
	}
	Ok(views)
}

fn parse_card_type_cell(cell: &str) -> CardTypeList {
	let mut include_all = false;
	let mut card_types = BTreeSet::new();
	for part in cell
		.split(|ch| ch == ',' || ch == '\n' || ch == ';')
		.map(|segment| segment.trim())
	{
		if part.is_empty() || part == "-" {
			continue;
		}
		if part.eq_ignore_ascii_case("All card types") {
			include_all = true;
			continue;
		}
		card_types.insert(normalize_card_type(part));
	}
	CardTypeList {
		card_types,
		include_all,
	}
}

fn slugify_view_name(name: &str) -> String {
	let base = if let Some((prefix, _)) = name.split_once(" View") {
		prefix.trim()
	} else {
		name.trim()
	};
	let mut slug = String::new();
	let mut underscore = false;
	for ch in base.chars() {
		let acceptable = matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9');
		if acceptable {
			slug.push(ch);
			underscore = false;
		} else if !underscore {
			slug.push('_');
			underscore = true;
		}
	}
	let cleaned = slug.trim_matches('_').to_string();
	if cleaned.is_empty() {
		"View".to_string()
	} else {
		cleaned
	}
}

fn parse_color_section(contents: &str) -> HashMap<String, CardColor> {
	let mut colors = HashMap::new();
	let mut in_section = false;
	for line in contents.lines() {
		let trimmed = line.trim();
		if trimmed.eq_ignore_ascii_case("## Canonical Color by Card Type") {
			in_section = true;
			continue;
		}
		if !in_section {
			continue;
		}
		if trimmed.starts_with("## ")
			&& !trimmed.eq_ignore_ascii_case("## Canonical Color by Card Type")
		{
			break;
		}
		if trimmed.is_empty() {
			continue;
		}
		if !trimmed.starts_with('-') {
			continue;
		}
		let entry = trimmed.trim_start_matches('-').trim();
		let (raw_key, rest) = match entry.split_once(':') {
			Some(parts) => parts,
			None => continue,
		};
		let card_type = expand_color_key(raw_key.trim());
		let color = colors.entry(card_type).or_insert_with(CardColor::default);
		for part in rest.split(',') {
			let fragment = part.trim().trim_end_matches(';').trim();
			if fragment.is_empty() {
				continue;
			}
			if let Some((prop, value)) = fragment.split_once(':') {
				let prop = prop.trim().to_lowercase();
				let value = value.trim().to_string();
				match prop.as_str() {
					"fill" => color.fill = Some(value),
					"color" | "fontcolor" => color.font = Some(value),
					"stroke" | "stroke-color" => color.stroke = Some(value),
					_ => {}
				}
			}
		}
	}
	colors
}

fn expand_color_key(raw: &str) -> String {
	let sanitized = raw.trim().trim_matches('`');
	if sanitized.contains(' ') {
		return normalize_card_type(sanitized);
	}
	sanitized
		.split('_')
		.filter(|segment| !segment.is_empty())
		.map(|segment| {
			let mut chars = segment.chars();
			match chars.next() {
				Some(first) => {
					let mut word = String::from(first.to_ascii_uppercase());
					for ch in chars {
						word.push(ch.to_ascii_lowercase());
					}
					word
				}
				None => String::new(),
			}
		})
		.collect::<Vec<String>>()
		.join(" ")
}

fn icon_symbol(name: &str) -> Option<&'static str> {
	match name.trim().to_ascii_lowercase().as_str() {
		"target" => Some("🎯"),
		"compass" => Some("🧭"),
		"checklist" => Some("☑️"),
		"spark" => Some("✨"),
		"star" => Some("⭐"),
		"layers" => Some("🗂️"),
		"app-window" => Some("🪟"),
		"cube" => Some("🧊"),
		"plug" => Some("🔌"),
		"file-text" => Some("📄"),
		"file" => Some("📁"),
		"shield-key" => Some("🛡️"),
		"database" => Some("🛢️"),
		"cloud" => Some("☁️"),
		"server" => Some("🖥️"),
		"server-cog" => Some("🖥️⚙️"),
		"workflow" => Some("🔁"),
		"steps" => Some("🪜"),
		"user" => Some("👤"),
		"book" => Some("📘"),
		"bolt" => Some("⚡"),
		"timeline" => Some("🕒"),
		"dot" => Some("•"),
		"split" => Some("🔀"),
		"lock" => Some("🔒"),
		"ruler" => Some("📏"),
		"alert-triangle" => Some("⚠️"),
		"bug" => Some("🐞"),
		"beaker" => Some("🧪"),
		"note-sticky" => Some("🗒️"),
		"braces" => Some("{}"),
		"square-dashed" => Some("▫️"),
		_ => None,
	}
}

fn collect_view_nodes<'a>(model: &'a AuroraModel, view: &ViewSpec) -> BTreeMap<String, &'a Card> {
	let mut nodes = BTreeMap::new();
	for card in model.iter_cards() {
		if is_annotation(card) {
			continue;
		}
		if view.includes_card_type(&card.card_type) {
			nodes.insert(card.id.clone(), card);
		}
	}
	let nodes = filter_nodes_by_roots(view, nodes);
	let mut with_annotations = nodes.clone();
	include_annotations_for_view(model, &mut with_annotations, &nodes);
	with_annotations
}

fn validate_view_connectivity(view: &ViewSpec, nodes: &BTreeMap<String, &Card>) -> Result<()> {
	let node_ids = collect_diagram_node_ids(nodes);
	if node_ids.is_empty() {
		return Ok(());
	}
	let edges = collect_edges(nodes);
	let incoming = collect_incoming_counts(&node_ids, &edges);
	let mut roots = collect_root_nodes_by_type(view, nodes);
	if roots.is_empty() {
		roots = collect_root_nodes(&node_ids, &incoming);
	}
	if roots.is_empty() {
		return Err(AuroraError::InvalidInput {
			message: format!(
				"View '{}' has no root nodes; all nodes have incoming edges in the diagram.",
				view.name
			),
		});
	}
	let reachable = collect_connected_nodes(&roots, &edges);
	let unreachable = node_ids.difference(&reachable).cloned().collect::<Vec<_>>();
	if !unreachable.is_empty() {
		return Err(AuroraError::InvalidInput {
			message: format!(
				"View '{}' has nodes without a path from a root: {}",
				view.name,
				unreachable.join(", ")
			),
		});
	}
	Ok(())
}

fn collect_root_nodes_by_type(view: &ViewSpec, nodes: &BTreeMap<String, &Card>) -> Vec<String> {
	if view.root_card_types.is_empty() {
		return Vec::new();
	}
	let mut roots = Vec::new();
	for (id, card) in nodes {
		if card.card_type == "Boundary" {
			continue;
		}
		if view.root_card_types.contains(&card.card_type) {
			roots.push(id.clone());
		}
	}
	roots
}

fn collect_diagram_node_ids(nodes: &BTreeMap<String, &Card>) -> BTreeSet<String> {
	let mut node_ids = BTreeSet::new();
	for (id, card) in nodes {
		if card.card_type != "Boundary" {
			node_ids.insert(id.clone());
		}
	}
	node_ids
}

fn collect_incoming_counts(
	node_ids: &BTreeSet<String>,
	edges: &[EdgeSpec],
) -> HashMap<String, usize> {
	let mut incoming = HashMap::new();
	for id in node_ids {
		incoming.insert(id.clone(), 0usize);
	}
	for edge in edges {
		if let Some(count) = incoming.get_mut(&edge.target) {
			*count += 1;
		}
	}
	incoming
}

fn collect_root_nodes(
	node_ids: &BTreeSet<String>,
	incoming: &HashMap<String, usize>,
) -> Vec<String> {
	let mut roots = Vec::new();
	for id in node_ids {
		let count = incoming.get(id).copied().unwrap_or(0);
		if count == 0 {
			roots.push(id.clone());
		}
	}
	roots
}

fn collect_connected_nodes(roots: &[String], edges: &[EdgeSpec]) -> BTreeSet<String> {
	let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
	for edge in edges {
		adjacency
			.entry(edge.source.clone())
			.or_default()
			.push(edge.target.clone());
		adjacency
			.entry(edge.target.clone())
			.or_default()
			.push(edge.source.clone());
	}
	let mut connected = BTreeSet::new();
	let mut stack = roots.to_vec();
	while let Some(node) = stack.pop() {
		if !connected.insert(node.clone()) {
			continue;
		}
		if let Some(neighbors) = adjacency.get(&node) {
			for neighbor in neighbors {
				if !connected.contains(neighbor) {
					stack.push(neighbor.clone());
				}
			}
		}
	}
	connected
}

fn filter_nodes_by_roots<'a>(
	view: &ViewSpec,
	nodes: BTreeMap<String, &'a Card>,
) -> BTreeMap<String, &'a Card> {
	let node_ids = collect_diagram_node_ids(&nodes);
	if node_ids.is_empty() {
		return nodes;
	}
	let edges = collect_edges(&nodes);
	let incoming = collect_incoming_counts(&node_ids, &edges);
	let mut roots = collect_root_nodes_by_type(view, &nodes);
	if roots.is_empty() {
		roots = collect_root_nodes(&node_ids, &incoming);
	}
	if roots.is_empty() {
		return nodes;
	}
	let reachable = collect_connected_nodes(&roots, &edges);
	let mut filtered = BTreeMap::new();
	for (id, card) in nodes {
		if reachable.contains(&id) {
			filtered.insert(id, card);
		}
	}
	filtered
}

fn include_annotations_for_view<'a>(
	model: &'a AuroraModel,
	nodes: &mut BTreeMap<String, &'a Card>,
	base_nodes: &BTreeMap<String, &'a Card>,
) {
	let linked_targets = collect_linked_targets(base_nodes);
	let mut additions = Vec::new();
	for card in model.iter_cards() {
		if nodes.contains_key(&card.id) {
			continue;
		}
		if !is_annotation(card) {
			continue;
		}
		if should_include_annotation(card, base_nodes, &linked_targets) {
			additions.push(card.id.clone());
		}
	}
	for id in additions {
		if let Some(card) = model.get(&id) {
			nodes.insert(id, card);
		}
	}
}

fn collect_linked_targets(nodes: &BTreeMap<String, &Card>) -> BTreeSet<String> {
	let mut linked = BTreeSet::new();
	for card in nodes.values() {
		for link in &card.links {
			linked.insert(link.target.clone());
		}
	}
	linked
}

fn is_annotation(card: &Card) -> bool {
	matches!(card.card_type.as_str(), "Boundary" | "Note")
}

fn should_include_annotation(
	card: &Card,
	nodes: &BTreeMap<String, &Card>,
	linked_targets: &BTreeSet<String>,
) -> bool {
	if linked_targets.contains(&card.id) {
		return true;
	}
	if card.card_type == "Boundary" {
		return card.links.iter().any(|link| {
			link.relationship.eq_ignore_ascii_case("contains") && nodes.contains_key(&link.target)
		});
	}
	if card.card_type == "Note" {
		return false;
	}
	false
}

fn build_dot(
	view: &ViewSpec,
	nodes: &BTreeMap<String, &Card>,
	palette: &CardPalette,
	colors: &HashMap<String, CardColor>,
	config: &GraphvizConfig,
) -> String {
	let edges = collect_edges(nodes);
	build_dot_with_edges(view, nodes, &edges, palette, colors, config)
}

fn build_dot_with_edges(
	view: &ViewSpec,
	nodes: &BTreeMap<String, &Card>,
	edges: &[EdgeSpec],
	palette: &CardPalette,
	colors: &HashMap<String, CardColor>,
	config: &GraphvizConfig,
) -> String {
	let mut lines = Vec::new();
	lines.push(format!("digraph \"{}\" {{", dot_escape(&view.name)));
	if !config.graph_defaults.is_empty() {
		lines.push(format!(
			"\tgraph [{}];",
			format_attributes(&config.graph_defaults)
		));
	}
	if !config.node_defaults.is_empty() {
		lines.push(format!(
			"\tnode [{}];",
			format_attributes(&config.node_defaults)
		));
	}
	if !config.edge_defaults.is_empty() {
		lines.push(format!(
			"\tedge [{}];",
			format_attributes(&config.edge_defaults)
		));
	}

	for (id, card) in nodes {
		if card.card_type == "Boundary" {
			continue;
		}
		let mut attrs = BTreeMap::new();
		attrs.insert(
			"label".to_string(),
			node_label(card, palette.icon_for(&card.card_type)),
		);
		let shape = palette
			.shape_for(&card.card_type)
			.or_else(|| config.shape_for(&card.card_type))
			.or_else(|| config.default_shape())
			.unwrap_or("box");
		attrs.insert("shape".to_string(), shape.to_string());
		let fill = colors
			.get(&card.card_type)
			.and_then(|color| color.fill.clone())
			.or_else(|| {
				config
					.fill_for(&card.card_type)
					.map(|value| value.to_string())
			})
			.or_else(|| config.default_fill().map(|value| value.to_string()));
		if let Some(fillcolor) = fill {
			attrs.insert("fillcolor".to_string(), fillcolor);
		}
		let font = colors
			.get(&card.card_type)
			.and_then(|color| color.font.clone())
			.or_else(|| {
				config
					.font_for(&card.card_type)
					.map(|value| value.to_string())
			})
			.or_else(|| config.default_font_color().map(|value| value.to_string()));
		if let Some(fontcolor) = font {
			attrs.insert("fontcolor".to_string(), fontcolor);
		}
		lines.push(format!("\t{} [{}];", dot_id(id), format_attributes(&attrs)));
	}

	let mut sorted_edges = edges.to_vec();
	sorted_edges.sort_by(|a, b| match a.source.cmp(&b.source) {
		std::cmp::Ordering::Equal => match a.target.cmp(&b.target) {
			std::cmp::Ordering::Equal => a.label.cmp(&b.label),
			other => other,
		},
		other => other,
	});
	for edge in sorted_edges {
		let mut attrs = BTreeMap::new();
		if !edge.label.is_empty() {
			attrs.insert("label".to_string(), edge.label.clone());
		}
		if attrs.is_empty() {
			lines.push(format!(
				"\t{} -> {};",
				dot_id(&edge.source),
				dot_id(&edge.target)
			));
		} else {
			lines.push(format!(
				"\t{} -> {} [{}];",
				dot_id(&edge.source),
				dot_id(&edge.target),
				format_attributes(&attrs)
			));
		}
	}

	let mut clusters = collect_boundary_clusters(nodes);
	clusters.sort_by(|a, b| a.id.cmp(&b.id));
	for cluster in clusters {
		lines.push(format!(
			"\tsubgraph \"cluster_{}\" {{",
			dot_escape(&cluster.id)
		));
		let mut attrs = config.boundary_cluster.clone();
		attrs
			.entry("label".to_string())
			.or_insert(cluster.label.clone());
		if !attrs.is_empty() {
			lines.push(format!("\t\t{};", format_attributes(&attrs)));
		}
		for member in cluster.members {
			lines.push(format!("\t\t{};", dot_id(&member)));
		}
		lines.push("\t}".to_string());
	}

	lines.push("}".to_string());
	lines.join("\n")
}

fn collect_edges(nodes: &BTreeMap<String, &Card>) -> Vec<EdgeSpec> {
	let mut edges = Vec::new();
	for (id, card) in nodes {
		if card.card_type == "Boundary" {
			continue;
		}
		for link in &card.links {
			if let Some(target) = nodes.get(&link.target) {
				if target.card_type == "Boundary" {
					continue;
				}
				edges.push(EdgeSpec {
					source: id.clone(),
					target: link.target.clone(),
					label: link.relationship.clone(),
				});
			}
		}
	}
	edges
}

fn collect_boundary_clusters(nodes: &BTreeMap<String, &Card>) -> Vec<BoundaryCluster> {
	let mut clusters = Vec::new();
	for (id, card) in nodes {
		if card.card_type != "Boundary" {
			continue;
		}
		let mut members: Vec<String> = card
			.links
			.iter()
			.filter(|link| link.relationship.eq_ignore_ascii_case("contains"))
			.filter(|link| {
				matches!(
					nodes.get(&link.target),
					Some(target_card) if target_card.card_type != "Boundary"
				)
			})
			.map(|link| link.target.clone())
			.collect();
		members.sort();
		members.dedup();
		if members.is_empty() {
			continue;
		}
		clusters.push(BoundaryCluster {
			id: id.clone(),
			label: format!("{} ({})", card.name, card.id),
			members,
		});
	}
	clusters
}

fn node_label(card: &Card, icon: Option<&str>) -> String {
	let mut lines = Vec::new();
	let mut type_line = String::new();
	if let Some(symbol) = icon {
		if !symbol.is_empty() {
			type_line.push_str(&html_escape(symbol));
			type_line.push(' ');
		}
	}
	type_line.push_str(&html_escape(&card.card_type));
	type_line.push(':');
	lines.push(type_line);
	if let Some(subtype) = card.card_subtype.as_deref() {
		if !subtype.trim().is_empty() {
			lines.push(format!("({})", html_escape(subtype)));
		}
	}
	lines.push(html_escape(&card.name));
	lines.push(format!("({})", html_escape(&card.id)));
	format!("<<FONT>{}</FONT>>", lines.join("<br />"))
}

fn render_svg(dot_path: &Path, svg_path: &Path, config: &GraphvizConfig) -> Result<()> {
	if let Some(parent) = svg_path.parent() {
		fs::create_dir_all(parent).map_err(|err| AuroraError::io(parent, err))?;
	}
	let command = config.dot_command();
	let output = Command::new(&command)
		.arg("-Tsvg")
		.arg(dot_path)
		.arg("-o")
		.arg(svg_path)
		.output();
	match output {
		Ok(result) => {
			if result.status.success() {
				Ok(())
			} else {
				let stderr = String::from_utf8_lossy(&result.stderr);
				Err(AuroraError::InvalidInput {
					message: format!("Graphviz command '{}' failed: {}", command, stderr.trim()),
				})
			}
		}
		Err(err) => {
			if err.kind() == ErrorKind::NotFound {
				Err(AuroraError::InvalidInput {
					message: format!(
						"command '{}' was not found. Install Graphviz (dot) to render views.",
						command
					),
				})
			} else {
				Err(AuroraError::io(dot_path, err))
			}
		}
	}
}

fn format_attributes(attrs: &BTreeMap<String, String>) -> String {
	attrs
		.iter()
		.map(|(key, value)| {
			if key == "label" && value.starts_with("<<") && value.ends_with(">>") {
				format!("{key}={value}")
			} else {
				format!("{key}=\"{}\"", dot_escape(value))
			}
		})
		.collect::<Vec<String>>()
		.join(", ")
}

fn dot_id(value: &str) -> String {
	format!("\"{}\"", dot_escape(value))
}

fn dot_escape(value: &str) -> String {
	value
		.replace('\\', "\\\\")
		.replace('"', "\\\"")
		.replace('\n', "\\n")
}

fn html_escape(value: &str) -> String {
	value
		.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
}

fn mission_identifier(model: &AuroraModel) -> Option<String> {
	model
		.iter_cards()
		.find(|card| card.card_type == "Mission")
		.map(|card| card.id.clone())
}

fn card_markdown_path(
	card: &Card,
	model: &AuroraModel,
	output_root: &Path,
	mission_id: Option<&str>,
) -> PathBuf {
	if let Some(source) = card.source_path() {
		if let Ok(relative) = source.strip_prefix(model.home().root()) {
			let mut target = relative.to_path_buf();
			target.set_extension("md");
			if let Some(mission) = mission_id {
				if let Ok(stripped) = target.strip_prefix(mission) {
					return output_root.join(stripped);
				}
			}
			return output_root.join(target);
		}
	}
	let file_name = format!("{}-{}.md", card.id, card.name.replace(' ', "_"));
	output_root.join(file_name)
}

/// Write a compact single-file representation of the model for agent consumption.
pub fn write_compact_model(model: &AuroraModel, output_path: Option<PathBuf>) -> Result<PathBuf> {
	let target_path = if let Some(path) = output_path {
		path
	} else {
		let mission_id = model
			.iter_cards()
			.find(|card| card.card_type == "Mission")
			.map(|card| card.id.clone())
			.unwrap_or_else(|| "MIS-COMPACT".to_string());
		model
			.home()
			.root()
			.join(format!("AGENT-{}.jsjson", mission_id))
	};

	let mut compact_cards = Vec::new();
	for card in model.iter_cards() {
		let mut value = serde_json::to_value(card).map_err(|err| AuroraError::InvalidInput {
			message: err.to_string(),
		})?;
		if let serde_json::Value::Object(ref mut map) = value {
			map.remove("$schema");
			map.remove("audit_trail");
			map.remove("source_path");
		}
		compact_cards.push(value);
	}

	let payload = json!({
		"cards": compact_cards,
		"generated_at": SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map(|duration| duration.as_secs())
			.unwrap_or_default(),
	});

	write_json_pretty(&target_path, &payload)?;
	Ok(target_path)
}

fn write_text(path: &Path, contents: String) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).map_err(|err| AuroraError::io(parent, err))?;
	}
	let tmp_path = tmp_path(path);
	{
		let mut file = File::create(&tmp_path).map_err(|err| AuroraError::io(&tmp_path, err))?;
		file.write_all(contents.as_bytes())
			.map_err(|err| AuroraError::io(&tmp_path, err))?;
		file.flush()
			.map_err(|err| AuroraError::io(&tmp_path, err))?;
	}
	fs::rename(&tmp_path, path).map_err(|err| AuroraError::io(path, err))?;
	Ok(())
}

fn write_json_pretty(path: &Path, value: &serde_json::Value) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).map_err(|err| AuroraError::io(parent, err))?;
	}
	let tmp_path = tmp_path(path);
	{
		let file = File::create(&tmp_path).map_err(|err| AuroraError::io(&tmp_path, err))?;
		serde_json::to_writer_pretty(&file, value).map_err(|err| AuroraError::InvalidInput {
			message: err.to_string(),
		})?;
	}
	fs::rename(&tmp_path, path).map_err(|err| AuroraError::io(path, err))?;
	Ok(())
}

fn tmp_path(path: &Path) -> PathBuf {
	let mut tmp_name = path
		.file_name()
		.and_then(|name| name.to_str())
		.map(|name| format!(".{name}.tmp"))
		.unwrap_or_else(|| ".aurora.tmp".to_string());
	if tmp_name == path.file_name().and_then(|n| n.to_str()).unwrap_or("") {
		tmp_name.push_str(".tmp");
	}
	path.with_file_name(tmp_name)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::discovery::ModelHome;
	use crate::model::{AuditTrail, AuroraModel, Card, Link};
	use serde_json::Value;
	use std::collections::BTreeMap;
	use std::env;
	use std::path::{Path, PathBuf};
	use std::sync::{Mutex, OnceLock};
	use tempfile::TempDir;

	#[test]
	fn render_markdown_creates_card_files() {
		let (tmp, model) = sample_model();
		let output = tmp.path().join("render");
		let summary = render_markdown(&model, &output).expect("render should succeed");
		assert_eq!(summary.cards_written, 1);
		assert!(output.join("MIS-001-Provide_Default_Tooling.md").exists());
	}

	#[test]
	fn render_views_emits_dot_and_svg_sources_only() {
		let (tmp, model) = sample_model();
		let output = tmp.path().join("views");
		let instructions = instructions_fixture_dir();
		let _guard = env_guard().lock().unwrap();
		let original = env::var(INSTRUCTIONS_DIR_ENV).ok();
		set_instruction_env(&instructions);
		let summary = render_views(&model, &output).expect("view render should succeed");
		restore_instruction_env(original);
		assert!(summary.views_written > 0);
		let requirements_md = output.join("Views/Requirements.view.md");
		assert!(!requirements_md.exists());
		let dot_path = output.join("Views/source/Requirements.view.dot");
		assert!(dot_path.exists());
		assert!(output.join("Views/Requirements.view.svg").exists());
		let dot = std::fs::read_to_string(dot_path).expect("dot");
		assert!(dot.contains("🎯 Mission:"));
		assert!(dot.contains("<br />"));
	}

	#[test]
	fn node_label_includes_subtype_line() {
		let card = Card {
			schema: None,
			id: "COM-009".into(),
			card_type: "Component".into(),
			card_subtype: Some("struct".into()),
			name: "Render Node".into(),
			description: "Test card".into(),
			status: None,
			links: Vec::new(),
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let label = node_label(&card, None);
		assert_eq!(
			label,
			"<<FONT>Component:<br />(struct)<br />Render Node<br />(COM-009)</FONT>>"
		);
	}

	#[test]
	fn node_label_omits_empty_subtype() {
		let card = Card {
			schema: None,
			id: "COM-010".into(),
			card_type: "Component".into(),
			card_subtype: Some("  ".into()),
			name: "Render View".into(),
			description: "Test card".into(),
			status: None,
			links: Vec::new(),
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let label = node_label(&card, None);
		assert_eq!(
			label,
			"<<FONT>Component:<br />Render View<br />(COM-010)</FONT>>"
		);
	}

	#[test]
	fn collect_view_nodes_includes_linked_notes() -> Result<()> {
		let tmp_dir = tempfile::tempdir().map_err(|err| AuroraError::InvalidInput {
			message: err.to_string(),
		})?;
		let schema_path = tmp_dir.path().join("Aurora.schema.json");
		std::fs::write(&schema_path, "{}").map_err(|err| AuroraError::io(&schema_path, err))?;
		let home = ModelHome::new(tmp_dir.path())?;
		let mission = Card {
			schema: None,
			id: "MIS-001".into(),
			card_type: "Mission".into(),
			card_subtype: None,
			name: "Provide Default Tooling".into(),
			description: "Test mission".into(),
			status: None,
			links: vec![Link {
				target: "NOT-001".into(),
				relationship: "includes".into(),
			}],
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let note = Card {
			schema: None,
			id: "NOT-001".into(),
			card_type: "Note".into(),
			card_subtype: None,
			name: "Modeling Footnote".into(),
			description: "Test note".into(),
			status: None,
			links: Vec::new(),
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let model = AuroraModel::new(home, vec![mission, note]);
		let mut card_types = BTreeSet::new();
		card_types.insert("Mission".into());
		let view = ViewSpec {
			name: "Mission Only".into(),
			slug: "Mission_Only".into(),
			root_card_types: BTreeSet::new(),
			card_types,
			include_all: false,
		};
		let nodes = collect_view_nodes(&model, &view);
		if !nodes.contains_key("NOT-001") {
			return Err(AuroraError::InvalidInput {
				message: "linked note missing from view nodes".to_string(),
			});
		}
		Ok(())
	}

	#[test]
	fn collect_view_nodes_skips_unlinked_notes() -> Result<()> {
		let tmp_dir = tempfile::tempdir().map_err(|err| AuroraError::InvalidInput {
			message: err.to_string(),
		})?;
		let schema_path = tmp_dir.path().join("Aurora.schema.json");
		std::fs::write(&schema_path, "{}").map_err(|err| AuroraError::io(&schema_path, err))?;
		let home = ModelHome::new(tmp_dir.path())?;
		let mission = Card {
			schema: None,
			id: "MIS-001".into(),
			card_type: "Mission".into(),
			card_subtype: None,
			name: "Provide Default Tooling".into(),
			description: "Test mission".into(),
			status: None,
			links: Vec::new(),
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let note = Card {
			schema: None,
			id: "NOT-001".into(),
			card_type: "Note".into(),
			card_subtype: None,
			name: "Modeling Footnote".into(),
			description: "Test note".into(),
			status: None,
			links: Vec::new(),
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let model = AuroraModel::new(home, vec![mission, note]);
		let mut card_types = BTreeSet::new();
		card_types.insert("Mission".into());
		card_types.insert("Note".into());
		let view = ViewSpec {
			name: "Mission and Note".into(),
			slug: "Mission_and_Note".into(),
			root_card_types: BTreeSet::new(),
			card_types,
			include_all: false,
		};
		let nodes = collect_view_nodes(&model, &view);
		if nodes.contains_key("NOT-001") {
			return Err(AuroraError::InvalidInput {
				message: "unlinked note should be excluded from view nodes".to_string(),
			});
		}
		Ok(())
	}

	#[test]
	fn tmp_path_is_hidden() {
		let path = PathBuf::from("/tmp/output.md");
		let tmp = tmp_path(&path);
		let name = tmp.file_name().and_then(|name| name.to_str()).unwrap();
		assert!(name.starts_with(".output.md"));
		assert!(name.ends_with(".tmp"));
	}

	fn sample_model() -> (TempDir, AuroraModel) {
		let tmp_dir = tempfile::tempdir().expect("temp dir");
		std::fs::write(tmp_dir.path().join("Aurora.schema.json"), "{}").expect("schema file");
		let home = ModelHome::new(tmp_dir.path()).expect("model home");
		let card = Card {
			schema: None,
			id: "MIS-001".into(),
			card_type: "Mission".into(),
			card_subtype: None,
			name: "Provide Default Tooling".into(),
			description: "Test mission".into(),
			status: None,
			links: Vec::new(),
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let model = AuroraModel::new(home, vec![card]);
		(tmp_dir, model)
	}

	fn instructions_fixture_dir() -> PathBuf {
		let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		for ancestor in crate_dir.ancestors() {
			let candidate = ancestor.join(".github/instructions");
			if candidate.is_dir() {
				return candidate;
			}
		}
		panic!("Unable to locate .github/instructions for tests");
	}

	fn env_guard() -> &'static Mutex<()> {
		static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
		GUARD.get_or_init(|| Mutex::new(()))
	}

	fn set_instruction_env(path: &Path) {
		unsafe {
			env::set_var(INSTRUCTIONS_DIR_ENV, path);
		}
	}

	fn restore_instruction_env(previous: Option<String>) {
		unsafe {
			if let Some(value) = previous {
				env::set_var(INSTRUCTIONS_DIR_ENV, value);
			} else {
				env::remove_var(INSTRUCTIONS_DIR_ENV);
			}
		}
	}
}
