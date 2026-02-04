use crate::render::svg::transform::Transform;
use std::fmt::Write;
use tracing::trace;

const SVG_SYMBOL_DEFS: &str = include_str!("SymbolDefs.txt");

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
			view_box: format!(
				"0 0 {} {}",
				Transform::format_f32(width),
				Transform::format_f32(height)
			),
			elements: Vec::new(),
		}
	}

	/// Push a raw SVG element into the document.
	pub fn push(&mut self, element: String) {
		if element.contains("use\"") {
			trace!("Adding SVG element: {}", &element);
		}
		self.elements.push(element);
	}

	/// Render the SVG document into a string.
	pub fn to_svg_string(&self) -> String {
		let mut output = String::new();
		let _ = write!(
			output,
			"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"{}\">\n",
			Transform::format_f32(self.width),
			Transform::format_f32(self.height),
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
