//! Graphviz-backed layout execution and `plain` output parsing.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::render::render_error::RenderError;

use super::graph::LayoutGraph;
use super::types::{LayoutFamily, LayoutNode, LayoutPoint};

const PIXELS_PER_INCH: f32 = 300.0;
const NODE_WIDTH_IN: f32 = 1.6;
const NODE_HEIGHT_IN: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
struct EngineSpec {
	command: &'static str,
	ranksep_attr: &'static str,
	ranksep: f32,
	nodesep_attr: &'static str,
	nodesep: f32,
	rankdir: Option<&'static str>,
	splines: &'static str,
	overlap: Option<&'static str>,
	oneblock: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub(super) struct GraphvizLayout {
	pub(super) nodes: HashMap<String, LayoutNode>,
	pub(super) routes: HashMap<(String, String), Vec<LayoutPoint>>,
}

pub(super) fn layout_with_graphviz(
	graph: &LayoutGraph,
	family: LayoutFamily,
) -> Result<GraphvizLayout, RenderError> {
	let spec = spec_for_family(family);
	let dot = build_graphviz_input(graph, spec, family);
	let output = run_graphviz(spec.command, dot.as_str())?;
	parse_plain_output(output.as_str(), graph)
}

pub(super) fn build_dot(graph: &LayoutGraph, family: LayoutFamily) -> String {
	let spec = spec_for_family(family);
	build_graphviz_input(graph, spec, family)
}

fn spec_for_family(family: LayoutFamily) -> EngineSpec {
	match family {
		LayoutFamily::TreeTopDown => EngineSpec {
			command: "dot",
			ranksep_attr: "ranksep",
			ranksep: 2.0,
			nodesep_attr: "nodesep",
			nodesep: 2.5,
			rankdir: Some("TB"),
			splines: "ortho",
			overlap: None,
			oneblock: None,
		},
		LayoutFamily::TreeLeftRight => EngineSpec {
			command: "dot",
			ranksep_attr: "ranksep",
			ranksep: 2.0,
			nodesep_attr: "nodesep",
			nodesep: 1.0,
			rankdir: Some("LR"),
			splines: "ortho",
			overlap: None,
			oneblock: None,
		},
	}
}

fn build_graphviz_input(graph: &LayoutGraph, spec: EngineSpec, family: LayoutFamily) -> String {
	let mut dot = String::from("digraph aurora {\n");
	let mut graph_attrs = vec![
		format!("{}={:.4}", spec.nodesep_attr, spec.nodesep),
		format!("{}={:.4}", spec.ranksep_attr, spec.ranksep),
		format!("splines={}", spec.splines),
	];
	if matches!(
		family,
		LayoutFamily::TreeTopDown | LayoutFamily::TreeLeftRight
	) {
		graph_attrs.push("compound=true".to_string());
		graph_attrs.push("reminicross=true".to_string());
		graph_attrs.push("center=true".to_string());
		graph_attrs.push("concentrate=true".to_string());
	}
	if let Some(rankdir) = spec.rankdir {
		graph_attrs.push(format!("rankdir={rankdir}"));
	}
	if let Some(overlap) = spec.overlap {
		graph_attrs.push(format!("overlap={overlap}"));
	}
	if let Some(oneblock) = spec.oneblock {
		graph_attrs.push(format!("oneblock={oneblock}"));
	}
	dot.push_str(format!("  graph [{}];\n", graph_attrs.join(", ")).as_str());
	dot.push_str(
		format!(
			"  node [shape=box, fixedsize=true, width={:.4}, height={:.4}, margin=0, label=\"\", fontsize=16];\n",
			NODE_WIDTH_IN,
			NODE_HEIGHT_IN,
		)
		.as_str(),
	);
	dot.push_str("  edge [arrowhead=none, penwidth=40];\n");

	let mut nodes: Vec<&String> = graph.allowed_nodes.iter().collect();
	nodes.sort();
	for node_id in nodes {
		dot.push_str(format!("  {};\n", quote_dot(node_id.as_str())).as_str());
	}

	let mut edges = graph.edges.iter().collect::<Vec<_>>();
	edges.sort();
	for (source, target) in edges {
		dot.push_str(
			format!(
				"  {} -> {};\n",
				quote_dot(source.as_str()),
				quote_dot(target.as_str()),
			)
			.as_str(),
		);
	}

	dot.push_str("}\n");
	dot
}

fn run_graphviz(command: &str, input: &str) -> Result<String, RenderError> {
	let mut child = match Command::new(command)
		.arg("-Tplain")
		.arg("-y")
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
	{
		Ok(child) => child,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
			return Err(RenderError::GraphvizUnavailable(command.to_string()));
		}
		Err(error) => return Err(RenderError::Io(error)),
	};
	if let Some(stdin) = child.stdin.as_mut() {
		stdin.write_all(input.as_bytes())?;
	}
	let output = child.wait_with_output()?;
	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
		let reason = if stderr.is_empty() {
			format!("{command} exited with {}", output.status)
		} else {
			format!("{command}: {stderr}")
		};
		return Err(RenderError::GraphvizFailed(reason));
	}
	Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_plain_output(output: &str, graph: &LayoutGraph) -> Result<GraphvizLayout, RenderError> {
	let expected_nodes = graph.allowed_nodes.iter().cloned().collect::<HashSet<_>>();
	let expected_edges = graph.edges.iter().cloned().collect::<HashSet<_>>();
	let mut nodes = HashMap::new();
	let mut routes = HashMap::new();
	for record in plain_records(output) {
		let trimmed = record.trim();
		if trimmed.is_empty() || trimmed == "stop" || trimmed.starts_with("graph ") {
			continue;
		}
		let fields = trimmed.split_whitespace().collect::<Vec<_>>();
		match fields.first().copied() {
			Some("node") => parse_node_line(fields.as_slice(), &expected_nodes, &mut nodes)?,
			Some("edge") => parse_edge_line(fields.as_slice(), &expected_edges, &mut routes)?,
			Some(other) => {
				return Err(RenderError::GraphvizParse(format!(
					"unsupported plain record '{other}'"
				)));
			}
			None => continue,
		}
	}
	for node_id in &expected_nodes {
		if !nodes.contains_key(node_id) {
			return Err(RenderError::GraphvizParse(format!(
				"missing node '{node_id}' in plain output"
			)));
		}
	}
	Ok(GraphvizLayout { nodes, routes })
}

