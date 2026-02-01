//! SVG rendering helpers for Graphviz DOT JSON output.

use crate::render::{
	dot::BoundingBox,
	geometry::{GeometryError, Point},
};
use crate::render::{
	dot::{Diagram, DotError, Edge, Node, Object, Spline, Subgraph},
	icons::icon_glyph,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write;
use thiserror::Error;
use tracing::trace;

const SVG_SYMBOL_DEFS: &str = include_str!("SymbolDefs.txt");

/// SVG rendering defaults for DOT-derived diagrams.
#[derive(Debug, Clone)]
pub struct SvgRenderOptions {
	/// Padding, in pixels, added around the diagram bounds.
	pub padding: f32,
	/// Default font family when the diagram does not specify one.
	pub font_family: String,
	/// Default font size, in pixels, when the diagram does not specify one.
	pub font_size: f32,
	/// Default node stroke color.
	pub node_stroke: String,
	/// Default node fill color.
	pub node_fill: String,
	/// Default edge stroke color.
	pub edge_stroke: String,
	/// Default stroke width, in pixels, for node borders.
	pub node_stroke_width: f32,
	/// Default stroke width, in pixels, for edges.
	pub edge_stroke_width: f32,
	/// Default stroke width, in pixels, for boundary boxes.
	pub boundary_stroke_width: f32,
	/// Optional background fill color for the SVG canvas.
	pub background: Option<String>,
}

impl Default for SvgRenderOptions {
	fn default() -> Self {
		Self {
			padding: 16.0,
			font_family: "\"Noto Sans\", Roboto, Verdana, system-ui, sans-serif".to_string(),
			font_size: 16.0,
			node_stroke: "#FFF000000FFF".to_string(),
			node_fill: "#00000000".to_string(),
			edge_stroke: "#000000".to_string(),
			node_stroke_width: 2.0,
			edge_stroke_width: 2.0,
			boundary_stroke_width: 2.0,
			background: None,
		}
	}
}

/// A lightweight SVG document builder for diagram rendering.
pub struct SvgDocument {
	width: f32,
	height: f32,
	view_box: String,
	elements: Vec<String>,
}

impl SvgDocument {
	/// Create a new SVG document with the provided dimensions in pixels.
	pub fn new(width: f32, height: f32) -> Self {
		Self {
			width,
			height,
			view_box: format!("0 0 {} {}", format_f32(width), format_f32(height)),
			elements: Vec::new(),
		}
	}

	/// Push a raw SVG element into the document.
	pub fn push(&mut self, element: String) {
		self.elements.push(element);
	}

	/// Render the SVG document into a string.
	pub fn to_svg_string(&self) -> String {
		let mut output = String::new();
		let _ = write!(
			output,
			"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"{}\">\n",
			format_f32(self.width),
			format_f32(self.height),
			self.view_box
		);
		let _ = write!(output, "{}\n", SVG_SYMBOL_DEFS,);
		for element in &self.elements {
			output.push_str(element);
		}
		output.push_str("</svg>");
		output
	}
}

/// Renderer for DOT JSON diagrams into SVG markup.
#[derive(Debug, Clone)]
pub struct SvgRenderer {
	options: SvgRenderOptions,
}

impl SvgRenderer {
	/// Render a diagram into SVG using the default renderer options.
	pub fn render_svg(diagram: &Diagram) -> Result<String, SvgError> {
		SvgRenderer::new(SvgRenderOptions::default()).render_to_string(diagram)
	}

	/// Create a renderer with custom options.
	pub fn new(options: SvgRenderOptions) -> Self {
		Self { options }
	}

	/// Render the diagram into an SVG document.
	pub fn render(&self, diagram: &Diagram) -> Result<SvgDocument, SvgError> {
		let bounds = diagram.bounding_box()?;
		trace!("Diagram bounds: {:?}", bounds);
		let transform = Transform::new(
			bounds.expect("The diagram is boundless"),
			self.options.padding,
		);
		let font_style = TextStyle::from_diagram(diagram, &self.options);

		let mut document = SvgDocument::new(transform.width, transform.height);
		if let Some(color) = &self.options.background {
			document.push(render_background(&transform, color));
		}
		trace!("New document created");

		for object in &diagram.objects {
			trace!("Rendering object: {:?}", object);
			if let Object::Subgraph(subgraph) = object {
				render_subgraph(
					subgraph,
					&transform,
					&self.options,
					&font_style,
					&mut document,
				)?;
			}
		}
		trace!("Subgraphs rendered");

		for edge in &diagram.edges {
			render_edge(edge, &transform, &self.options, &font_style, &mut document)?;
		}
		trace!("Edges rendered");

		for object in &diagram.objects {
			if let Object::Node(node) = object {
				render_node(node, &transform, &self.options, &font_style, &mut document)?;
			}
		}
		trace!(
			"\n---Document---\n{}\n---End Document---",
			document.to_svg_string()
		);
		Ok(document)
	}

	/// Render the diagram directly into SVG markup.
	pub fn render_to_string(&self, diagram: &Diagram) -> Result<String, SvgError> {
		Ok(self.render(diagram)?.to_svg_string())
	}
}

#[derive(Debug, Clone, Copy)]
struct Transform {
	xmin: f32,
	ymax: f32,
	padding: f32,
	width: f32,
	height: f32,
}

impl Transform {
	fn new(bounds: crate::render::dot::BoundingBox, padding: f32) -> Self {
		let width = (bounds.xmax - bounds.xmin) + padding * 2.0;
		let height = (bounds.ymax - bounds.ymin) + padding * 2.0;
		Self {
			xmin: bounds.xmin,
			ymax: bounds.ymax,
			padding,
			width,
			height,
		}
	}

	fn map_point(&self, point: &Point) -> Point {
		Point {
			x: point.x - self.xmin + self.padding,
			y: (self.ymax - point.y) + self.padding,
		}
	}
}

#[derive(Debug, Clone)]
struct TextStyle {
	font_family: String,
	font_size: f32,
	fill: String,
}

impl TextStyle {
	fn from_diagram(diagram: &Diagram, options: &SvgRenderOptions) -> Self {
		let family = diagram
			.fontname
			.clone()
			.unwrap_or_else(|| options.font_family.clone());
		let size = diagram
			.fontsize
			.as_deref()
			.and_then(|value| value.parse::<f32>().ok())
			.unwrap_or(options.font_size);
		Self {
			font_family: family,
			font_size: size,
			fill: "#111827".to_string(),
		}
	}
}

fn render_background(transform: &Transform, color: &str) -> String {
	// No need to render if Alpha = 0
	if color.len() == 9 && color.ends_with("00") {
		"".to_string()
	} else {
		format!(
			"<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\" />",
			format_f32(transform.width),
			format_f32(transform.height),
			escape_text(color)
		)
	}
}

fn render_node(
	node: &Node,
	transform: &Transform,
	options: &SvgRenderOptions,
	text_style: &TextStyle,
	document: &mut SvgDocument,
) -> Result<(), SvgError> {
	let pos = node
		.pos
		.as_deref()
		.ok_or_else(|| SvgError::MissingNodePosition(node.name.clone()))?;
	let center = transform.map_point(&Point::parse(pos)?);
	let width = parse_node_dimension(&node.width, &node.name, "width")?;
	let height = parse_node_dimension(&node.height, &node.name, "height")?;

	let shape = if let Some(shape) = &node.svg_shape {
		shape.clone()
	} else {
		"box".to_string()
	};

	let color = if let Some(color) = &node.color {
		color.clone()
	} else {
		options.node_stroke.clone()
	};
	let fill = if let Some(fill) = &node.fillcolor {
		fill.clone()
	} else {
		options.node_fill.clone()
	};

	let shape_style = ShapeStyle {
		stroke: color,
		fill,
		stroke_width: options.node_stroke_width,
	};

	let shape_svg = render_shape(&shape, &center, width, height, &shape_style, &node.icon);
	document.push(shape_svg);

	let label = node.label.clone().unwrap_or_else(|| node.name.clone());
	let lines = split_label(&label);
	for text in render_text_lines(&lines, &center, text_style, "middle") {
		document.push(text);
	}

	Ok(())
}

fn render_subgraph(
	subgraph: &Subgraph,
	transform: &Transform,
	options: &SvgRenderOptions,
	text_style: &TextStyle,
	document: &mut SvgDocument,
) -> Result<(), SvgError> {
	let mut bounds = subgraph.bounding_box()?;
	if bounds.is_none() {
		bounds = Some(BoundingBox {
			xmin: 0.0,
			ymin: 0.0,
			xmax: 100.0,
			ymax: 100.0,
		});
	}
	trace!("Subgraph bounds {:?}", bounds);
	let (x, y, width, height) = rect_from_bounds(transform, &bounds.expect("Out of bounds?"));
	let mut rect = format!(
		"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"",
		format_f32(x),
		format_f32(y),
		format_f32(width),
		format_f32(height),
		escape_text(&options.node_stroke),
		format_f32(options.boundary_stroke_width)
	);

	let _ = write!(rect, " stroke-dasharray=\"5 5\"");

	if let Some(label) = subgraph
		.label
		.clone()
		.or_else(|| attr_as_string(&subgraph.attrs, "label"))
	{
		rect.push_str(" />");
		document.push(rect);
		let anchor = Point {
			x: x + 6.0,
			y: y + text_style.font_size,
		};
		for text in render_text_lines(&split_label(&label), &anchor, text_style, "start") {
			document.push(text);
		}
		return Ok(());
	}
	if let Some(pos) = subgraph.pos.as_deref() {
		let label_point = transform.map_point(&Point::parse(pos)?);
		rect.push_str(" />");
		document.push(rect);
		for text in render_text_lines(
			&split_label(&subgraph.name),
			&label_point,
			text_style,
			"middle",
		) {
			document.push(text);
		}
		return Ok(());
	}

	rect.push_str(" />");
	document.push(rect);
	Ok(())
}

fn render_edge(
	edge: &Edge,
	transform: &Transform,
	options: &SvgRenderOptions,
	text_style: &TextStyle,
	document: &mut SvgDocument,
) -> Result<(), SvgError> {
	let pos = edge
		.pos
		.as_deref()
		.ok_or_else(|| SvgError::MissingEdgePosition(edge.gvid.to_string()))?;
	let splines = Edge::splines(pos)?;
	let stroke = &edge
		.color
		.clone()
		.unwrap_or_else(|| options.edge_stroke.clone());

	for spline in splines {
		let path_data = build_path_data(&spline, transform)?;
		let mut path = format!(
			"<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"",
			path_data,
			escape_text(&stroke),
			format_f32(options.edge_stroke_width)
		);
		if let Some(style) = &edge.style {
			if style == "dotted" {
				let _ = write!(path, " stroke-dasharray=\"2 2\"");
			}
		}
		path.push_str(" />");
		document.push(path);
	}

	if let (Some(label), Some(label_pos)) = (
		edge.label
			.clone()
			.or_else(|| attr_as_string(&edge.attrs, "label")),
		attr_as_string(&edge.attrs, "lp"),
	) {
		let label_point = transform.map_point(&Point::parse(&label_pos)?);
		for text in render_text_lines(&split_label(&label), &label_point, text_style, "middle") {
			document.push(text);
		}
	}

	Ok(())
}

fn build_path_data(spline: &Spline, transform: &Transform) -> Result<String, SvgError> {
	let pixels = spline_points(spline, transform);
	if pixels.len() < 2 {
		return Err(SvgError::InvalidEdgePoints(
			"Insufficient pixels".to_string(),
		));
	}

	let mut path = String::new();
	let start = &pixels[0];
	let _ = write!(path, "M {} {}", format_f32(start.x), format_f32(start.y));

	let mut index = 1;
	while index + 2 < pixels.len() {
		let c1 = &pixels[index];
		let c2 = &pixels[index + 1];
		let c3 = &pixels[index + 2];
		let _ = write!(
			path,
			" C {} {} {} {} {} {}",
			format_f32(c1.x),
			format_f32(c1.y),
			format_f32(c2.x),
			format_f32(c2.y),
			format_f32(c3.x),
			format_f32(c3.y)
		);
		index += 3;
	}

	while index < pixels.len() {
		let point = &pixels[index];
		let _ = write!(path, " L {} {}", format_f32(point.x), format_f32(point.y));
		index += 1;
	}

	Ok(path)
}

fn spline_points(spline: &Spline, transform: &Transform) -> Vec<Point> {
	let mut pixels: Vec<Point> = Vec::new();
	if let Some(tail) = &spline.tail {
		pixels.push(transform.map_point(tail));
	}
	for point in &spline.points {
		pixels.push(transform.map_point(point));
	}
	if let Some(head) = &spline.head {
		pixels.push(transform.map_point(head));
	}
	pixels
}

#[derive(Debug, Clone)]
struct ShapeStyle {
	stroke: String,
	fill: String,
	stroke_width: f32,
}

fn render_shape(
	shape: &str,
	center: &Point,
	width: f32,
	height: f32,
	style: &ShapeStyle,
	icon: &Option<String>,
) -> String {
	let mut node: String = format!(
		"<use href=\"#{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\" />",
		shape,
		format_f32(center.x - width / 2.0),
		format_f32(center.y - height / 2.0),
		format_f32(width),
		format_f32(height),
		escape_text(&style.fill),
		escape_text(&style.stroke),
		format_f32(style.stroke_width),
	);
	if let Some(icon_name) = icon {
		node.push_str(&format!(
			"\n<text x=\"{}\" y=\"{}\" font-size=\"48\">{}</text>",
			format_f32((center.x - width / 2.0) + 64.0),
			format_f32(center.y),
			icon_glyph(icon_name).unwrap_or_default(),
		));
	}
	node
}

fn rect_from_bounds(
	transform: &Transform,
	bounds: &crate::render::dot::BoundingBox,
) -> (f32, f32, f32, f32) {
	let top_left = transform.map_point(&Point {
		x: bounds.xmin,
		y: bounds.ymax,
	});
	let width = bounds.xmax - bounds.xmin;
	let height = bounds.ymax - bounds.ymin;
	(top_left.x, top_left.y, width, height)
}

fn render_text_lines(
	lines: &[String],
	center: &Point,
	style: &TextStyle,
	anchor: &str,
) -> Vec<String> {
	let mut rendered: Vec<String> = Vec::new();
	if lines.is_empty() {
		return rendered;
	}
	let line_height = style.font_size * 1.2;
	let total_height = line_height * (lines.len().saturating_sub(1) as f32);
	let start_y = center.y - total_height / 2.0;

	for (index, line) in lines.iter().enumerate() {
		let y = start_y + line_height * index as f32;
		let text = format!(
			"<text x=\"{}\" y=\"{}\" text-anchor=\"{}\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\" dominant-baseline=\"middle\">{}</text>",
			format_f32(center.x),
			format_f32(y),
			escape_text(anchor),
			escape_text(&style.font_family),
			format_f32(style.font_size),
			escape_text(&style.fill),
			escape_text(line)
		);
		rendered.push(text);
	}

	rendered
}

fn split_label(label: &str) -> Vec<String> {
	label
		.replace("<BR/>", "\n")
		.replace("<BR>", "\n")
		.replace("<br/>", "\n")
		.replace("<br>", "\n")
		.split('\n')
		.map(|line| line.trim().to_string())
		.filter(|line| !line.is_empty())
		.collect()
}

fn parse_node_dimension(value: &Option<String>, name: &str, axis: &str) -> Result<f32, SvgError> {
	let raw = value
		.as_deref()
		.ok_or_else(|| SvgError::MissingNodeSize(name.to_string()))?;
	let size = raw
		.parse::<f32>()
		.map_err(|_| SvgError::InvalidNodeSize(format!("{}:{}={}", name, axis, raw)))?;
	if size <= 0.0 {
		return Err(SvgError::InvalidNodeSize(format!(
			"{}:{}={}",
			name, axis, raw
		)));
	}
	Ok(size)
}

fn attr_as_string(attrs: &HashMap<String, Value>, key: &str) -> Option<String> {
	match attrs.get(key) {
		Some(Value::String(value)) => Some(value.clone()),
		Some(Value::Number(value)) => Some(value.to_string()),
		Some(Value::Bool(value)) => Some(value.to_string()),
		_ => None,
	}
}

fn escape_text(value: &str) -> String {
	value
		.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
		.replace('\'', "&apos;")
}

fn format_f32(value: f32) -> String {
	format!("{:.2}", value)
}

/// Errors that can occur while rendering DOT JSON into SVG.
#[derive(Debug, Error)]
pub enum SvgError {
	#[error("Missing node position for node: {0}")]
	MissingNodePosition(String),
	#[error("Missing node size for node: {0}")]
	MissingNodeSize(String),
	#[error("Invalid node size for node: {0}")]
	InvalidNodeSize(String),
	#[error("Missing edge position for edge: {0}")]
	MissingEdgePosition(String),
	#[error("Invalid edge pixels: {0}")]
	InvalidEdgePoints(String),
	#[error("Failed to render SVG: {0}")]
	RenderFailure(String),
	#[error("A DOT parsing error has occurred: {0}")]
	DotError(#[from] DotError),
	#[error("A geometry parsing error has occurred: {0}")]
	GeometryError(#[from] GeometryError),
}
