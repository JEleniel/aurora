//! SVG rendering for Aurora layout graphs.
//!
//! This module renders an Aurora [`Model`](crate::Model) using an existing [`Layout`](super::Layout)
//! into a standalone SVG string based on the `svgtemplate.txt` template.

#[cfg(test)]
use super::LayoutEdge;
use super::render_error::RenderError;
use super::{Layout, LayoutFamily};
use crate::registry::CardRegistry;
use crate::{Card, Model};
use std::collections::{HashMap, HashSet};
use std::path::Path;

// Limit the size to prevent runaway rendering in case of very large graphs.
const SVG_MAX_SIZE: u32 = 50 * 1024 * 1024; // 50 MiB, limit of many SVG renderers
const SVG_EXPORT_PPI: f64 = 300.0;

const SYMBOL_BASE_WIDTH_PX: i32 = 720;
const SYMBOL_BASE_HEIGHT_PX: i32 = 450;

/// Renders an Aurora model graph as SVG.
#[derive(Debug, Default, Clone, Copy)]
pub struct Svg;

/// Rendering configuration for SVG output.
#[derive(Debug, Clone)]
pub struct SvgConfig {
	/// Minimum space between nodes (both horizontally and vertically) in pixels.
	pub node_spacing_px: i32,
	/// Base font size in pixels.
	pub base_font_size_px: i32,
	/// How edges should be drawn.
	pub edge_style: EdgeStyle,
}

impl Default for SvgConfig {
	fn default() -> Self {
		// Default geometry uses a fixed 16px base and 1112x842 grid pitch for 720x450 symbols.
		let base_font_size_px = 16;
		Self {
			node_spacing_px: 392,
			base_font_size_px,
			edge_style: EdgeStyle::Orthogonal,
		}
	}
}

/// Styling choices for edge rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeStyle {
	/// Orthogonal polyline routing (right angles).
	Orthogonal,
	/// Smooth cubic Bezier edges built from the routed polyline.
	Curved,
}

impl Svg {
	/// Render a model+layout into an SVG string.
	pub fn render(
		model: &Model,
		layout: &Layout,
		card_registry: &CardRegistry,
		svg_template: &str,
		config: Option<SvgConfig>,
	) -> Result<String, RenderError> {
		let base_size: u32 = svg_template.len() as u32;

		let mut config = config.unwrap_or_default();
		if matches!(layout.family, Some(LayoutFamily::RadialSubtree)) {
			config.node_spacing_px = (config.node_spacing_px / 2).max(120);
		}
		let template_shape_ids = collect_template_shape_ids(svg_template);

		let cards_by_id = index_cards(model)?;
		let node_layouts = node::collect_node_layouts(layout, &cards_by_id, &config)?;
		let positioned = node::position_nodes(&node_layouts, layout.family, &config);

		let mut edges_svg = String::new();
		let mut edge_bounds: Vec<geom::Bounds> = Vec::new();
		let mut edge_points: Vec<geom::PointF> = Vec::new();
		let boundaries_svg = render_boundaries(&cards_by_id, &positioned, &config);
		let (note_edges_svg, note_shapes_svg, note_labels_svg, note_bounds, note_points) =
			render_note_callouts(&cards_by_id, &positioned, &config);
		edge_points.extend(note_points);
		let mut node_shapes_svg = String::new();
		let mut node_labels_svg = String::new();

		let rem_px = config.base_font_size_px.max(1);
		let _ = rem_px;
		let routed_edges = edge_router::route_edges(&positioned, layout.edges.as_slice(), &config)?;
		for route in &routed_edges {
			edge_points.extend(route.points.iter().copied());
			edge_bounds.push(route.bounds);
		}

		let mut edges_base_svg = String::new();
		let mut edges_overlay_svg = String::new();
		for route in &routed_edges {
			let layers = edge::render_edge_layers(route, config.edge_style);
			edges_base_svg.push_str(layers.base.as_str());
			edges_overlay_svg.push_str(layers.overlay.as_str());
		}
		let edge_jump_svg = edge::build_jump_overlay(routed_edges.as_slice());
		edges_svg.push_str(edges_base_svg.as_str());
		edges_svg.push_str(edge_jump_svg.as_str());
		edges_svg.push_str(edges_overlay_svg.as_str());

		for (id, node) in positioned.iter() {
			let card = cards_by_id
				.get(id.as_str())
				.ok_or_else(|| RenderError::SvgMissingNode(id.clone()))?;
			if base_size
				+ edges_svg.len() as u32
				+ node_shapes_svg.len() as u32
				+ node_labels_svg.len() as u32
				> SVG_MAX_SIZE
			{
				return Err(RenderError::SvgTooLarge);
			}
			let rendered =
				node::render_node(card, node, card_registry, &template_shape_ids, &config);
			node_shapes_svg.push_str(&rendered.shape);
			node_labels_svg.push_str(&rendered.labels);
		}

		let mut render_bboxes: Vec<geom::RectI> = positioned.values().map(|n| n.bbox).collect();
		render_bboxes.extend(note_bounds.iter().copied());
		let raw_viewbox = compute_viewbox(
			render_bboxes.into_iter(),
			&edge_bounds,
			&edge_points,
			&config,
		);
		let (viewbox, shift_x, shift_y) = normalize_viewbox_origin(raw_viewbox);

		let background = format!(
			"<style>@media print{{#aurora-bg{{display:none;}}}}</style><rect id=\"aurora-bg\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"fill:#ffffff;stroke:none;\" />",
			viewbox.x, viewbox.y, viewbox.w, viewbox.h
		);
		if base_size
			+ edges_svg.len() as u32
			+ note_edges_svg.len() as u32
			+ boundaries_svg.len() as u32
			+ note_shapes_svg.len() as u32
			+ note_labels_svg.len() as u32
			+ node_shapes_svg.len() as u32
			+ node_labels_svg.len() as u32
			+ background.len() as u32
			> SVG_MAX_SIZE
		{
			return Err(RenderError::SvgTooLarge);
		}

		let mut content = String::new();
		if !boundaries_svg.is_empty() {
			content.push_str(format!("<g id=\"boundaries\">{}</g>", boundaries_svg).as_str());
		}
		if !edges_svg.is_empty() || !note_edges_svg.is_empty() {
			content
				.push_str(format!("<g id=\"edges\">{}{}</g>", edges_svg, note_edges_svg).as_str());
		}
		if !node_shapes_svg.is_empty() || !note_shapes_svg.is_empty() {
			content.push_str(
				format!(
					"<g id=\"node-shapes\">{}{}</g>",
					node_shapes_svg, note_shapes_svg
				)
				.as_str(),
			);
		}
		if !node_labels_svg.is_empty() || !note_labels_svg.is_empty() {
			content.push_str(
				format!(
					"<g id=\"node-labels\">{}{}</g>",
					node_labels_svg, note_labels_svg
				)
				.as_str(),
			);
		}

		let mut drawing = String::new();
		drawing.push_str(background.as_str());
		if shift_x != 0 || shift_y != 0 {
			drawing.push_str(
				format!(
					"<g id=\"aurora-content\" transform=\"translate({} {})\">{}</g>",
					shift_x, shift_y, content
				)
				.as_str(),
			);
		} else {
			drawing.push_str(content.as_str());
		}

		if base_size + drawing.len() as u32 > SVG_MAX_SIZE {
			return Err(RenderError::SvgTooLarge);
		}

		let mut svg = fill_template(svg_template, &drawing, &viewbox)?;
		if svg.len() as u32 > SVG_MAX_SIZE {
			return Err(RenderError::SvgTooLarge);
		}
		svg = prune_unused_defs_groups(svg);
		svg = collapse_blank_lines(svg);
		if svg.len() as u32 > SVG_MAX_SIZE {
			return Err(RenderError::SvgTooLarge);
		}

		Ok(svg)
	}

