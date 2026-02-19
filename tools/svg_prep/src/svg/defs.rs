use xmltree::{Element, XMLNode};

use super::util::{is_defs_tag, local_name};

pub(super) fn extract_defs_elements(svg: &Element) -> Vec<Element> {
	for node in &svg.children {
		if let XMLNode::Element(element) = node {
			if is_defs_tag(&element.name) {
				return element
					.children
					.iter()
					.filter_map(|child| match child {
						XMLNode::Element(next) => Some(next.clone()),
						_ => None,
					})
					.collect();
			}
		}
	}
	Vec::new()
}

pub(super) fn replace_defs_section(svg: &mut Element, defs_children: Vec<Element>) {
	let mut defs = Element::new("defs");
	defs.children = defs_children
		.into_iter()
		.map(XMLNode::Element)
		.collect::<Vec<_>>();

	let mut first_index = None;
	let mut trailing = Vec::new();
	for (index, node) in svg.children.iter().enumerate() {
		if let XMLNode::Element(element) = node {
			if is_defs_tag(&element.name) {
				if first_index.is_none() {
					first_index = Some(index);
				} else {
					trailing.push(index);
				}
			}
		}
	}

	if let Some(index) = first_index {
		svg.children[index] = XMLNode::Element(defs);
		for index in trailing.into_iter().rev() {
			svg.children.remove(index);
		}
	} else {
		svg.children.insert(0, XMLNode::Element(defs));
	}
}

pub(super) fn sort_defs_by_id(elements: &mut [Element]) {
	elements.sort_by(|a, b| {
		let a_id = a.attributes.get("id").cloned().unwrap_or_default();
		let b_id = b.attributes.get("id").cloned().unwrap_or_default();
		let a_ci = a_id.to_ascii_lowercase();
		let b_ci = b_id.to_ascii_lowercase();
		a_ci.cmp(&b_ci)
			.then(a_id.cmp(&b_id))
			.then(local_name(&a.name).cmp(local_name(&b.name)))
	});
}