fn plain_records(output: &str) -> Vec<String> {
	let mut records: Vec<String> = Vec::new();
	for line in output.lines() {
		let trimmed = line.trim();
		if trimmed.is_empty() {
			continue;
		}
		if is_plain_record_start(trimmed) || records.is_empty() {
			records.push(trimmed.to_string());
			continue;
		}
		if let Some(previous) = records.last_mut() {
			previous.push(' ');
			previous.push_str(trimmed);
		}
	}
	records
}

fn is_plain_record_start(line: &str) -> bool {
	matches!(
		line.split_whitespace().next(),
		Some("graph") | Some("node") | Some("edge") | Some("stop")
	)
}

fn parse_node_line(
	fields: &[&str],
	expected_nodes: &HashSet<String>,
	nodes: &mut HashMap<String, LayoutNode>,
) -> Result<(), RenderError> {
	if fields.len() < 6 {
		return Err(RenderError::GraphvizParse(format!(
			"node record is too short: {}",
			fields.join(" ")
		)));
	}
	let node_id = fields[1].trim_matches('"');
	if !expected_nodes.contains(node_id) {
		return Ok(());
	}
	let center_x = parse_inches(fields[2])? * PIXELS_PER_INCH;
	let center_y = parse_inches(fields[3])? * PIXELS_PER_INCH;
	let width_px = parse_inches(fields[4])? * PIXELS_PER_INCH;
	let height_px = parse_inches(fields[5])? * PIXELS_PER_INCH;
	nodes.insert(
		node_id.to_string(),
		LayoutNode {
			id: node_id.to_string(),
			x: (center_x - (width_px / 2.0)).round() as i32,
			y: (center_y - (height_px / 2.0)).round() as i32,
		},
	);
	Ok(())
}

fn parse_edge_line(
	fields: &[&str],
	expected_edges: &HashSet<(String, String)>,
	routes: &mut HashMap<(String, String), Vec<LayoutPoint>>,
) -> Result<(), RenderError> {
	if fields.len() < 5 {
		return Err(RenderError::GraphvizParse(format!(
			"edge record is too short: {}",
			fields.join(" ")
		)));
	}
	let source = normalize_edge_endpoint(fields[1]);
	let target = normalize_edge_endpoint(fields[2]);
	if !expected_edges.contains(&(source.clone(), target.clone())) {
		return Ok(());
	}
	let point_count = fields[3].parse::<usize>().map_err(|_| {
		RenderError::GraphvizParse(format!("invalid edge point count '{}'", fields[3]))
	})?;
	let coord_fields = 4 + point_count * 2;
	if fields.len() < coord_fields {
		return Err(RenderError::GraphvizParse(format!(
			"edge record is missing route coordinates: {}",
			fields.join(" ")
		)));
	}
	let mut points = Vec::with_capacity(point_count);
	for index in 0..point_count {
		let x = parse_inches(fields[4 + index * 2])? * PIXELS_PER_INCH;
		let y = parse_inches(fields[5 + index * 2])? * PIXELS_PER_INCH;
		points.push(LayoutPoint { x, y });
	}
	routes.insert((source, target), points);
	Ok(())
}

fn normalize_edge_endpoint(value: &str) -> String {
	value
		.trim_matches('"')
		.split(':')
		.next()
		.unwrap_or(value)
		.to_string()
}

fn parse_inches(value: &str) -> Result<f32, RenderError> {
	value.parse::<f32>().map_err(|_| {
		RenderError::GraphvizParse(format!("invalid numeric value '{value}' in plain output"))
	})
}

fn quote_dot(value: &str) -> String {
	let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
	format!("\"{escaped}\"")
}

#[cfg(test)]
#[path = "graphviz_tests.rs"]
mod tests;