	/// Render SVG and write it to the provided path.
	pub fn write_to_file(
		path: impl AsRef<Path>,
		model: &Model,
		layout: &Layout,
		card_registry: &CardRegistry,
		svg_template: &str,
		config: Option<SvgConfig>,
	) -> Result<(), RenderError> {
		let svg = Self::render(model, layout, card_registry, svg_template, config)?;
		std::fs::write(path, svg)?;
		Ok(())
	}
}

fn index_cards(model: &Model) -> Result<HashMap<String, &Card>, RenderError> {
	let mut cards_by_id: HashMap<String, &Card> = HashMap::new();
	for card in std::iter::once(&model.root_card).chain(model.cards.iter()) {
		if cards_by_id.insert(card.id.clone(), card).is_some() {
			return Err(RenderError::SvgDuplicateCardId(card.id.clone()));
		}
	}
	Ok(cards_by_id)
}

fn compute_viewbox(
	node_bboxes: impl Iterator<Item = geom::RectI>,
	edge_bounds: &[geom::Bounds],
	edge_points: &[geom::PointF],
	config: &SvgConfig,
) -> geom::RectI {
	let _ = config;
	let margin = 150;

	let mut bounds = geom::Bounds::empty();
	for bbox in node_bboxes {
		bounds = bounds.union_rect_i(bbox);
	}
	for b in edge_bounds {
		bounds = bounds.union(*b);
	}
	for p in edge_points {
		bounds = bounds.union_point(*p);
	}

	if !bounds.min_x.is_finite() {
		return geom::RectI {
			x: 0,
			y: 0,
			w: 1,
			h: 1,
		};
	}

	let min_x = (bounds.min_x.floor() as i32) - margin;
	let min_y = (bounds.min_y.floor() as i32) - margin;
	let max_x = (bounds.max_x.ceil() as i32) + margin;
	let max_y = (bounds.max_y.ceil() as i32) + margin;

	geom::RectI {
		x: min_x,
		y: min_y,
		w: (max_x - min_x).max(1),
		h: (max_y - min_y).max(1),
	}
}

fn normalize_viewbox_origin(viewbox: geom::RectI) -> (geom::RectI, i32, i32) {
	let shift_x = (-viewbox.x).max(0);
	let shift_y = (-viewbox.y).max(0);
	(
		geom::RectI {
			x: viewbox.x + shift_x,
			y: viewbox.y + shift_y,
			w: viewbox.w,
			h: viewbox.h,
		},
		shift_x,
		shift_y,
	)
}

fn grow_rect(r: geom::RectI, pad: i32) -> geom::RectI {
	let p = pad.max(0);
	geom::RectI {
		x: r.x - p,
		y: r.y - p,
		w: r.w + 2 * p,
		h: r.h + 2 * p,
	}
}

