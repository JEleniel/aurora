//! Geometry primitives for SVG rendering.

#[derive(Debug, Clone, Copy)]
pub struct PointF {
	pub x: f32,
	pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct RectI {
	pub x: i32,
	pub y: i32,
	pub w: i32,
	pub h: i32,
}

impl RectI {
	pub fn center(&self) -> PointF {
		PointF {
			x: (self.x as f32) + (self.w as f32) / 2.0,
			y: (self.y as f32) + (self.h as f32) / 2.0,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
	pub min_x: f32,
	pub min_y: f32,
	pub max_x: f32,
	pub max_y: f32,
}

impl Bounds {
	pub fn empty() -> Self {
		Self {
			min_x: f32::INFINITY,
			min_y: f32::INFINITY,
			max_x: f32::NEG_INFINITY,
			max_y: f32::NEG_INFINITY,
		}
	}

	pub fn union(mut self, other: Bounds) -> Self {
		self.min_x = self.min_x.min(other.min_x);
		self.min_y = self.min_y.min(other.min_y);
		self.max_x = self.max_x.max(other.max_x);
		self.max_y = self.max_y.max(other.max_y);
		self
	}

	pub fn union_point(self, p: PointF) -> Self {
		self.union(Bounds {
			min_x: p.x,
			min_y: p.y,
			max_x: p.x,
			max_y: p.y,
		})
	}

	pub fn union_rect_i(self, r: RectI) -> Self {
		self.union(Bounds {
			min_x: r.x as f32,
			min_y: r.y as f32,
			max_x: (r.x + r.w) as f32,
			max_y: (r.y + r.h) as f32,
		})
	}
}
