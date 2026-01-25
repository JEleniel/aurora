//! Graphviz `-Tplain` layout extraction and parsing.
//!
//! Aurora uses Graphviz for graph layout, but renders SVG itself for full styling control.
//! This module executes `dot -Tplain` and parses its output into geometry suitable for
//! custom SVG generation.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::errors::{AuroraError, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Point {
	pub(super) x_in: f64,
	pub(super) y_in: f64,
}

#[derive(Debug, Clone)]
pub(super) struct PlainNode {
	pub(super) name: String,
	pub(super) center: Point,
	pub(super) width_in: f64,
	pub(super) height_in: f64,
	pub(super) shape: String,
}

#[derive(Debug, Clone)]
pub(super) struct PlainEdge {
	pub(super) tail: String,
	pub(super) head: String,
	pub(super) points: Vec<Point>,
	pub(super) label: Option<String>,
	pub(super) label_pos: Option<Point>,
}

#[derive(Debug, Clone)]
pub(super) struct PlainGraph {
	pub(super) width_in: f64,
	pub(super) height_in: f64,
	pub(super) nodes: HashMap<String, PlainNode>,
	pub(super) edges: Vec<PlainEdge>,
}

pub(super) fn dot_plain_layout_with_engine(
	dot_command: &str,
	dot_path: &Path,
	engine: &str,
) -> Result<PlainGraph> {
	let output = Command::new(dot_command)
		.arg(format!("-K{engine}"))
		.arg("-Tplain")
		.arg(dot_path)
		.output();
	match output {
		Ok(result) => {
			if !result.status.success() {
				let stderr = String::from_utf8_lossy(&result.stderr);
				return Err(AuroraError::InvalidInput {
					message: format!(
						"Graphviz command '{}' failed: {}",
						dot_command,
						stderr.trim()
					),
				});
			}
			let stdout = String::from_utf8_lossy(&result.stdout);
			parse_plain_output(&stdout)
		}
		Err(err) => Err(AuroraError::io(dot_path, err)),
	}
}

fn parse_plain_output(contents: &str) -> Result<PlainGraph> {
	let mut width_in = None;
	let mut height_in = None;
	let mut nodes = HashMap::new();
	let mut edges = Vec::new();

	for raw_line in contents.lines() {
		let line = raw_line.trim();
		if line.is_empty() {
			continue;
		}
		let tokens = tokenize_plain_line(line)?;
		if tokens.is_empty() {
			continue;
		}
		match tokens[0].as_str() {
			"graph" => {
				// graph <scale> <width> <height>
				if tokens.len() < 4 {
					return Err(AuroraError::InvalidInput {
						message: "Graphviz plain output 'graph' record is malformed".to_string(),
					});
				}
				width_in = Some(parse_f64(&tokens[2])?);
				height_in = Some(parse_f64(&tokens[3])?);
			}
			"node" => {
				// node <name> <x> <y> <w> <h> <label> <style> <shape> <color> <fillcolor>
				if tokens.len() < 11 {
					return Err(AuroraError::InvalidInput {
						message: format!(
							"Graphviz plain output 'node' record is malformed: {}",
							line
						),
					});
				}
				let name = tokens[1].clone();
				let x_in = parse_f64(&tokens[2])?;
				let y_in = parse_f64(&tokens[3])?;
				let width_in = parse_f64(&tokens[4])?;
				let height_in = parse_f64(&tokens[5])?;
				let shape_idx = tokens.len() - 3;
				let shape = tokens[shape_idx].clone();
				nodes.insert(
					name.clone(),
					PlainNode {
						name,
						center: Point { x_in, y_in },
						width_in,
						height_in,
						shape,
					},
				);
			}
			"edge" => {
				// edge <tail> <head> <n> <x1> <y1> ... <xn> <yn> <label> <lx> <ly> <style> <color>
				if tokens.len() < 6 {
					return Err(AuroraError::InvalidInput {
						message: format!(
							"Graphviz plain output 'edge' record is malformed: {}",
							line
						),
					});
				}
				let tail = tokens[1].clone();
				let head = tokens[2].clone();
				let n = parse_usize(&tokens[3])?;
				let mut idx = 4;
				if tokens.len() < idx + (2 * n) {
					return Err(AuroraError::InvalidInput {
						message: format!(
							"Graphviz plain output 'edge' record is missing points: {}",
							line
						),
					});
				}
				let mut points = Vec::with_capacity(n);
				for _ in 0..n {
					let x_in = parse_f64(&tokens[idx])?;
					let y_in = parse_f64(&tokens[idx + 1])?;
					points.push(Point { x_in, y_in });
					idx += 2;
				}

				let mut label = None;
				let mut label_pos = None;
				// Remaining tokens may include: <label> <lx> <ly> <style> <color>.
				// Relationship labels can contain spaces (e.g., "depends on"), and HTML labels may be
				// unquoted, so reconstruct from the tail.
				if tokens.len() >= idx + 5 {
					let lx_idx = tokens.len() - 4;
					let ly_idx = tokens.len() - 3;
					let label_tokens = &tokens[idx..lx_idx];
					if !label_tokens.is_empty() {
						let lx = parse_f64(&tokens[lx_idx])?;
						let ly = parse_f64(&tokens[ly_idx])?;
						label = Some(label_tokens.join(" "));
						label_pos = Some(Point { x_in: lx, y_in: ly });
					}
				}

				edges.push(PlainEdge {
					tail,
					head,
					points,
					label,
					label_pos,
				});
			}
			"stop" => break,
			_ => {}
		}
	}

	let width_in = width_in.ok_or_else(|| AuroraError::InvalidInput {
		message: "Graphviz plain output missing graph width".to_string(),
	})?;
	let height_in = height_in.ok_or_else(|| AuroraError::InvalidInput {
		message: "Graphviz plain output missing graph height".to_string(),
	})?;

	Ok(PlainGraph {
		width_in,
		height_in,
		nodes,
		edges,
	})
}