fn render_boundaries(
	cards_by_id: &HashMap<String, &Card>,
	positioned: &HashMap<String, node::PositionedNode>,
	config: &SvgConfig,
) -> String {
	let mut bounds_by_path: HashMap<String, geom::RectI> = HashMap::new();
	for (id, card) in cards_by_id {
		let Some(boundary) = card.boundary.as_ref().map(|value| value.trim()) else {
			continue;
		};
		if boundary.is_empty() {
			continue;
		}
		let Some(node) = positioned.get(id) else {
			continue;
		};
		let mut prefix = String::new();
		for (index, segment) in boundary
			.split('.')
			.map(str::trim)
			.filter(|segment| !segment.is_empty())
			.enumerate()
		{
			if index > 0 {
				prefix.push('.');
			}
			prefix.push_str(segment);
			bounds_by_path
				.entry(prefix.clone())
				.and_modify(|existing| *existing = union_rect(*existing, node.bbox))
				.or_insert(node.bbox);
		}
	}

	let mut items: Vec<(String, geom::RectI)> = bounds_by_path.into_iter().collect();
	items.sort_by(|left, right| {
		left.0
			.matches('.')
			.count()
			.cmp(&right.0.matches('.').count())
			.then_with(|| left.0.cmp(&right.0))
	});

	let pad = (config.base_font_size_px / 2).max(8);
	let mut out = String::new();
	for (path, rect) in items {
		let label = path.rsplit('.').next().unwrap_or(path.as_str()).trim();
		if label.is_empty() {
			continue;
		}
		let frame = grow_rect(rect, pad);
		let label_x = frame.x + (frame.w / 2);
		let label_y = frame.y + config.base_font_size_px + 2;
		out.push_str(&format!(
			"<g class=\"aurora-boundary\" data-boundary=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"fill:none;stroke:#6b7280;stroke-width:2;stroke-dasharray:12 8;\" /><text x=\"{}\" y=\"{}\" style=\"fill:#4b5563;stroke:none;font-size:{}px;text-anchor:middle;\">{}</text></g>",
			escape_attr(path.as_str()),
			frame.x,
			frame.y,
			frame.w,
			frame.h,
			label_x,
			label_y,
			config.base_font_size_px,
			node::escape_text(label)
		));
	}

	out
}

fn render_note_callouts(
	cards_by_id: &HashMap<String, &Card>,
	positioned: &HashMap<String, node::PositionedNode>,
	config: &SvgConfig,
) -> (String, String, String, Vec<geom::RectI>, Vec<geom::PointF>) {
	let mut ids: Vec<&str> = cards_by_id.keys().map(|id| id.as_str()).collect();
	ids.sort();

	let mut edges_svg = String::new();
	let mut shapes_svg = String::new();
	let mut labels_svg = String::new();
	let mut bounds: Vec<geom::RectI> = Vec::new();
	let mut points: Vec<geom::PointF> = Vec::new();

	let line_step_px = (config.base_font_size_px + 8).max(1) as f32;

	for id in ids {
		let Some(card) = cards_by_id.get(id) else {
			continue;
		};
		let Some(note_text) = card.notes.as_ref().map(|value| value.trim()) else {
			continue;
		};
		if note_text.is_empty() {
			continue;
		}

		let Some(node) = positioned.get(id) else {
			continue;
		};

		let note_rect = geom::RectI {
			x: node.bbox.x + node.bbox.w + (config.node_spacing_px / 2).max(80),
			y: node.bbox.y,
			w: SYMBOL_BASE_WIDTH_PX,
			h: SYMBOL_BASE_HEIGHT_PX,
		};
		bounds.push(note_rect);

		let edge_start = geom::PointF {
			x: (node.bbox.x + node.bbox.w) as f32,
			y: (node.bbox.y + (node.bbox.h / 2)) as f32,
		};
		let edge_end = geom::PointF {
			x: note_rect.x as f32,
			y: (note_rect.y + (note_rect.h / 2)) as f32,
		};
		points.push(edge_start);
		points.push(edge_end);

		edges_svg.push_str(&format!(
			"<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" style=\"fill:none;stroke:#6b7280;stroke-width:2;stroke-dasharray:8 8;\" />",
			edge_start.x, edge_start.y, edge_end.x, edge_end.y
		));

		shapes_svg.push_str(&format!(
			"<g class=\"aurora-note-shape\" transform=\"translate({}, {})\"><use href=\"#curly-braces\" style=\"fill:#ffffff;stroke:#6b7280;\" /></g>",
			note_rect.x,
			note_rect.y
		));

		let text_x = note_rect.x as f32 + 398.0;
		let start_y = note_rect.y as f32 + 100.2;
		for (line_index, line) in node::wrap_text(note_text, 67).iter().enumerate() {
			if line.trim().is_empty() {
				continue;
			}
			let y = start_y + (line_index as f32) * line_step_px;
			labels_svg.push_str(&format!(
				"<text x=\"{:.2}\" y=\"{:.2}\" style=\"fill:#4b5563;stroke:none;text-anchor:middle;font-size:{}px;\">{}</text>",
				text_x,
				y,
				config.base_font_size_px,
				node::escape_text(line)
			));
		}
	}

	(edges_svg, shapes_svg, labels_svg, bounds, points)
}

