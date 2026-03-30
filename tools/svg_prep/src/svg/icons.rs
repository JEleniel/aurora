use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use xmltree::{Element, XMLNode};

use super::defs::extract_defs_elements;
use super::io::read_svg_file;
use super::refs::{rewrite_references_in_elements, rewrite_references_in_nodes};
use super::types::IconGroup;
use super::util::{is_excluded_tag, is_svg_file, local_name, parse_dimension, parse_view_box};

pub(super) fn load_icon_groups(icons_dir: &Path) -> Result<Vec<IconGroup>> {
	let mut files = Vec::new();
	for entry in fs::read_dir(icons_dir)
		.with_context(|| format!("Failed reading icon directory: {}", icons_dir.display()))?
	{
		let entry = entry.with_context(|| {
			format!(
				"Failed reading an entry from icon directory: {}",
				icons_dir.display()
			)
		})?;
		let path = entry.path();
		if path.is_file() && is_svg_file(&path) {
			files.push(path);
		}
	}

	sort_paths_case_insensitive(&mut files);

	let mut icons = Vec::with_capacity(files.len());
	for path in files {
		let stem = path
			.file_stem()
			.and_then(OsStr::to_str)
			.map(str::to_owned)
			.with_context(|| {
				format!(
					"Failed extracting filename stem for icon file: {}",
					path.display()
				)
			})?;
		let svg = read_svg_file(&path)?;
		let icon_id = format!("i-{stem}");
		let styles = extract_style_texts(&svg);
		let mut defs = extract_defs_elements(&svg);
		let id_map = maybe_prefix_element_ids(&mut defs, &icon_id);
		rewrite_references_in_elements(&mut defs, &id_map);

		let mut children = extract_icon_children(&svg);
		rewrite_references_in_nodes(&mut children, &id_map);
		let (width, height) = svg_dimensions(&svg);

		let mut group = Element::new("g");
		group.attributes.insert("id".to_string(), icon_id.clone());
		group.children = children;

		icons.push(IconGroup {
			id: icon_id,
			group,
			defs,
			styles,
			width,
			height,
		});
	}

	Ok(icons)
}

pub(super) fn extract_style_texts(svg: &Element) -> Vec<String> {
	let mut styles = Vec::new();
	for node in &svg.children {
		let XMLNode::Element(element) = node else {
			continue;
		};
		if !local_name(&element.name).eq_ignore_ascii_case("style") {
			continue;
		}
		let text = text_content(element);
		let trimmed = text.trim();
		if !trimmed.is_empty() {
			styles.push(trimmed.to_string());
		}
	}
	styles
}

pub(super) fn svg_dimensions(root: &Element) -> (f64, f64) {
	let width = root
		.attributes
		.get("width")
		.and_then(|value| parse_dimension(value));
	let height = root
		.attributes
		.get("height")
		.and_then(|value| parse_dimension(value));

	let from_view_box = root
		.attributes
		.get("viewBox")
		.and_then(|value| parse_view_box(value));

	let w = width
		.or_else(|| from_view_box.map(|parts| parts.0))
		.unwrap_or(1.0);
	let h = height
		.or_else(|| from_view_box.map(|parts| parts.1))
		.unwrap_or(1.0);

	(w.max(1.0), h.max(1.0))
}

fn sort_paths_case_insensitive(paths: &mut [std::path::PathBuf]) {
	paths.sort_by(|a, b| {
		let a_name = a.file_name().and_then(OsStr::to_str).unwrap_or_default();
		let b_name = b.file_name().and_then(OsStr::to_str).unwrap_or_default();
		let a_ci = a_name.to_ascii_lowercase();
		let b_ci = b_name.to_ascii_lowercase();
		a_ci.cmp(&b_ci).then(a_name.cmp(b_name))
	});
}

fn text_content(element: &Element) -> String {
	element
		.children
		.iter()
		.filter_map(|node| match node {
			XMLNode::Text(text) => Some(text.as_str()),
			_ => None,
		})
		.collect::<String>()
}

fn extract_icon_children(root: &Element) -> Vec<XMLNode> {
	let mut output = Vec::new();
	for node in &root.children {
		if let XMLNode::Element(element) = node
			&& let Some(cleaned) = sanitize_element(element)
		{
			output.push(XMLNode::Element(cleaned));
		}
	}
	output
}

fn sanitize_element(element: &Element) -> Option<Element> {
	if is_excluded_tag(&element.name) {
		return None;
	}

	let mut cleaned = element.clone();
	cleaned.attributes.remove("id");
	cleaned.children.clear();

	for node in &element.children {
		match node {
			XMLNode::Element(child) => {
				cleaned
					.children
					.extend(sanitize_element(child).into_iter().map(XMLNode::Element));
			}
			_ => cleaned.children.push(node.clone()),
		}
	}

	Some(cleaned)
}

fn prefix_element_ids(elements: &mut [Element], id_prefix: &str) -> HashMap<String, String> {
	let mut id_map = HashMap::new();
	for element in elements {
		prefix_element_ids_recursive(element, id_prefix, &mut id_map);
	}
	id_map
}

fn maybe_prefix_element_ids(elements: &mut [Element], id_prefix: &str) -> HashMap<String, String> {
	if defs_ids_already_prefixed(elements, id_prefix) {
		return HashMap::new();
	}
	prefix_element_ids(elements, id_prefix)
}

fn defs_ids_already_prefixed(elements: &[Element], id_prefix: &str) -> bool {
	for element in elements {
		if !element_ids_prefixed_recursive(element, id_prefix) {
			return false;
		}
	}
	true
}

fn element_ids_prefixed_recursive(element: &Element, id_prefix: &str) -> bool {
	if let Some(id) = element.attributes.get("id")
		&& !id.starts_with(&format!("{id_prefix}-"))
	{
		return false;
	}
	for node in &element.children {
		if let XMLNode::Element(child) = node
			&& !element_ids_prefixed_recursive(child, id_prefix)
		{
			return false;
		}
	}
	true
}

fn prefix_element_ids_recursive(
	element: &mut Element,
	id_prefix: &str,
	id_map: &mut HashMap<String, String>,
) {
	if let Some(existing_id) = element.attributes.get("id").cloned() {
		let prefixed_id = format!("{id_prefix}-{existing_id}");
		element
			.attributes
			.insert("id".to_string(), prefixed_id.clone());
		id_map.insert(existing_id, prefixed_id);
	}

	for node in &mut element.children {
		if let XMLNode::Element(child) = node {
			prefix_element_ids_recursive(child, id_prefix, id_map);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn parse_inline(svg: &str) -> Element {
		Element::parse(svg.as_bytes()).expect("inline SVG should parse")
	}

	#[test]
	fn sanitize_removes_ids_and_excluded_nodes() {
		let source = parse_inline(
			r#"<svg xmlns="http://www.w3.org/2000/svg">
				<metadata><x>drop-me</x></metadata>
				<style>.cls{fill:#fff}</style>
				<g id="outer"><path id="p1" d="M0,0 L1,1"/></g>
			</svg>"#,
		);

		let children = extract_icon_children(&source);
		assert_eq!(children.len(), 1);

		let XMLNode::Element(group) = &children[0] else {
			panic!("Expected first child to be an element")
		};
		assert!(!group.attributes.contains_key("id"));

		let XMLNode::Element(path) = &group.children[0] else {
			panic!("Expected nested path element")
		};
		assert!(!path.attributes.contains_key("id"));
	}
}
