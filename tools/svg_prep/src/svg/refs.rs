use std::collections::HashMap;

use xmltree::{Element, XMLNode};

use super::util::{is_svg_id_char, local_name};

pub(super) fn rewrite_references_in_nodes(nodes: &mut [XMLNode], id_map: &HashMap<String, String>) {
	for node in nodes {
		if let XMLNode::Element(element) = node {
			rewrite_references_in_element(element, id_map);
		}
	}
}

pub(super) fn rewrite_references_in_elements(
	elements: &mut [Element],
	id_map: &HashMap<String, String>,
) {
	for element in elements {
		rewrite_references_in_element(element, id_map);
	}
}

pub(super) fn rewrite_references_in_element(
	element: &mut Element,
	id_map: &HashMap<String, String>,
) {
	for (name, value) in &mut element.attributes {
		let rewritten = rewrite_reference_value(name, value, id_map);
		*value = rewritten;
	}

	for node in &mut element.children {
		if let XMLNode::Element(child) = node {
			rewrite_references_in_element(child, id_map);
		}
	}
}

pub(super) fn extract_url_ids(value: &str) -> Vec<String> {
	let mut ids = Vec::new();
	let mut cursor = 0usize;
	while let Some(rel) = value[cursor..].find("url(#") {
		let start = cursor + rel;
		let id_start = start + "url(#".len();
		let id: String = value[id_start..]
			.chars()
			.take_while(|ch| is_svg_id_char(*ch))
			.collect();
		let id_len = id.len();
		ids.push(id);
		cursor = id_start + id_len;
	}
	ids
}

fn rewrite_reference_value(
	attribute_name: &str,
	value: &str,
	id_map: &HashMap<String, String>,
) -> String {
	let mut rewritten = rewrite_url_reference_ids(value, id_map);

	if local_name(attribute_name).eq_ignore_ascii_case("href")
		&& let Some(id) = rewritten.strip_prefix('#')
		&& id.chars().all(is_svg_id_char)
		&& let Some(prefixed) = id_map.get(id)
	{
		rewritten = format!("#{prefixed}");
	}

	rewritten
}

fn rewrite_url_reference_ids(value: &str, id_map: &HashMap<String, String>) -> String {
	let mut out = String::new();
	let mut cursor = 0usize;

	while let Some(rel) = value[cursor..].find("url(#") {
		let start = cursor + rel;
		let id_start = start + "url(#".len();
		out.push_str(&value[cursor..id_start]);
		let id: String = value[id_start..]
			.chars()
			.take_while(|ch| is_svg_id_char(*ch))
			.collect();
		if let Some(prefixed) = id_map.get(&id) {
			out.push_str(prefixed);
		} else {
			out.push_str(&id);
		}

		cursor = id_start + id.len();
	}

	if cursor == 0 {
		return value.to_string();
	}

	out.push_str(&value[cursor..]);
	out
}