fn union_rect(a: geom::RectI, b: geom::RectI) -> geom::RectI {
	let min_x = a.x.min(b.x);
	let min_y = a.y.min(b.y);
	let max_x = (a.x + a.w).max(b.x + b.w);
	let max_y = (a.y + a.h).max(b.y + b.h);
	geom::RectI {
		x: min_x,
		y: min_y,
		w: (max_x - min_x).max(1),
		h: (max_y - min_y).max(1),
	}
}

fn escape_attr(value: &str) -> String {
	node::escape_text(value)
		.replace('"', "&quot;")
		.replace('\'', "&apos;")
}

fn collect_template_shape_ids(svg_template: &str) -> HashSet<String> {
	let mut ids = HashSet::new();
	let mut cursor = 0usize;

	while let Some(rel) = svg_template[cursor..].find("<g") {
		let group_start = cursor + rel;
		let next = svg_template[group_start + 2..].chars().next();
		if !matches!(
			next,
			Some(' ') | Some('\t') | Some('\n') | Some('\r') | Some('>')
		) {
			cursor = group_start + 2;
			continue;
		}

		let Some(group_end_rel) = svg_template[group_start..].find('>') else {
			break;
		};
		let group_end = group_start + group_end_rel;
		let group_tag = &svg_template[group_start..=group_end];
		if let Some(id) = extract_attribute_value(group_tag, "id")
			&& !id.starts_with("i-")
		{
			ids.insert(id);
		}

		cursor = group_end + 1;
	}

	ids.insert("rectangle".to_string());
	ids
}
#[cfg(test)]
fn normalized_slot_bias(index: usize, total: usize) -> f32 {
	if total <= 1 {
		return 0.0;
	}

	let center = (total as f32 - 1.0) / 2.0;
	let mut slots: Vec<usize> = (0..total).collect();
	slots.sort_by(|left, right| {
		let left_dist = (*left as f32 - center).abs();
		let right_dist = (*right as f32 - center).abs();
		left_dist
			.partial_cmp(&right_dist)
			.unwrap_or(std::cmp::Ordering::Equal)
			.then_with(|| left.cmp(right))
	});

	let slot_index = slots
		.get(index)
		.copied()
		.unwrap_or_else(|| total.saturating_sub(1));

	let lane_offset = slot_index as f32 - center;
	let max_lane_offset = (edge::LANE_COUNT - 1) as f32 / 2.0;
	if max_lane_offset <= 0.0 {
		return 0.0;
	}

	(lane_offset / max_lane_offset).clamp(-1.0, 1.0)
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeMergeMode {
	Source,
	Target,
	None,
}

#[cfg(test)]
fn edge_merge_mode(source_total: usize, target_total: usize) -> EdgeMergeMode {
	if source_total <= 1 && target_total <= 1 {
		return EdgeMergeMode::None;
	}
	if source_total > 1 && target_total <= 1 {
		return EdgeMergeMode::Source;
	}
	if target_total > 1 && source_total <= 1 {
		return EdgeMergeMode::Target;
	}

	if target_total >= source_total {
		EdgeMergeMode::Target
	} else {
		EdgeMergeMode::Source
	}
}

#[cfg(test)]
fn compute_source_slot_indexes(
	edges: &[LayoutEdge],
	positioned: &HashMap<String, node::PositionedNode>,
) -> HashMap<usize, usize> {
	let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
	for (index, edge) in edges.iter().enumerate() {
		groups.entry(edge.a.as_str()).or_default().push(index);
	}

	let mut slots: HashMap<usize, usize> = HashMap::new();
	for mut indexes in groups.into_values() {
		indexes.sort_by(|left, right| {
			let left_key = source_group_sort_key(edges, *left, positioned);
			let right_key = source_group_sort_key(edges, *right, positioned);
			left_key
				.target_x
				.cmp(&right_key.target_x)
				.then_with(|| left_key.target_y.cmp(&right_key.target_y))
				.then_with(|| left_key.target_id.cmp(right_key.target_id))
				.then_with(|| left.cmp(right))
		});

		for (slot, edge_index) in indexes.into_iter().enumerate() {
			slots.insert(edge_index, slot);
		}
	}

	slots
}

#[cfg(test)]
fn compute_target_slot_indexes(
	edges: &[LayoutEdge],
	positioned: &HashMap<String, node::PositionedNode>,
) -> HashMap<usize, usize> {
	let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
	for (index, edge) in edges.iter().enumerate() {
		groups.entry(edge.b.as_str()).or_default().push(index);
	}

	let mut slots: HashMap<usize, usize> = HashMap::new();
	for mut indexes in groups.into_values() {
		indexes.sort_by(|left, right| {
			let left_key = target_group_sort_key(edges, *left, positioned);
			let right_key = target_group_sort_key(edges, *right, positioned);
			left_key
				.source_x
				.cmp(&right_key.source_x)
				.then_with(|| left_key.source_y.cmp(&right_key.source_y))
				.then_with(|| left_key.source_id.cmp(right_key.source_id))
				.then_with(|| left.cmp(right))
		});

		for (slot, edge_index) in indexes.into_iter().enumerate() {
			slots.insert(edge_index, slot);
		}
	}

	slots
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
struct SourceGroupSortKey<'a> {
	target_x: i32,
	target_y: i32,
	target_id: &'a str,
}

#[cfg(test)]
fn source_group_sort_key<'a>(
	edges: &'a [LayoutEdge],
	index: usize,
	positioned: &HashMap<String, node::PositionedNode>,
) -> SourceGroupSortKey<'a> {
	let edge = &edges[index];
	let center = positioned
		.get(edge.b.as_str())
		.map(|node| node.bbox.center())
		.unwrap_or(geom::PointF { x: 0.0, y: 0.0 });
	SourceGroupSortKey {
		target_x: center.x.round() as i32,
		target_y: center.y.round() as i32,
		target_id: edge.b.as_str(),
	}
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
struct TargetGroupSortKey<'a> {
	source_x: i32,
	source_y: i32,
	source_id: &'a str,
}

