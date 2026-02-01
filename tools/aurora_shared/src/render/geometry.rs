use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Point {
	pub x: f32,
	pub y: f32,
}

impl Point {
	pub fn parse(value: &str) -> Result<Point, GeometryError> {
		let parts: Vec<&str> = value.split(',').collect();
		if parts.len() != 2 {
			return Err(GeometryError::InvalidPoint(value.to_string()));
		}
		let x = parts[0]
			.parse::<f32>()
			.map_err(|_| GeometryError::InvalidPoint(value.to_string()))?;
		let y = parts[1]
			.parse::<f32>()
			.map_err(|_| GeometryError::InvalidPoint(value.to_string()))?;
		Ok(Point { x, y })
	}
}

#[derive(Debug, Error)]
pub enum GeometryError {
	#[error("Invalid point: {0}")]
	InvalidPoint(String),
}
