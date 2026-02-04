use crate::render::geometry::Point;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
	pub xmin: f32,
	pub ymax: f32,
	pub padding: f32,
	pub width: f32,
	pub height: f32,
}

impl Transform {
	pub fn from_dot(bounds: crate::render::dot::BoundingBox, padding: f32) -> Self {
		let width = (Self::dot_to_svg_units(bounds.xmax) - Self::dot_to_svg_units(bounds.xmin))
			+ padding * 2.0;
		let height = (Self::dot_to_svg_units(bounds.ymax) - Self::dot_to_svg_units(bounds.ymin))
			+ padding * 2.0;
		Self {
			xmin: Self::dot_to_svg_units(bounds.xmin),
			ymax: Self::dot_to_svg_units(bounds.ymax),
			padding,
			width,
			height,
		}
	}

	pub fn map_point(&self, point: &Point) -> Point {
		Point {
			x: point.x - self.xmin + self.padding,
			y: (self.ymax - point.y) + self.padding,
		}
	}

	pub fn format_f32(value: f32) -> String {
		format!("{:.2}", value)
	}

	fn dot_to_svg_units(value: f32) -> f32 {
		value
	}
}