#[cfg(test)]
fn target_group_sort_key<'a>(
	edges: &'a [LayoutEdge],
	index: usize,
	positioned: &HashMap<String, node::PositionedNode>,
) -> TargetGroupSortKey<'a> {
	let edge = &edges[index];
	let center = positioned
		.get(edge.a.as_str())
		.map(|node| node.bbox.center())
		.unwrap_or(geom::PointF { x: 0.0, y: 0.0 });
	TargetGroupSortKey {
		source_x: center.x.round() as i32,
		source_y: center.y.round() as i32,
		source_id: edge.a.as_str(),
	}
}

fn fill_template(
	template: &str,
	drawing: &str,
	viewbox: &geom::RectI,
) -> Result<String, RenderError> {
	let viewbox_str = format!("{} {} {} {}", viewbox.x, viewbox.y, viewbox.w, viewbox.h);
	let width_inches = pixels_to_inches_attr(viewbox.w);
	let height_inches = pixels_to_inches_attr(viewbox.h);
	let mut out = if template.contains("{{viewbox}}") {
		template.replace("{{viewbox}}", &viewbox_str)
	} else {
		template.to_string()
	};

	replace_or_insert_svg_attribute(&mut out, "viewBox", &viewbox_str)?;
	replace_or_insert_svg_attribute(&mut out, "width", width_inches.as_str())?;
	replace_or_insert_svg_attribute(&mut out, "height", height_inches.as_str())?;

	if out.contains("{{diagram}}") {
		return Ok(out.replace("{{diagram}}", drawing));
	}
	if out.contains("{{drawing}}") {
		return Ok(out.replace("{{drawing}}", drawing));
	}

	if let Some(defs_end) = out.find("</defs>") {
		let insert_at = defs_end + "</defs>".len();
		out.insert_str(insert_at, drawing);
		return Ok(out);
	}

	if let Some(svg_end) = out.rfind("</svg>") {
		out.insert_str(svg_end, drawing);
		return Ok(out);
	}

	Err(RenderError::SvgTemplateMissingDiagram)
}

fn pixels_to_inches_attr(pixels: i32) -> String {
	let inches = (pixels.max(1) as f64) / SVG_EXPORT_PPI;
	let mut numeric = format!("{inches:.3}");
	while numeric.ends_with('0') {
		numeric.pop();
	}
	if numeric.ends_with('.') {
		numeric.pop();
	}
	format!("{numeric}in")
}

fn replace_or_insert_svg_attribute(
	template: &mut String,
	attribute: &str,
	value: &str,
) -> Result<(), RenderError> {
	let Some(svg_start) = template.find("<svg") else {
		return Err(RenderError::SvgTemplateMissingViewbox);
	};
	let Some(svg_tag_end_rel) = template[svg_start..].find('>') else {
		return Err(RenderError::SvgTemplateMissingViewbox);
	};
	let svg_tag_end = svg_start + svg_tag_end_rel;
	let tag = &template[svg_start..svg_tag_end];
	let needle = format!("{}=\"", attribute);

	if let Some(index_rel) = tag.find(needle.as_str()) {
		let index = svg_start + index_rel + needle.len();
		let rest = &template[index..];
		if let Some(end_rel) = rest.find('"') {
			template.replace_range(index..index + end_rel, value);
			return Ok(());
		}
	}

	template.insert_str(
		svg_tag_end,
		format!(" {}=\"{}\"", attribute, value).as_str(),
	);
	Ok(())
}

fn prune_unused_defs_groups(mut svg: String) -> String {
	let Some(defs_start) = svg.find("<defs") else {
		return svg;
	};
	let Some(defs_open_end_rel) = svg[defs_start..].find('>') else {
		return svg;
	};
	let defs_content_start = defs_start + defs_open_end_rel + 1;
	let Some(defs_close_rel) = svg[defs_content_start..].find("</defs>") else {
		return svg;
	};
	let defs_content_end = defs_content_start + defs_close_rel;

	let referenced_ids = collect_referenced_ids(svg.as_str());
	if referenced_ids.is_empty() {
		return svg;
	}

	let defs_content = &svg[defs_content_start..defs_content_end];
	let pruned = prune_defs_content(defs_content, &referenced_ids);
	svg.replace_range(defs_content_start..defs_content_end, pruned.as_str());
	svg
}

fn collapse_blank_lines(text: String) -> String {
	let mut out = String::with_capacity(text.len());
	let mut previous_blank = false;

	for line in text.lines() {
		let is_blank = line.trim().is_empty();
		if is_blank {
			if previous_blank {
				continue;
			}
			previous_blank = true;
			out.push('\n');
			continue;
		}

		previous_blank = false;
		out.push_str(line);
		out.push('\n');
	}

	if !text.ends_with('\n') && out.ends_with('\n') {
		out.pop();
	}

	out
}

