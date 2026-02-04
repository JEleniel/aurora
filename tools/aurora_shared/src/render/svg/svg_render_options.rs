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
			font_family: "'Noto Sans', Roboto, Verdana, system-ui, sans-serif".to_string(),
			font_size: 12.0,
			node_stroke: "#FFFFFF".to_string(),
			node_fill: "#00000000".to_string(),
			edge_stroke: "#000000".to_string(),
			node_stroke_width: 2.0,
			edge_stroke_width: 2.0,
			boundary_stroke_width: 2.0,
			background: None,
		}
	}
}
