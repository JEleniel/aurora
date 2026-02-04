use crate::render::svg::svg_render_options::SvgRenderOptions;

#[derive(Debug, Clone)]
pub struct TextStyle {
	pub font_family: String,
	pub font_size: f32,
	pub fill: String,
}

impl TextStyle {
	pub fn from_options(options: &SvgRenderOptions) -> Self {
		let family = options.font_family.clone();
		let size = options.font_size;
		Self {
			font_family: family,
			font_size: size,
			fill: "#FFFFFF".to_string(),
		}
	}
}