fn tokenize_plain_line(line: &str) -> Result<Vec<String>> {
	let mut tokens = Vec::new();
	let mut current = String::new();
	let mut chars = line.chars().peekable();
	let mut in_quotes = false;

	while let Some(ch) = chars.next() {
		if in_quotes {
			match ch {
				'\\' => {
					let escaped = chars.next().ok_or_else(|| AuroraError::InvalidInput {
						message: format!("invalid escape sequence in Graphviz plain line: {line}"),
					})?;
					current.push(match escaped {
						'n' => '\n',
						't' => '\t',
						'r' => '\r',
						'\\' => '\\',
						'"' => '"',
						other => other,
					});
				}
				'"' => {
					in_quotes = false;
					tokens.push(current.clone());
					current.clear();
				}
				other => current.push(other),
			}
			continue;
		}

		match ch {
			'"' => {
				in_quotes = true;
			}
			ch if ch.is_whitespace() => {
				if !current.is_empty() {
					tokens.push(current.clone());
					current.clear();
				}
				while chars.peek().is_some_and(|peek| peek.is_whitespace()) {
					chars.next();
				}
			}
			other => current.push(other),
		}
	}

	if in_quotes {
		return Err(AuroraError::InvalidInput {
			message: format!("unterminated quoted string in Graphviz plain line: {line}"),
		});
	}

	if !current.is_empty() {
		tokens.push(current);
	}

	Ok(tokens)
}

#[cfg(test)]
mod tests {
	use super::parse_plain_output;

	#[test]
	fn parses_unquoted_html_node_label_with_spaces() {
		let plain = "graph 1 2 3\nnode CAP-001 1 1 2 1 <<FONT>✨ Capability:<br />My Name<br />(CAP-001)</FONT>> solid box black white\nstop\n";
		let graph = parse_plain_output(plain).expect("plain output parses");
		let node = graph.nodes.get("CAP-001").expect("node parsed");
		assert_eq!(node.shape, "box");
	}

	#[test]
	fn parses_edge_label_with_spaces() {
		let plain = "graph 1 2 3\nedge A B 2 0 0 1 1 depends on 0.5 0.5 solid black\nstop\n";
		let graph = parse_plain_output(plain).expect("plain output parses");
		let edge = graph.edges.first().expect("edge parsed");
		assert_eq!(edge.label.as_deref(), Some("depends on"));
		assert!(edge.label_pos.is_some());
	}
}

fn parse_f64(value: &str) -> Result<f64> {
	value.parse::<f64>().map_err(|_| AuroraError::InvalidInput {
		message: format!("invalid float in Graphviz plain output: '{value}'"),
	})
}

fn parse_usize(value: &str) -> Result<usize> {
	value
		.parse::<usize>()
		.map_err(|_| AuroraError::InvalidInput {
			message: format!("invalid integer in Graphviz plain output: '{value}'"),
		})
}