fn collect_referenced_ids(svg: &str) -> HashSet<String> {
	let mut referenced = HashSet::new();
	collect_ids_after_marker(svg, "href=\"#", '"', &mut referenced);
	collect_ids_after_marker(svg, "href='#", '\'', &mut referenced);
	collect_ids_after_marker(svg, "url(#", ')', &mut referenced);
	referenced
}

fn collect_ids_after_marker(text: &str, marker: &str, terminator: char, ids: &mut HashSet<String>) {
	let mut cursor = 0usize;
	while let Some(rel) = text[cursor..].find(marker) {
		let start = cursor + rel + marker.len();
		let mut id = String::new();
		for ch in text[start..].chars() {
			if ch == terminator {
				break;
			}
			if !is_svg_id_char(ch) {
				break;
			}
			id.push(ch);
		}
		if !id.is_empty() {
			ids.insert(id);
		}
		cursor = start;
	}
}

fn is_svg_id_char(ch: char) -> bool {
	ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.')
}

fn prune_defs_content(content: &str, referenced_ids: &HashSet<String>) -> String {
	let mut out = String::new();
	let mut cursor = 0usize;
	while let Some(group_start) = find_next_group_start(content, cursor) {
		out.push_str(&content[cursor..group_start]);
		let Some((group_end, group_id)) = parse_group_span(content, group_start) else {
			out.push_str(&content[group_start..]);
			return out;
		};
		let group = &content[group_start..group_end];
		let keep_group = group_id
			.as_ref()
			.is_none_or(|id| referenced_ids.contains(id))
			|| span_contains_referenced_id(group, referenced_ids);
		if keep_group {
			out.push_str(group);
		}
		cursor = group_end;
	}
	out.push_str(&content[cursor..]);
	out
}

fn find_next_group_start(content: &str, from: usize) -> Option<usize> {
	let mut cursor = from;
	while let Some(rel) = content[cursor..].find("<g") {
		let idx = cursor + rel;
		let next = content[idx + 2..].chars().next();
		if matches!(
			next,
			Some(' ') | Some('\t') | Some('\n') | Some('\r') | Some('>')
		) {
			return Some(idx);
		}
		cursor = idx + 2;
	}
	None
}

fn parse_group_span(content: &str, group_start: usize) -> Option<(usize, Option<String>)> {
	let open_end = content[group_start..].find('>')? + group_start;
	let open_tag = &content[group_start..=open_end];
	let group_id = extract_attribute_value(open_tag, "id");
	if open_tag.trim_end().ends_with("/>") {
		return Some((open_end + 1, group_id));
	}

	let mut depth = 1usize;
	let mut cursor = open_end + 1;
	while depth > 0 {
		let next_open = find_next_group_start(content, cursor);
		let next_close = content[cursor..].find("</g>").map(|rel| cursor + rel);
		match (next_open, next_close) {
			(Some(open_idx), Some(close_idx)) if open_idx < close_idx => {
				let nested_end = content[open_idx..].find('>')? + open_idx;
				let nested_tag = &content[open_idx..=nested_end];
				if !nested_tag.trim_end().ends_with("/>") {
					depth += 1;
				}
				cursor = nested_end + 1;
			}
			(_, Some(close_idx)) => {
				depth = depth.saturating_sub(1);
				cursor = close_idx + "</g>".len();
			}
			_ => return None,
		}
	}

	Some((cursor, group_id))
}

fn extract_attribute_value(tag: &str, attribute: &str) -> Option<String> {
	for quote in ['"', '\''] {
		let needle = format!("{}={}", attribute, quote);
		if let Some(start_rel) = tag.find(needle.as_str()) {
			let start = start_rel + needle.len();
			let rest = &tag[start..];
			if let Some(end_rel) = rest.find(quote) {
				let value = &rest[..end_rel];
				if !value.is_empty() {
					return Some(value.to_string());
				}
			}
		}
	}
	None
}

fn span_contains_referenced_id(span: &str, referenced_ids: &HashSet<String>) -> bool {
	let mut cursor = 0usize;
	while let Some(rel) = span[cursor..].find("id=") {
		let idx = cursor + rel;
		let rest = &span[idx..];
		let Some(quote) = rest.chars().nth(3) else {
			break;
		};
		if quote != '"' && quote != '\'' {
			cursor = idx + 3;
			continue;
		}
		let value_start = idx + 4;
		if let Some(value_end_rel) = span[value_start..].find(quote) {
			let value = &span[value_start..value_start + value_end_rel];
			if referenced_ids.contains(value) {
				return true;
			}
			cursor = value_start + value_end_rel + 1;
		} else {
			break;
		}
	}
	false
}

mod edge;
mod edge_router;
mod edge_router_graph;
mod edge_router_unicode;
mod edge_router_utils;

pub use edge::Route;
mod geom;
mod node;

#[cfg(test)]
mod tests {
	use super::node;
	use super::{EdgeMergeMode, RenderError, edge_merge_mode, fill_template, geom};
	use crate::{Attributes, AuditLog, Card, Link, Model};
	use std::collections::HashMap;
	use std::path::PathBuf;

