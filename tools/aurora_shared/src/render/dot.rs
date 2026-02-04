use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::render::geometry::{GeometryError, Point};

/// A Graphviz DOT diagram
#[derive(Debug, Deserialize, Serialize)]
pub struct Diagram {
	/// The name of the graph
	pub name: String,
	/// Whether the graph is directed
	pub directed: bool,
	/// Whether the graph is strict
	pub strict: bool,
	/// Number of subgraphs
	#[serde(rename = "_subgraph_cnt")]
	pub subgraph_cnt: u32,
	/// Bounding Box
	pub bb: Option<String>,
	/// Graph direction
	pub rankdir: Option<String>,
	/// Font name
	pub fontname: Option<String>,
	/// Font size
	pub fontsize: Option<String>,
	/// Label
	pub label: Option<String>,
	/// Objects in the graph (nodes and subgraphs)
	#[serde(default)]
	pub objects: Vec<Object>,
	/// Edges in the graph
	#[serde(default)]
	pub edges: Vec<Edge>,
	/// Additional attributes
	#[serde(flatten)]
	pub attrs: HashMap<String, serde_json::Value>,
}

impl Diagram {
	pub fn bounding_box(&self) -> Result<Option<BoundingBox>, DotError> {
		BoundingBox::parse(&self.bb)
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BoundingBox {
	pub xmin: f32,
	pub ymin: f32,
	pub xmax: f32,
	pub ymax: f32,
}

impl BoundingBox {
	pub fn parse(value: &Option<String>) -> Result<Option<BoundingBox>, DotError> {
		if value.is_none() {
			return Ok(None);
		}
		let value = value.clone().unwrap();

		let parts: Vec<&str> = value.split(',').collect();
		if parts.len() != 4 {
			return Err(DotError::InvalidBoundingBox(value.to_string()));
		}

		let xmin = parts[0]
			.trim()
			.parse::<f32>()
			.map_err(|_| DotError::InvalidBoundingBox(value.to_string()))?;
		let ymin = parts[1]
			.trim()
			.parse::<f32>()
			.map_err(|_| DotError::InvalidBoundingBox(value.to_string()))?;
		let xmax = parts[2]
			.trim()
			.parse::<f32>()
			.map_err(|_| DotError::InvalidBoundingBox(value.to_string()))?;
		let ymax = parts[3]
			.trim()
			.parse::<f32>()
			.map_err(|_| DotError::InvalidBoundingBox(value.to_string()))?;

		Ok(Some(BoundingBox {
			xmin,
			ymin,
			xmax,
			ymax,
		}))
	}
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Object {
	Node(Node),
	Subgraph(Subgraph),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
	#[serde(rename = "_gvid")]
	pub gvid: u32,
	pub name: String,
	pub label: Option<String>,
	pub pos: Option<String>,
	pub width: Option<String>,
	pub height: Option<String>,
	pub color: Option<String>,
	pub fillcolor: Option<String>,
	pub svg_shape: Option<String>,
	pub icon: Option<String>,
	#[serde(flatten)]
	pub attrs: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Subgraph {
	#[serde(rename = "_gvid")]
	pub gvid: u32,
	pub name: String,
	pub bb: Option<String>,
	pub label: Option<String>,
	pub pos: Option<String>,
	pub nodes: Option<Vec<u32>>,
	pub edges: Option<Vec<u32>>,
	pub subgraphs: Option<Vec<u32>>,
	#[serde(flatten)]
	pub attrs: HashMap<String, serde_json::Value>,
}

impl Subgraph {
	pub fn bounding_box(&self) -> Result<Option<BoundingBox>, DotError> {
		BoundingBox::parse(&self.bb)
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Edge {
	pub tail: u32,
	pub head: u32,
	#[serde(rename = "_gvid")]
	pub gvid: u32,
	pub pos: Option<String>,
	pub label: Option<String>,
	pub arrowhead: Option<String>,
	pub color: Option<String>,
	pub style: Option<String>,
	#[serde(flatten)]
	pub attrs: HashMap<String, serde_json::Value>,
}

impl Edge {
	pub fn splines(pos: &str) -> Result<Vec<Spline>, DotError> {
		let mut splines = Vec::new();

		for segment in pos.split(';') {
			let segment = segment.trim();
			if segment.is_empty() {
				continue;
			}

			let mut tail: Option<Point> = None;
			let mut head: Option<Point> = None;
			let mut points: Vec<Point> = Vec::new();

			for token in segment.split_whitespace() {
				if let Some(rest) = token.strip_prefix("s,") {
					tail = Some(Point::parse(rest)?);
					continue;
				}

				if let Some(rest) = token.strip_prefix("e,") {
					head = Some(Point::parse(rest)?);
					continue;
				}

				points.push(Point::parse(token)?);
			}

			if points.is_empty() && tail.is_none() && head.is_none() {
				return Err(DotError::InvalidEdge(segment.to_string()));
			}

			splines.push(Spline { tail, head, points });
		}

		if splines.is_empty() {
			return Err(DotError::InvalidEdge(pos.to_string()));
		}

		Ok(splines)
	}
}

#[derive(Debug)]
pub struct Spline {
	pub tail: Option<Point>,
	pub head: Option<Point>,
	pub points: Vec<Point>, // control + end points, in order
}

#[derive(Debug, Error)]
pub enum DotError {
	#[error("Invalid bounding box data: {0}")]
	InvalidBoundingBox(String),
	#[error("Missing head on edge: {0}")]
	MissingEdgeHead(String),
	#[error("Invalid head on edge: {0}")]
	InvalidEdgeHead(String),
	#[error("Invalid point: {0}")]
	InvalidPoint(String),
	#[error("A geometry error has occurred: {0}")]
	GeometryError(#[from] GeometryError),
	#[error("Invalid edge: {0}")]
	InvalidEdge(String),
	#[error("Missing bounding box")]
	MissingBoundingBox,
}
