mod graphviz_plain;
mod icons;
mod themed_svg;

use std::env;
use std::fs::{self};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
	collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
	fs::File,
};

use serde::Deserialize;
use serde_json::json;

use crate::errors::{AuroraError, Result};
use crate::model::{AuroraModel, Card};
use crate::registry::{CardDefinition, ViewDefinition};

const DOT_COMMAND_ENV: &str = "AURORA_DOT_COMMAND";
const ICON_POINT_SIZE: i32 = 32;

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
		buffer.push_str(&format!("# {}: **{}**\n\n", card.id, card.card_type));
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

	render_executive_summary(model, output_dir)?;

	Ok(RenderSummary {
		cards_written,
		views_written: 0,
		output_dir: output_dir.to_path_buf(),
	})
}

fn render_executive_summary(model: &AuroraModel, output_dir: &Path) -> Result<()> {
	let mission = match mission_card(model) {
		Some(card) => card,
		None => return Ok(()),
	};
	let path = executive_summary_path(output_dir, &mission.id);
	let summary = build_executive_summary(model, mission);
	write_text(&path, summary)
}

fn executive_summary_path(output_root: &Path, mission_id: &str) -> PathBuf {
	output_root.join(format!("{mission_id}-Executive_Summary.md"))
}

fn build_executive_summary(model: &AuroraModel, mission: &Card) -> String {
	let mut drivers = collect_cards_by_type(model, "Driver");
	drivers.sort_by(|left, right| left.id.cmp(&right.id));
	let mut requirements = collect_cards_by_type(model, "Requirement");
	requirements.sort_by(|left, right| left.id.cmp(&right.id));

	let mut buffer = String::new();
	buffer.push_str(&format!("# {} Executive Summary\n\n", mission.id));
	buffer.push_str(&format!(
		"This executive summary captures the mission intent plus {} and {}.\n\n",
		format_count(drivers.len(), "driver", "drivers"),
		format_count(requirements.len(), "requirement", "requirements")
	));

	buffer.push_str("## Mission\n\n");
	buffer.push_str(&format!("- **ID:** `{}`\n", mission.id));
	buffer.push_str(&format!(
		"- **Name:** {}\n",
		normalize_summary_text(&mission.name)
	));
	let description = normalize_summary_text(&mission.description);
	if description.is_empty() {
		buffer.push_str("- **Description:** No description provided.\n\n");
	} else {
		buffer.push_str(&format!("- **Description:** {}\n\n", description));
	}

	buffer.push_str("## Drivers\n\n");
	if drivers.is_empty() {
		buffer.push_str("_No drivers recorded for this mission._\n\n");
	} else {
		for driver in drivers {
			buffer.push_str(&format!(
				"- `{}` — {}\n",
				driver.id,
				card_summary_line(driver)
			));
		}
		buffer.push('\n');
	}

	buffer.push_str("## Requirements\n\n");
	if requirements.is_empty() {
		buffer.push_str("_No requirements recorded for this mission._\n");
	} else {
		for requirement in requirements {
			buffer.push_str(&format!(
				"- `{}` — {}\n",
				requirement.id,
				card_summary_line(requirement)
			));
		}
	}
	buffer
}

fn collect_cards_by_type<'a>(model: &'a AuroraModel, card_type: &str) -> Vec<&'a Card> {
	model
		.iter_cards()
		.filter(|card| card.card_type.eq_ignore_ascii_case(card_type))
		.collect()
}

fn card_summary_line(card: &Card) -> String {
	let name = normalize_summary_text(&card.name);
	let description = normalize_summary_text(&card.description);
	match (name.is_empty(), description.is_empty()) {
		(false, false) => {
			if name.ends_with(['.', '!', '?']) {
				format!("{name} {description}")
			} else {
				format!("{name}. {description}")
			}
		}
		(false, true) => name,
		(true, false) => description,
		(true, true) => "No summary available.".to_string(),
	}
}

fn normalize_summary_text(value: &str) -> String {
	value
		.split_whitespace()
		.filter(|segment| !segment.is_empty())
		.collect::<Vec<_>>()
		.join(" ")
}

fn format_count(count: usize, singular: &str, plural: &str) -> String {
	if count == 1 {
		format!("1 {singular}")
	} else {
		format!("{count} {plural}")
	}
}

/// Render the canonical view set (DOT → SVG) defined by the registry.
pub fn render_views(model: &AuroraModel, output_dir: impl AsRef<Path>) -> Result<RenderSummary> {
	let output_dir = output_dir.as_ref();
	fs::create_dir_all(output_dir).map_err(|err| AuroraError::io(output_dir, err))?;
	render_views_with_registry(model, output_dir)
}

/// Render the canonical view set (embedded registries; instructions root ignored).
pub fn render_views_with_instructions(
	model: &AuroraModel,
	output_dir: impl AsRef<Path>,
	_instructions_root: impl AsRef<Path>,
) -> Result<RenderSummary> {
	render_views(model, output_dir)
}