	fn read_testdata(rel_path: &str) -> String {
		let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("src")
			.join("testdata")
			.join(rel_path);
		std::fs::read_to_string(&path)
			.unwrap_or_else(|e| panic!("failed to read testdata file {}: {e}", path.display()))
	}

	#[test]
	fn wrap_preserves_newlines() {
		let lines = node::wrap_text("a b c\n\n1 2 3", 4);
		assert_eq!(lines[0], "a b");
		assert_eq!(lines[1], "c");
		assert_eq!(lines[2], "");
		assert_eq!(lines[3], "1 2");
		assert_eq!(lines[4], "3");
	}

	#[test]
	fn escape_xml_text() {
		assert_eq!(node::escape_text("a&b<c>d"), "a&amp;b&lt;c&gt;d");
	}

	#[test]
	fn template_replaces_diagram_placeholder() {
		let tpl = "<svg width=\"1600\" height=\"400\" viewBox=\"{{viewbox}}\">{{diagram}}</svg>";
		let vb = geom::RectI {
			x: 1,
			y: 2,
			w: 3300,
			h: 2400,
		};
		let out = fill_template(tpl, "<g/>", &vb).expect("template should render");
		assert!(out.contains("viewBox=\"1 2 3300 2400\""));
		assert!(out.contains("width=\"11in\""));
		assert!(out.contains("height=\"8in\""));
		assert!(out.contains("<g/>"));
	}

	#[test]
	fn template_replaces_drawing_placeholder() {
		let tpl = "<svg viewBox=\"{{viewbox}}\">{{drawing}}</svg>";
		let vb = geom::RectI {
			x: 0,
			y: 0,
			w: 10,
			h: 10,
		};
		let out = fill_template(tpl, "X", &vb).expect("template should render");
		assert!(out.contains("X"));
	}

	#[test]
	fn template_missing_placeholders_errors() {
		let vb = geom::RectI {
			x: 0,
			y: 0,
			w: 1,
			h: 1,
		};
		let err = fill_template("nope", "X", &vb).unwrap_err();
		assert!(matches!(err, RenderError::SvgTemplateMissingViewbox));
	}

	#[test]
	fn prune_unused_defs_groups_removes_unreferenced_group() {
		let svg = "<svg><defs><g id=\"keep\"><path /></g><g id=\"drop\"><path /></g></defs><use href=\"#keep\" /></svg>";
		let pruned = super::prune_unused_defs_groups(svg.to_string());
		assert!(pruned.contains("id=\"keep\""));
		assert!(!pruned.contains("id=\"drop\""));
	}

	#[test]
	fn prune_unused_defs_groups_keeps_parent_when_child_is_referenced() {
		let svg = "<svg><defs><g id=\"parent\"><g id=\"child\"><path /></g></g></defs><use href=\"#child\" /></svg>";
		let pruned = super::prune_unused_defs_groups(svg.to_string());
		assert!(pruned.contains("id=\"parent\""));
		assert!(pruned.contains("id=\"child\""));
	}

	#[test]
	fn render_includes_screen_background_and_no_text_stroke() {
		let model_configuration = read_testdata("modelconfiguration/svg_render_registry.json");
		let view_configuration =
			read_testdata("modelconfiguration/svg_render_viewconfiguration.json");
		let registry = crate::registry::CardRegistry::try_new_from_configurations(
			&model_configuration,
			&view_configuration,
		)
		.expect("registry");
		let svg_template = read_testdata("svg/template_minimal.svg");

		let root = Card {
			schema: None,
			id: "MIS-001".to_string(),
			card_type: "Mission".to_string(),
			card_subtype: None,
			name: "Test Mission".to_string(),
			description: "desc".to_string(),
			version: Some("1.0.0".to_string()),
			status: None,
			boundary: None,
			notes: None,
			icon: None,
			attributes: Attributes::new(),
			external_references: Vec::new(),
			links: vec![Link {
				target: "C-001".to_string(),
				relationship: "rel".to_string(),
			}],
			source_path: PathBuf::from("MIS-001.json"),
			validation_errors: Vec::new(),
			validation_warnings: Vec::new(),
		};
		let child = Card {
			schema: None,
			id: "C-001".to_string(),
			card_type: "Activity".to_string(),
			card_subtype: None,
			name: "Child".to_string(),
			description: "desc".to_string(),
			version: Some("1.0.0".to_string()),
			status: None,
			boundary: None,
			notes: None,
			icon: None,
			attributes: Attributes::new(),
			external_references: Vec::new(),
			links: Vec::new(),
			source_path: PathBuf::from("C-001.json"),
			validation_errors: Vec::new(),
			validation_warnings: Vec::new(),
		};
		let model = Model {
			root_card: root,
			cards: vec![child],
			audit_log: AuditLog {
				schema: None,
				history: Vec::new(),
				source_path: PathBuf::from("AuditLog.ndjson"),
				validation_errors: Vec::new(),
			},
			model_home: PathBuf::from("model"),
			mission_home: PathBuf::from("mission"),
		};

		let mut nodes = HashMap::new();
		nodes.insert(
			"MIS-001".to_string(),
			crate::render::LayoutNode {
				id: "MIS-001".to_string(),
				x: 0,
				y: 0,
			},
		);
		nodes.insert(
			"C-001".to_string(),
			crate::render::LayoutNode {
				id: "C-001".to_string(),
				x: 1,
				y: 0,
			},
		);
		let layout = crate::render::Layout {
			family: None,
			nodes,
			edges: vec![crate::render::LayoutEdge {
				a: "MIS-001".to_string(),
				b: "C-001".to_string(),
			}],
		};

		let svg = super::Svg::render(&model, &layout, &registry, svg_template.as_str(), None)
			.expect("svg render");
		assert!(!svg.contains("fill:#00000000;stroke:#000000"));
		assert!(svg.contains("id=\"aurora-bg\""));
		assert!(svg.contains("id=\"aurora-bg\" x=\"0\" y=\"0\""));
		assert!(svg.contains("viewBox=\"0 0 "));
		assert!(svg.contains("stroke:none"));
		assert!(!svg.contains("dominant-baseline:middle"));
		assert!(svg.contains("id=\"aurora-content\" transform=\"translate("));
	}

