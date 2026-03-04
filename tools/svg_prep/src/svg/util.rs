use std::ffi::OsStr;
use std::path::Path;

use xmltree::{Element, XMLNode};

const SVG_XMLNS: &str = "http://www.w3.org/2000/svg";

pub(super) const EXCLUDED_TAGS: [&str; 6] = ["svg", "defs", "metadata", "title", "desc", "style"];

pub(super) fn is_svg_file(path: &Path) -> bool {
	path.extension()
		.and_then(OsStr::to_str)
		.is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
}

pub(super) fn is_excluded_tag(name: &str) -> bool {
	let local = local_name(name);
	EXCLUDED_TAGS
		.iter()
		.any(|candidate| local.eq_ignore_ascii_case(candidate))
}

pub(super) fn is_defs_tag(name: &str) -> bool {
	local_name(name).eq_ignore_ascii_case("defs")
}

pub(super) fn local_name(name: &str) -> &str {
	if let Some(stripped) = name.strip_prefix('{')
		&& let Some(end) = stripped.find('}')
	{
		return &stripped[end + 1..];
	}
	name.rsplit(':').next().unwrap_or(name)
}

pub(super) fn format_number(value: f64) -> String {
	if (value - value.round()).abs() < f64::EPSILON {
		return format!("{value:.0}");
	}

	let mut out = format!("{value:.6}");
	while out.contains('.') && out.ends_with('0') {
		out.pop();
	}
	if out.ends_with('.') {
		out.pop();
	}
	out
}

pub(super) fn is_svg_id_char(ch: char) -> bool {
	ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.')
}

pub(super) fn parse_dimension(value: &str) -> Option<f64> {
	let mut out = String::new();
	let mut started = false;

	for ch in value.chars() {
		let allowed = ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E');
		if allowed {
			started = true;
			out.push(ch);
			continue;
		}
		if started {
			break;
		}
	}

	if out.is_empty() {
		return None;
	}
	out.parse::<f64>().ok().filter(|v| v.is_finite())
}

pub(super) fn parse_view_box(value: &str) -> Option<(f64, f64)> {
	let mut parts = Vec::new();
	for piece in value.split(|ch: char| ch.is_ascii_whitespace() || ch == ',') {
		if piece.is_empty() {
			continue;
		}
		let parsed = piece.parse::<f64>().ok()?;
		parts.push(parsed);
	}

	if parts.len() != 4 {
		return None;
	}

	Some((parts[2].abs(), parts[3].abs()))
}

/// Remove namespace declarations and namespace-qualified names from all descendants.
///
/// Contract: In generated SVGs, only the root `<svg>` element may contain `xmlns*` attributes.
pub(super) fn strip_non_root_namespaces_in_place(svg: &mut Element) {
	for node in &mut svg.children {
		if let XMLNode::Element(element) = node {
			strip_namespaces_recursive(element);
		}
	}
}

pub(super) fn remove_attribute_by_local_name_in_place(svg: &mut Element, attr_local: &str) {
	remove_attribute_by_local_name_recursive(svg, attr_local);
}

/// Remove namespaces and `xmlns*` declarations from the entire SVG tree, including the root.
///
/// This is primarily useful for optimized source assets, where Inkscape/Sodipodi namespaces are
/// noise once all such attributes and elements have been stripped.
pub(super) fn strip_all_namespaces_in_place(svg: &mut Element) {
	strip_namespaces_recursive(svg);
	svg.attributes
		.insert("xmlns".to_string(), SVG_XMLNS.to_string());
}

fn strip_namespaces_recursive(element: &mut Element) {
	// Normalize the element name to the local name so xmltree doesn't re-emit xmlns attributes.
	let local = local_name(&element.name).to_string();
	element.name = local;

	// xmltree may track namespaces separately from attributes; clear to avoid per-element xmlns.
	element.prefix = None;
	element.namespace = None;
	element.namespaces = None;

	normalize_attribute_names_in_place(element);

	remove_xmlns_attributes(element);

	for node in &mut element.children {
		if let XMLNode::Element(child) = node {
			strip_namespaces_recursive(child);
		}
	}
}

fn normalize_attribute_names_in_place(element: &mut Element) {
	if element.attributes.is_empty() {
		return;
	}

	let mut replacements: Vec<(String, String)> = Vec::new();
	for (key, value) in &element.attributes {
		let local = local_name(key);
		if local.eq_ignore_ascii_case("href") {
			replacements.push((key.clone(), value.clone()));
			continue;
		}
		if key.starts_with('{') {
			replacements.push((key.clone(), value.clone()));
		}
	}
	if replacements.is_empty() {
		return;
	}

	for (key, value) in replacements {
		let _ = element.attributes.remove(&key);
		let local = local_name(&key).to_string();
		if local.eq_ignore_ascii_case("href") {
			element.attributes.insert("href".to_string(), value);
		} else {
			element.attributes.insert(local, value);
		}
	}
}

fn remove_xmlns_attributes(element: &mut Element) {
	let keys: Vec<String> = element
		.attributes
		.keys()
		.filter(|key| {
			key.eq_ignore_ascii_case("xmlns") || key.to_ascii_lowercase().starts_with("xmlns:")
		})
		.cloned()
		.collect();
	for key in keys {
		element.attributes.remove(&key);
	}
}

fn remove_attribute_by_local_name_recursive(element: &mut Element, attr_local: &str) {
	let keys: Vec<String> = element
		.attributes
		.keys()
		.filter(|key| local_name(key).eq_ignore_ascii_case(attr_local))
		.cloned()
		.collect();
	for key in keys {
		element.attributes.remove(&key);
	}

	for node in &mut element.children {
		if let XMLNode::Element(child) = node {
			remove_attribute_by_local_name_recursive(child, attr_local);
		}
	}
}