fn render_views_with_registry(model: &AuroraModel, output_dir: &Path) -> Result<RenderSummary> {
	let palette = CardPalette::embedded()?;
	let icon_glyphs = palette.icon_glyphs();
	let registry = ViewRegistry::embedded()?;
	let graphviz = GraphvizConfig::from_model(model)?;

	let mut views_written = 0usize;
	let views_dir = output_dir.join("Views");
	fs::create_dir_all(&views_dir).map_err(|err| AuroraError::io(&views_dir, err))?;
	let view_source_dir = views_dir.join("source");
	fs::create_dir_all(&view_source_dir).map_err(|err| AuroraError::io(&view_source_dir, err))?;
	for view in &registry.views {
		let legacy_dot = view_source_dir.join(format!("{}.view.dot", view.slug));
		let legacy_svg = views_dir.join(format!("{}.view.svg", view.slug));
		remove_if_exists(&legacy_dot)?;
		remove_if_exists(&legacy_svg)?;

		let base_nodes = collect_view_base_nodes(model, view);
		let root_ids = collect_view_root_ids(view, &base_nodes);
		if root_ids.is_empty() && !is_everything_view(view) {
			continue;
		}
		for root_id in &root_ids {
			let legacy_file_base = format!("{}_View-{}", view.slug, root_id);
			let legacy_dot = view_source_dir.join(format!("{legacy_file_base}.view.dot"));
			let legacy_svg = views_dir.join(format!("{legacy_file_base}.view.svg"));
			remove_if_exists(&legacy_dot)?;
			remove_if_exists(&legacy_svg)?;
		}
		let engine = graphviz_engine_for_view(view);
		let nodes = if root_ids.is_empty() {
			base_nodes.clone()
		} else {
			filter_nodes_by_explicit_roots(view, base_nodes.clone(), &root_ids)
		};
		if nodes.is_empty() && !is_everything_view(view) {
			continue;
		}
		if nodes.len() <= 1 && !is_everything_view(view) {
			continue;
		}
		let mut nodes_with_annotations = nodes.clone();
		include_annotations_for_view(model, &mut nodes_with_annotations, &nodes);
		let file_base = format!("{}_View", view.slug);
		let dot_path = view_source_dir.join(format!("{file_base}.view.dot"));
		let svg_path = views_dir.join(format!("{file_base}.view.svg"));

		let clusters = collect_boundary_clusters(&nodes_with_annotations);
		validate_view_connectivity(view, &nodes_with_annotations)?;
		let dot = build_dot(
			view,
			&nodes_with_annotations,
			&palette,
			&icon_glyphs,
			&registry.card_colors,
			&graphviz,
		);
		write_text(&dot_path, dot)?;
		render_svg(
			&dot_path,
			&svg_path,
			&graphviz,
			engine,
			&nodes_with_annotations,
			&icon_glyphs,
			&registry.card_colors,
			&clusters,
		)?;
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

/// Render both card markdown and relationship views (embedded registries; instructions root ignored).
pub fn render_all_with_instructions(
	model: &AuroraModel,
	output_dir: impl AsRef<Path>,
	_instructions_root: impl AsRef<Path>,
) -> Result<RenderSummary> {
	render_all(model, output_dir)
}

#[derive(Debug, Clone, Default)]
struct CardColor {
	fill: Option<String>,
	font: Option<String>,
}

#[derive(Debug, Clone)]
struct CardPalette {
	shapes: HashMap<String, String>,
	icons: HashMap<String, String>,
}

impl CardPalette {
	fn embedded() -> Result<Self> {
		Self::from_definitions(CardDefinition::get_all())
	}

	fn from_definitions(definitions: &[CardDefinition]) -> Result<Self> {
		let mut shapes = HashMap::new();
		let mut icons = HashMap::new();
		for definition in definitions {
			let card_type = normalize_card_type(definition.card_type);
			if card_type.is_empty() {
				continue;
			}
			if let Some(shape) = normalize_shape(definition.shape) {
				shapes.insert(card_type.clone(), shape);
			}
			let icon = definition.icon.trim();
			if !icon.is_empty() && !icon.eq_ignore_ascii_case("n/a") {
				icons.insert(card_type.clone(), icon.to_string());
			}
		}
		if shapes.is_empty() {
			return Err(AuroraError::InvalidInput {
				message: "failed to load card palette from registry data".to_string(),
			});
		}
		Ok(CardPalette { shapes, icons })
	}

	fn shape_for(&self, card_type: &str) -> Option<&str> {
		self.shapes.get(card_type).map(|value| value.as_str())
	}

	fn icon_glyphs(&self) -> HashMap<String, String> {
		let mut glyphs = HashMap::new();
		for (card_type, icon) in &self.icons {
			if let Some(glyph) = icons::icon_glyph(icon) {
				glyphs.insert(card_type.clone(), glyph.to_string());
			}
		}
		glyphs
	}
}
fn normalize_shape(value: &str) -> Option<String> {
	let trimmed = value.trim();
	if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("n/a") {
		return None;
	}
	let lower = trimmed.to_ascii_lowercase();
	let normalized = match lower.as_str() {
		"rounded box" => "box",
		"double-octagon" | "double octagon" => "doubleoctagon",
		"cluster (dashed)" => "cluster",
		_ => trimmed,
	};
	Some(normalized.to_string())
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
	root_card_types: Vec<CardTypeFilter>,
	card_types: Vec<CardTypeFilter>,
	include_all: bool,
}

impl ViewSpec {
	fn includes_card(&self, card: &Card) -> bool {
		self.include_all || self.card_types.iter().any(|filter| filter.matches(card))
	}
}

impl ViewRegistry {
	fn embedded() -> Result<Self> {
		let views = view_specs_from_definitions(ViewDefinition::get_all());
		if views.is_empty() {
			return Err(AuroraError::InvalidInput {
				message: "failed to load view registry from embedded definitions".to_string(),
			});
		}
		let card_colors = card_colors_from_definitions(CardDefinition::get_all());
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
				("concentrate".to_string(), "true".to_string()),
				("nodesep".to_string(), "0.3".to_string()),
				("ranksep".to_string(), "0.5".to_string()),
				("fontname".to_string(), "Inter".to_string()),
			]),
			node_defaults: BTreeMap::from([
				("style".to_string(), "filled".to_string()),
				("shape".to_string(), "box".to_string()),
				("fontname".to_string(), "Inter".to_string()),
				("fontsize".to_string(), "11".to_string()),
				("color".to_string(), "#000000".to_string()),
				("fillcolor".to_string(), "#FFFFFF".to_string()),
				("fontcolor".to_string(), "#000000".to_string()),
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
				("color".to_string(), "#000000".to_string()),
				("penwidth".to_string(), "4".to_string()),
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

#[cfg(test)]
#[derive(Debug, Clone)]
struct CardTypeList {
	card_types: Vec<CardTypeFilter>,
	include_all: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CardTypeFilter {
	card_type: String,
	card_subtype: Option<String>,
}

impl CardTypeFilter {
	fn new(card_type: &str, card_subtype: Option<&str>) -> Self {
		CardTypeFilter {
			card_type: card_type.trim().to_string(),
			card_subtype: card_subtype
				.map(|value| value.trim().to_string())
				.filter(|value| !value.is_empty()),
		}
	}

	fn matches(&self, card: &Card) -> bool {
		if !card.card_type.eq_ignore_ascii_case(&self.card_type) {
			return false;
		}
		match &self.card_subtype {
			Some(subtype) => card
				.card_subtype
				.as_deref()
				.map(|value| value.trim())
				.is_some_and(|value| value.eq_ignore_ascii_case(subtype)),
			None => true,
		}
	}
}

#[derive(Debug, Clone)]
struct EdgeSpec {
	source: String,
	target: String,
	label: String,
	dotted: bool,
}

#[derive(Debug, Clone)]
struct BoundaryCluster {
	id: String,
	label: String,
	members: Vec<String>,
}

fn normalize_card_type(value: &str) -> String {
	let trimmed = value.trim().trim_matches('`');
	let base = match trimmed.split_once('(') {
		Some((head, _)) => head.trim(),
		None => trimmed,
	};
	base.trim().to_string()
}

#[cfg(test)]
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

fn view_specs_from_definitions(definitions: &[ViewDefinition]) -> Vec<ViewSpec> {
	definitions
		.iter()
		.filter_map(|definition| {
			let name = definition.name.trim();
			if name.is_empty() {
				return None;
			}
			let name = name.to_string();
			let slug = slugify_view_name(&name);
			let root_card_types = card_type_filters_from_list(definition.root_card_types);
			let mut card_types = card_type_filters_from_list(definition.include_card_types);
			card_types.extend(card_type_filters_from_list(definition.optional_card_types));
			let include_all = definition
				.include_card_types
				.iter()
				.chain(definition.optional_card_types.iter())
				.any(|value| is_all_card_types_marker(value));
			Some(ViewSpec {
				name,
				slug,
				root_card_types,
				card_types,
				include_all,
			})
		})
		.collect()
}

fn card_type_filters_from_list(values: &[&'static str]) -> Vec<CardTypeFilter> {
	values
		.iter()
		.filter_map(|value| parse_card_type_filter(value))
		.collect()
}

fn is_all_card_types_marker(value: &str) -> bool {
	value.trim().eq_ignore_ascii_case("all card types")
		|| value.trim().eq_ignore_ascii_case("all cards")
		|| value.trim() == "*"
}

#[cfg(test)]
fn parse_card_type_cell(cell: &str) -> CardTypeList {
	let mut include_all = false;
	let mut card_types = Vec::new();
	for part in cell
		.split(|ch| ch == ',' || ch == '\n' || ch == ';')
		.map(|segment| segment.trim())
	{
		if part.is_empty() || part == "-" {
			continue;
		}
		if part.eq_ignore_ascii_case("none") || part.eq_ignore_ascii_case("n/a") {
			continue;
		}
		if part.eq_ignore_ascii_case("All card types") {
			include_all = true;
			continue;
		}
		if let Some(filter) = parse_card_type_filter(part) {
			card_types.push(filter);
		}
	}
	CardTypeList {
		card_types,
		include_all,
	}
}

fn parse_card_type_filter(value: &str) -> Option<CardTypeFilter> {
	let trimmed = value.trim().trim_matches('`');
	if trimmed.is_empty() {
		return None;
	}
	let (card_type, subtype) = match trimmed.split_once('(') {
		Some((head, rest)) => {
			let subtype = rest.trim().trim_end_matches(')').trim();
			(head.trim(), Some(subtype))
		}
		None => (trimmed, None),
	};
	if card_type.is_empty() {
		return None;
	}
	Some(CardTypeFilter::new(card_type, subtype))
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

fn card_colors_from_definitions(definitions: &[CardDefinition]) -> HashMap<String, CardColor> {
	let mut colors = HashMap::new();
	for definition in definitions {
		let card_type = normalize_card_type(definition.card_type);
		if card_type.is_empty() {
			continue;
		}
		let fill = definition.fill.trim();
		let font = definition.color.trim();
		let entry = colors.entry(card_type).or_insert_with(CardColor::default);
		if !fill.is_empty() {
			entry.fill = Some(fill.to_string());
		}
		if !font.is_empty() {
			entry.font = Some(font.to_string());
		}
	}
	colors
}

#[cfg(test)]
fn collect_view_nodes<'a>(model: &'a AuroraModel, view: &ViewSpec) -> BTreeMap<String, &'a Card> {
	let nodes = collect_view_base_nodes(model, view);
	let nodes = filter_nodes_by_roots(view, nodes);
	let mut with_annotations = nodes.clone();
	include_annotations_for_view(model, &mut with_annotations, &nodes);
	with_annotations
}

fn collect_view_base_nodes<'a>(
	model: &'a AuroraModel,
	view: &ViewSpec,
) -> BTreeMap<String, &'a Card> {
	let mut nodes = BTreeMap::new();
	for card in model.iter_cards() {
		if is_annotation(card) {
			continue;
		}
		if view.includes_card(card) {
			nodes.insert(card.id.clone(), card);
		}
	}
	nodes
}

fn collect_view_root_ids(view: &ViewSpec, nodes: &BTreeMap<String, &Card>) -> Vec<String> {
	let node_ids = collect_diagram_node_ids(nodes);
	if node_ids.is_empty() {
		return Vec::new();
	}
	let edges = collect_edges(nodes);
	let incoming = collect_incoming_counts(&node_ids, &edges);
	let mut roots = collect_root_nodes_by_type(view, nodes);
	if roots.is_empty() {
		roots = collect_root_nodes(&node_ids, &incoming);
	}
	roots.sort();
	roots
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
		if view
			.root_card_types
			.iter()
			.any(|filter| filter.matches(card))
		{
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

#[cfg(test)]
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

fn filter_nodes_by_explicit_roots<'a>(
	_view: &ViewSpec,
	nodes: BTreeMap<String, &'a Card>,
	roots: &[String],
) -> BTreeMap<String, &'a Card> {
	if roots.is_empty() {
		return nodes;
	}
	let edges = collect_edges(&nodes);
	let reachable = collect_connected_nodes(roots, &edges);
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
	icons: &HashMap<String, String>,
	colors: &HashMap<String, CardColor>,
	config: &GraphvizConfig,
) -> String {
	let edges = collect_edges(nodes);
	build_dot_with_edges(view, nodes, &edges, palette, icons, colors, config)
}

fn build_dot_with_edges(
	view: &ViewSpec,
	nodes: &BTreeMap<String, &Card>,
	edges: &[EdgeSpec],
	palette: &CardPalette,
	icons: &HashMap<String, String>,
	colors: &HashMap<String, CardColor>,
	config: &GraphvizConfig,
) -> String {
	let mut lines = Vec::new();
	let direct_edges = if is_everything_view(view) {
		direct_edges_from_mission(nodes, edges)
	} else {
		None
	};
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
		let icon = icons.get(&card.card_type).map(String::as_str);
		attrs.insert("label".to_string(), node_label(card, icon));
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
		let mut styles = Vec::new();
		if edge.dotted {
			styles.push("dotted");
		}
		if let Some(ref direct_edges) = direct_edges {
			let key = (edge.source.clone(), edge.target.clone(), edge.label.clone());
			if !direct_edges.contains(&key) {
				styles.push("dashed");
			}
		}
		if !styles.is_empty() {
			attrs.insert("style".to_string(), styles.join(","));
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
			lines.push(format!("\t\tgraph [{}];", format_attributes(&attrs)));
		}
		for member in cluster.members {
			lines.push(format!("\t\t{};", dot_id(&member)));
		}
		lines.push("\t}".to_string());
	}

	lines.push("}".to_string());
	lines.join("\n")
}

fn is_everything_view(view: &ViewSpec) -> bool {
	view.slug == "Everything"
		|| view.name.eq_ignore_ascii_case("Everything View")
		|| view.name.eq_ignore_ascii_case("Entire Model")
		|| view.name.eq_ignore_ascii_case("Enitire Model")
		|| view.slug == "Entire_Model"
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
					dotted: target.card_type == "Note" || card.card_type == "Note",
				});
			}
		}
	}
	edges
}