	#[test]
	fn normalize_viewbox_origin_shifts_negative_coordinates_to_zero() {
		let original = geom::RectI {
			x: -16,
			y: -32,
			w: 15712,
			h: 2448,
		};

		let (normalized, shift_x, shift_y) = super::normalize_viewbox_origin(original);

		assert_eq!(normalized.x, 0);
		assert_eq!(normalized.y, 0);
		assert_eq!(normalized.w, original.w);
		assert_eq!(normalized.h, original.h);
		assert_eq!(shift_x, 16);
		assert_eq!(shift_y, 32);
	}

	#[test]
	fn collect_template_shape_ids_uses_template_groups() {
		let template =
			r#"<svg><defs><g id="rectangle" /><g id="hexagon" /><g id="i-wrench" /></defs></svg>"#;
		let ids = super::collect_template_shape_ids(template);
		assert!(ids.contains("rectangle"));
		assert!(ids.contains("hexagon"));
		assert!(!ids.contains("i-wrench"));
	}

	#[test]
	fn outgoing_edges_use_distinct_source_biases() {
		let edges = vec![
			super::LayoutEdge {
				a: "SRC-001".to_string(),
				b: "TGT-001".to_string(),
			},
			super::LayoutEdge {
				a: "SRC-001".to_string(),
				b: "TGT-002".to_string(),
			},
		];

		let mut positioned = HashMap::new();
		for id in ["SRC-001", "TGT-001", "TGT-002"] {
			positioned.insert(
				id.to_string(),
				node::PositionedNode {
					bbox: geom::RectI {
						x: 0,
						y: 0,
						w: 720,
						h: 450,
					},
					width_px: 720,
					height_px: 450,
					geom: node::NodeGeom {
						width_px: 720,
						height_px: 450,
						lines: Vec::new(),
						bold_line_index: None,
						description_start_index: 0,
					},
				},
			);
		}

		let slots = super::compute_source_slot_indexes(edges.as_slice(), &positioned);
		let bias_a = super::normalized_slot_bias(slots.get(&0).copied().unwrap_or(0), 2);
		let bias_b = super::normalized_slot_bias(slots.get(&1).copied().unwrap_or(0), 2);
		assert_ne!(bias_a, bias_b, "source biases should be distinct");
		assert!(
			bias_a.abs() < 0.2 && bias_b.abs() < 0.2,
			"two-edge source biases should remain near center lanes: ({bias_a}, {bias_b})"
		);
	}

	#[test]
	fn incoming_edges_use_distinct_target_biases() {
		let edges = vec![
			super::LayoutEdge {
				a: "SRC-001".to_string(),
				b: "TGT-001".to_string(),
			},
			super::LayoutEdge {
				a: "SRC-002".to_string(),
				b: "TGT-001".to_string(),
			},
		];

		let mut positioned = HashMap::new();
		for id in ["SRC-001", "SRC-002", "TGT-001"] {
			positioned.insert(
				id.to_string(),
				node::PositionedNode {
					bbox: geom::RectI {
						x: 0,
						y: 0,
						w: 720,
						h: 450,
					},
					width_px: 720,
					height_px: 450,
					geom: node::NodeGeom {
						width_px: 720,
						height_px: 450,
						lines: Vec::new(),
						bold_line_index: None,
						description_start_index: 0,
					},
				},
			);
		}

		let slots = super::compute_target_slot_indexes(edges.as_slice(), &positioned);
		let bias_a = super::normalized_slot_bias(slots.get(&0).copied().unwrap_or(0), 2);
		let bias_b = super::normalized_slot_bias(slots.get(&1).copied().unwrap_or(0), 2);
		assert_ne!(bias_a, bias_b, "target biases should be distinct");
		assert!(
			bias_a.abs() < 0.2 && bias_b.abs() < 0.2,
			"two-edge target biases should remain near center lanes: ({bias_a}, {bias_b})"
		);
	}

	#[test]
	fn edge_merge_mode_selects_one_merge_side() {
		assert_eq!(edge_merge_mode(6, 1), EdgeMergeMode::Source);
		assert_eq!(edge_merge_mode(1, 6), EdgeMergeMode::Target);
		assert_eq!(edge_merge_mode(6, 6), EdgeMergeMode::Target);
		assert_eq!(edge_merge_mode(3, 5), EdgeMergeMode::Target);
		assert_eq!(edge_merge_mode(5, 3), EdgeMergeMode::Source);
		assert_eq!(edge_merge_mode(1, 1), EdgeMergeMode::None);
	}
}