fn direct_edges_from_mission(
	nodes: &BTreeMap<String, &Card>,
	edges: &[EdgeSpec],
) -> Option<BTreeSet<(String, String, String)>> {
	let mission_id = nodes
		.values()
		.find(|card| card.card_type == "Mission")
		.map(|card| card.id.clone())?;

	let mut adjacency: HashMap<String, Vec<&EdgeSpec>> = HashMap::new();
	for edge in edges {
		adjacency.entry(edge.source.clone()).or_default().push(edge);
	}

	let mut direct_edges = BTreeSet::new();
	let mut visited = BTreeSet::new();
	let mut queue = VecDeque::new();
	visited.insert(mission_id.clone());
	queue.push_back(mission_id);

	while let Some(node) = queue.pop_front() {
		if let Some(outgoing) = adjacency.get(&node) {
			for edge in outgoing {
				if visited.contains(&edge.target) {
					continue;
				}
				visited.insert(edge.target.clone());
				direct_edges.insert((edge.source.clone(), edge.target.clone(), edge.label.clone()));
				queue.push_back(edge.target.clone());
			}
		}
	}

	Some(direct_edges)
}

fn collect_boundary_clusters(nodes: &BTreeMap<String, &Card>) -> Vec<BoundaryCluster> {
	let adjacency = collect_view_adjacency(nodes);
	let mut clusters = Vec::new();
	for (id, card) in nodes {
		if card.card_type != "Boundary" {
			continue;
		}
		let recursive = boundary_recursive(card);
		let mut members = BTreeSet::new();
		for link in &card.links {
			if !link.relationship.eq_ignore_ascii_case("contains") {
				continue;
			}
			let target = match nodes.get(&link.target) {
				Some(target_card) if target_card.card_type != "Boundary" => target_card,
				_ => continue,
			};
			members.insert(target.id.clone());
			if recursive {
				include_descendants(&target.id, &adjacency, &mut members);
			}
		}
		let members: Vec<String> = members.into_iter().collect();
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

fn collect_view_adjacency(nodes: &BTreeMap<String, &Card>) -> HashMap<String, Vec<String>> {
	let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
	for (id, card) in nodes {
		if card.card_type == "Boundary" {
			continue;
		}
		for link in &card.links {
			let target = match nodes.get(&link.target) {
				Some(target_card) if target_card.card_type != "Boundary" => target_card,
				_ => continue,
			};
			adjacency
				.entry(id.clone())
				.or_default()
				.push(target.id.clone());
		}
	}
	adjacency
}

fn include_descendants(
	root: &str,
	adjacency: &HashMap<String, Vec<String>>,
	members: &mut BTreeSet<String>,
) {
	let mut visited = BTreeSet::new();
	let mut stack = vec![root.to_string()];
	while let Some(node) = stack.pop() {
		if !visited.insert(node.clone()) {
			continue;
		}
		if let Some(children) = adjacency.get(&node) {
			for child in children {
				if !visited.contains(child) {
					stack.push(child.clone());
				}
			}
		}
	}
	for id in visited {
		members.insert(id);
	}
}

fn boundary_recursive(card: &Card) -> bool {
	card.attributes
		.get("recursive")
		.and_then(|value| value.as_bool())
		.or_else(|| {
			card.attributes
				.get("inherited")
				.and_then(|value| value.as_bool())
		})
		.unwrap_or(false)
}

fn node_label(card: &Card, icon: Option<&str>) -> String {
	let mut lines = Vec::new();

	let mut type_line = String::new();
	type_line.push_str("<B>");
	type_line.push_str(&html_escape(&card.card_type));
	if let Some(subtype) = card.card_subtype.as_deref() {
		let subtype = subtype.trim();
		if !subtype.is_empty() {
			type_line.push(' ');
			type_line.push('(');
			type_line.push_str(&html_escape(subtype));
			type_line.push(')');
		}
	}
	type_line.push_str("</B>");
	lines.push(type_line);
	lines.push(format!("<B>{}</B>", html_escape(&card.id)));

	// Blank line between header and details.
	lines.push(String::new());

	let name = card.name.trim();
	let description = card.description.trim();
	let detail = if description.is_empty() {
		name.to_string()
	} else if name.is_empty() {
		description.to_string()
	} else {
		format!("{name} — {description}")
	};
	for line in wrap_text(&detail, 56) {
		lines.push(html_escape(&line));
	}

	let text_block = lines
		.into_iter()
		.map(|line| {
			if line.is_empty() {
				"&#160;".to_string()
			} else {
				line
			}
		})
		.collect::<Vec<_>>()
		.join("<BR/>");
	if let Some(icon) = icon {
		let icon_font_size = ICON_POINT_SIZE;
		return format!(
			"<<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"16\" CELLPADDING=\"0\"><TR><TD ALIGN=\"LEFT\" VALIGN=\"TOP\"><FONT POINT-SIZE=\"{icon_font_size}\">{}</FONT></TD><TD ALIGN=\"LEFT\" VALIGN=\"TOP\"><FONT>{}</FONT></TD></TR></TABLE>>",
			html_escape(icon),
			text_block
		);
	}
	format!("<<FONT>{}</FONT>>", text_block)
}

fn wrap_text(text: &str, max_len: usize) -> Vec<String> {
	let mut lines = Vec::new();
	let mut current = String::new();
	for word in text.split_whitespace() {
		let extra = if current.is_empty() {
			word.len()
		} else {
			word.len() + 1
		};
		if !current.is_empty() && current.len() + extra > max_len {
			lines.push(current);
			current = String::new();
		}
		if !current.is_empty() {
			current.push(' ');
		}
		current.push_str(word);
	}
	if !current.is_empty() {
		lines.push(current);
	}
	if lines.is_empty() {
		lines.push(text.trim().to_string());
	}
	lines
}

fn render_svg(
	dot_path: &Path,
	svg_path: &Path,
	config: &GraphvizConfig,
	engine: &str,
	nodes: &BTreeMap<String, &Card>,
	icons: &HashMap<String, String>,
	colors: &HashMap<String, CardColor>,
	clusters: &[BoundaryCluster],
) -> Result<()> {
	let command = config.dot_command();
	let layout = graphviz_plain::dot_plain_layout_with_engine(&command, dot_path, engine).map_err(
		|err| {
			if matches!(err, AuroraError::Io { ref source, .. } if source.kind() == ErrorKind::NotFound)
			{
				AuroraError::InvalidInput {
					message: format!(
						"command '{}' was not found. Install Graphviz (dot) to render views.",
						command
					),
				}
			} else {
				err
			}
		},
	)?;

	let svg = themed_svg::render_svg(&layout, nodes, icons, colors, clusters);
	write_text(svg_path, svg)
}

fn graphviz_engine_for_view(_view: &ViewSpec) -> &'static str {
	"dot"
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

fn mission_card(model: &AuroraModel) -> Option<&Card> {
	model.iter_cards().find(|card| card.card_type == "Mission")
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

fn remove_if_exists(path: &Path) -> Result<()> {
	if path.exists() {
		fs::remove_file(path).map_err(|err| AuroraError::io(path, err))?;
	}
	Ok(())
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
	use serde_json::json;
	use std::collections::BTreeMap;
	use std::path::{Path, PathBuf};
	use tempfile::TempDir;

	#[test]
	fn render_markdown_creates_card_files() {
		let (tmp, model) = sample_model();
		let output = tmp.path().join("render");
		let summary = render_markdown(&model, &output).expect("render should succeed");
		assert_eq!(summary.cards_written, 1);
		assert!(output.join("MIS-001-Provide_Default_Tooling.md").exists());
		assert!(output.join("MIS-001-Executive_Summary.md").exists());
	}

	#[test]
	fn render_views_emits_dot_and_svg_sources_only() {
		let (tmp, model) = sample_model();
		let output = tmp.path().join("views");
		let summary = render_views(&model, &output).expect("view render should succeed");
		assert_eq!(summary.views_written, 1);
		let requirements_md = output.join("Views/Requirements_View.view.md");
		assert!(!requirements_md.exists());
		let entire_md = output.join("Views/Entire_Model_View.view.md");
		assert!(!entire_md.exists());
		let legacy_dot = output.join("Views/source/Requirements.view.dot");
		let legacy_svg = output.join("Views/Requirements.view.svg");
		assert!(!legacy_dot.exists());
		assert!(!legacy_svg.exists());
		let legacy_per_root = output.join("Views/source/Requirements_View-MIS-001.view.dot");
		assert!(!legacy_per_root.exists());
		let dot_path = output.join("Views/source/Entire_Model_View.view.dot");
		assert!(dot_path.exists());
		assert!(
			output
				.join("Views/Entire_Model_View.view.svg")
				.exists()
		);
		let dot = std::fs::read_to_string(dot_path).expect("dot");
		assert!(dot.contains("<B>Mission</B>"));
		assert!(dot.contains("<B>MIS-001</B>"));
		assert!(dot.contains("POINT-SIZE=\"32\">🎯"));
		assert!(dot.contains("<BR/>"));
	}

	#[test]
	fn parse_view_table_supports_everything_view() -> Result<()> {
		let contents = r#"
# View Definitions

## Views

| View | Description | Root cards (selectable) | Included card types | Supporting card types | Notes |
| --- | --- | --- | --- | --- | --- |
| Everything View | Full-model overview. | Mission | All card types | None | Use for full-model rendering. |
"#;
		let views = parse_view_table(contents, Path::new("View_Definitions.md"))?;
		assert_eq!(views.len(), 1);
		let view = &views[0];
		assert_eq!(view.name, "Everything View");
		assert_eq!(view.slug, "Everything");
		assert!(view.include_all);
		assert!(
			view.root_card_types
				.iter()
				.any(|filter| filter.card_type == "Mission")
		);
		assert!(view.card_types.is_empty());
		Ok(())
	}

	#[test]
	fn everything_view_dashes_alternate_paths() {
		let mission = Card {
			schema: None,
			id: "MIS-001".into(),
			card_type: "Mission".into(),
			card_subtype: None,
			name: "Mission".into(),
			description: "Test mission".into(),
			status: None,
			links: vec![
				Link {
					target: "DRI-001".into(),
					relationship: "establishes".into(),
				},
				Link {
					target: "REQ-001".into(),
					relationship: "drives".into(),
				},
			],
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: Value::Null,
			source_path: None,
			extra: BTreeMap::new(),
		};
		let driver = Card {
			schema: None,
			id: "DRI-001".into(),
			card_type: "Driver".into(),
			card_subtype: None,
			name: "Driver".into(),
			description: "Test driver".into(),
			status: None,
			links: vec![Link {
				target: "REQ-001".into(),
				relationship: "drives".into(),
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
		let requirement = Card {
			schema: None,
			id: "REQ-001".into(),
			card_type: "Requirement".into(),
			card_subtype: None,
			name: "Requirement".into(),
			description: "Test requirement".into(),
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

		let mut nodes = BTreeMap::new();
		nodes.insert(mission.id.clone(), &mission);
		nodes.insert(driver.id.clone(), &driver);
		nodes.insert(requirement.id.clone(), &requirement);

		let view = ViewSpec {
			name: "Everything View".into(),
			slug: "Everything".into(),
			root_card_types: Vec::new(),
			card_types: Vec::new(),
			include_all: true,
		};
		let palette = CardPalette {
			shapes: std::collections::HashMap::new(),
			icons: std::collections::HashMap::new(),
		};
		let icons = std::collections::HashMap::new();
		let colors = std::collections::HashMap::new();
		let config = GraphvizConfig::default();

		let dot = build_dot(&view, &nodes, &palette, &icons, &colors, &config);
		assert!(dot.contains("\"DRI-001\" -> \"REQ-001\" [label=\"drives\", style=\"dashed\"];"));
		assert!(dot.contains("\"MIS-001\" -> \"REQ-001\" [label=\"drives\"];"));
	}

	#[test]
	fn boundary_cluster_uses_graph_attributes() {
		let actor = Card {
			schema: None,
			id: "ACT-001".into(),
			card_type: "Actor".into(),
			card_subtype: None,
			name: "Architect".into(),
			description: "Test actor".into(),
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
		let boundary = Card {
			schema: None,
			id: "BND-001".into(),
			card_type: "Boundary".into(),
			card_subtype: Some("Trust".into()),
			name: "Untrusted".into(),
			description: "Test boundary".into(),
			status: None,
			links: vec![Link {
				target: "ACT-001".into(),
				relationship: "contains".into(),
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
		let mut nodes = BTreeMap::new();
		nodes.insert(actor.id.clone(), &actor);
		nodes.insert(boundary.id.clone(), &boundary);
		let view = ViewSpec {
			name: "Use Case".into(),
			slug: "Use_Case".into(),
			root_card_types: Vec::new(),
			card_types: Vec::new(),
			include_all: false,
		};
		let palette = CardPalette {
			shapes: std::collections::HashMap::new(),
			icons: std::collections::HashMap::new(),
		};
		let icons = std::collections::HashMap::new();
		let colors = std::collections::HashMap::new();
		let config = GraphvizConfig::default();

		let dot = build_dot(&view, &nodes, &palette, &icons, &colors, &config);
		assert!(dot.contains("subgraph \"cluster_BND-001\" {"));
		assert!(dot.contains("\t\tgraph ["));
		assert!(dot.contains("label=\"Untrusted (BND-001)\""));
	}

	#[test]
	fn boundary_recursive_includes_descendants() {
		let parent = Card {
			schema: None,
			id: "COM-100".into(),
			card_type: "Component".into(),
			card_subtype: None,
			name: "Parent".into(),
			description: "Parent node".into(),
			status: None,
			links: vec![Link {
				target: "COM-101".into(),
				relationship: "uses".into(),
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
		let child = Card {
			schema: None,
			id: "COM-101".into(),
			card_type: "Component".into(),
			card_subtype: None,
			name: "Child".into(),
			description: "Child node".into(),
			status: None,
			links: vec![Link {
				target: "COM-102".into(),
				relationship: "uses".into(),
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
		let grandchild = Card {
			schema: None,
			id: "COM-102".into(),
			card_type: "Component".into(),
			card_subtype: None,
			name: "Grandchild".into(),
			description: "Grandchild node".into(),
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
		let boundary = Card {
			schema: None,
			id: "BND-010".into(),
			card_type: "Boundary".into(),
			card_subtype: None,
			name: "Recursive".into(),
			description: "Recursive boundary".into(),
			status: None,
			links: vec![Link {
				target: "COM-100".into(),
				relationship: "contains".into(),
			}],
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: Vec::new(),
			},
			attributes: json!({"recursive": true}),
			source_path: None,
			extra: BTreeMap::new(),
		};
		let mut nodes = BTreeMap::new();
		nodes.insert(parent.id.clone(), &parent);
		nodes.insert(child.id.clone(), &child);
		nodes.insert(grandchild.id.clone(), &grandchild);
		nodes.insert(boundary.id.clone(), &boundary);
		let clusters = collect_boundary_clusters(&nodes);
		assert_eq!(clusters.len(), 1);
		let members = clusters[0]
			.members
			.iter()
			.cloned()
			.collect::<BTreeSet<String>>();
		assert!(members.contains("COM-100"));
		assert!(members.contains("COM-101"));
		assert!(members.contains("COM-102"));
	}

	#[test]
	fn boundary_non_recursive_only_contains_direct_targets() {
		let parent = Card {
			schema: None,
			id: "COM-200".into(),
			card_type: "Component".into(),
			card_subtype: None,
			name: "Parent".into(),
			description: "Parent node".into(),
			status: None,
			links: vec![Link {
				target: "COM-201".into(),
				relationship: "uses".into(),
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
		let child = Card {
			schema: None,
			id: "COM-201".into(),
			card_type: "Component".into(),
			card_subtype: None,
			name: "Child".into(),
			description: "Child node".into(),
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
		let boundary = Card {
			schema: None,
			id: "BND-020".into(),
			card_type: "Boundary".into(),
			card_subtype: None,
			name: "Direct".into(),
			description: "Direct boundary".into(),
			status: None,
			links: vec![Link {
				target: "COM-200".into(),
				relationship: "contains".into(),
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
		let mut nodes = BTreeMap::new();
		nodes.insert(parent.id.clone(), &parent);
		nodes.insert(child.id.clone(), &child);
		nodes.insert(boundary.id.clone(), &boundary);
		let clusters = collect_boundary_clusters(&nodes);
		assert_eq!(clusters.len(), 1);
		let members = clusters[0]
			.members
			.iter()
			.cloned()
			.collect::<BTreeSet<String>>();
		assert!(members.contains("COM-200"));
		assert!(!members.contains("COM-201"));
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
		let label = node_label(&card, Some("🧊"));
		assert_eq!(
			label,
			"<<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"16\" CELLPADDING=\"0\"><TR><TD ALIGN=\"LEFT\" VALIGN=\"TOP\"><FONT POINT-SIZE=\"32\">🧊</FONT></TD><TD ALIGN=\"LEFT\" VALIGN=\"TOP\"><FONT><B>Component (struct)</B><BR/><B>COM-009</B><BR/>&#160;<BR/>Render Node — Test card</FONT></TD></TR></TABLE>>"
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
		let label = node_label(&card, Some("🧊"));
		assert_eq!(
			label,
			"<<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"16\" CELLPADDING=\"0\"><TR><TD ALIGN=\"LEFT\" VALIGN=\"TOP\"><FONT POINT-SIZE=\"32\">🧊</FONT></TD><TD ALIGN=\"LEFT\" VALIGN=\"TOP\"><FONT><B>Component</B><BR/><B>COM-010</B><BR/>&#160;<BR/>Render View — Test card</FONT></TD></TR></TABLE>>"
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
		let card_types = vec![CardTypeFilter::new("Mission", None)];
		let view = ViewSpec {
			name: "Mission Only".into(),
			slug: "Mission_Only".into(),
			root_card_types: Vec::new(),
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
		let card_types = vec![
			CardTypeFilter::new("Mission", None),
			CardTypeFilter::new("Note", None),
		];
		let view = ViewSpec {
			name: "Mission and Note".into(),
			slug: "Mission_and_Note".into(),
			root_card_types: Vec::new(),
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
}
