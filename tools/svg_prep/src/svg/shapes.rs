use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use xmltree::{Element, XMLNode};

use super::defs::extract_defs_elements;
use super::io::read_svg_file;
use super::types::IconGroup;
use super::util::{is_defs_tag, is_svg_file, local_name, parse_view_box};

const DEFAULT_SHAPE_WIDTH_PX: f64 = 720.0;
const DEFAULT_SHAPE_HEIGHT_PX: f64 = 450.0;

pub(super) fn load_shape_defs(path: &Path) -> Result<Vec<Element>> {
	if path.is_dir() {
		let shapes = load_shape_groups(path)?;
		let mut defs = Vec::new();
		for shape in shapes {
			defs.push(shape.group);
		}
		return Ok(defs);
	}

	let svg = read_svg_file(path)?;
	Ok(extract_defs_elements(&svg))
}

pub(super) fn load_shape_groups(shapes_dir: &Path) -> Result<Vec<IconGroup>> {
	let mut files = Vec::new();
	for entry in fs::read_dir(shapes_dir)
		.with_context(|| format!("Failed reading shapes directory: {}", shapes_dir.display()))?
	{
		let entry = entry.with_context(|| {
			format!(
				"Failed reading an entry from shapes directory: {}",
				shapes_dir.display()
			)
		})?;
		let path = entry.path();
		if path.is_file() && is_svg_file(&path) {
			files.push(path);
		}
	}

	sort_paths_case_insensitive(&mut files);

	let mut shapes = Vec::with_capacity(files.len());
	for path in files {
		let stem = path
			.file_stem()
			.and_then(OsStr::to_str)
			.map(str::to_owned)
			.with_context(|| {
				format!(
					"Failed extracting filename stem for shape file: {}",
					path.display()
				)
			})?;
		let svg = read_svg_file(&path)?;
		let (width, height) = shape_dimensions(&svg);
		let mut group = find_top_level_group_by_name(&svg, &stem)
			.with_context(|| format!("Shape '{}' not found in {}", stem, path.display()))?;
		normalize_shape_group_id(&mut group, &stem);
		sanitize_shape_group_in_place(&mut group);

		shapes.push(IconGroup {
			id: stem,
			group,
			defs: Vec::new(),
			styles: Vec::new(),
			width,
			height,
		});
	}

	Ok(shapes)
}

pub(super) fn shape_dimensions(svg: &Element) -> (f64, f64) {
	let from_view_box = svg
		.attributes
		.get("viewBox")
		.and_then(|value| parse_view_box(value));

	let (w, h) = from_view_box.unwrap_or((DEFAULT_SHAPE_WIDTH_PX, DEFAULT_SHAPE_HEIGHT_PX));
	(w.max(1.0), h.max(1.0))
}

pub(super) fn find_or_wrap_shape_group(svg: &Element, stem: &str) -> Element {
	if let Some(mut group) = find_top_level_group_by_name(svg, stem) {
		normalize_shape_group_id(&mut group, stem);
		return group;
	}

	let mut group = Element::new("g");
	group.attributes.insert("id".to_string(), stem.to_string());
	for node in &svg.children {
		let XMLNode::Element(element) = node else {
			continue;
		};
		if is_defs_tag(&element.name) || local_name(&element.name).eq_ignore_ascii_case("style") {
			continue;
		}
		group.children.push(XMLNode::Element(element.clone()));
	}
	group
}

pub(super) fn find_top_level_group_by_name(svg: &Element, name: &str) -> Option<Element> {
	for node in &svg.children {
		let XMLNode::Element(element) = node else {
			continue;
		};
		if !local_name(&element.name).eq_ignore_ascii_case("g") {
			continue;
		}
		if group_matches_name(element, name) {
			return Some(element.clone());
		}
	}
	None
}

pub(super) fn sanitize_shape_group_in_place(group: &mut Element) {
	if let Some(style) = group.attributes.get("style").cloned() {
		if let Some(retained) = retained_inline_style(&style) {
			group.attributes.insert("style".to_string(), retained);
		} else {
			group.attributes.remove("style");
		}
	}
	ensure_group_class(group, "aurora-symbol");
	remove_inkscape_label_attribute(group);
	sanitize_shape_children(&mut group.children);
}

fn retained_inline_style(style: &str) -> Option<String> {
	let mut fill: Option<String> = None;
	let mut keep_fill_opacity_zero = false;
	let mut keep_stroke_opacity_zero = false;
	let mut stroke: Option<String> = None;
	let mut dasharray: Option<String> = None;

	for decl in style.split(';') {
		let decl = decl.trim();
		if decl.is_empty() {
			continue;
		}

		let Some((prop, value)) = decl.split_once(':') else {
			continue;
		};
		let prop = prop.trim().to_ascii_lowercase();
		let value = value.trim();

		if prop == "fill-opacity" && value == "0" {
			keep_fill_opacity_zero = true;
			continue;
		}

		if prop == "fill" {
			if value.eq_ignore_ascii_case("#00000000") {
				keep_fill_opacity_zero = true;
			} else if value.eq_ignore_ascii_case("none") || is_black_six_fill(value) {
				fill = None;
			} else if !value.is_empty() {
				fill = Some(value.to_string());
			}
			continue;
		}

		if prop == "stroke-opacity" && value == "0" {
			keep_stroke_opacity_zero = true;
			continue;
		}

		if prop == "stroke-dasharray" {
			if !value.is_empty() && !value.eq_ignore_ascii_case("none") {
				dasharray = Some(value.to_string());
			}
			continue;
		}

		if prop == "stroke" {
			if value.eq_ignore_ascii_case("#000000") {
				stroke = None;
			} else if !value.is_empty() {
				stroke = Some(value.to_string());
			}
			continue;
		}
	}

	let mut out = Vec::new();
	if let Some(value) = fill {
		out.push(format!("fill:{value}"));
	}
	if keep_fill_opacity_zero {
		out.push("fill-opacity:0".to_string());
	}
	if keep_stroke_opacity_zero {
		out.push("stroke-opacity:0".to_string());
	}
	if let Some(value) = stroke {
		out.push(format!("stroke:{value}"));
	}
	if let Some(value) = dasharray {
		out.push(format!("stroke-dasharray:{value}"));
	}

	if out.is_empty() {
		None
	} else {
		Some(format!("{};", out.join(";")))
	}
}

fn is_black_six_fill(value: &str) -> bool {
	let trimmed = value.trim();
	let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
	hex.eq_ignore_ascii_case("000000")
}

fn ensure_group_class(group: &mut Element, class_name: &str) {
	let existing = group.attributes.get("class").cloned().unwrap_or_default();
	let mut classes: Vec<&str> = existing
		.split_whitespace()
		.filter(|s| !s.is_empty())
		.collect();
	if !classes.iter().any(|c| c == &class_name) {
		classes.push(class_name);
	}
	let combined = classes.join(" ");
	if combined.is_empty() {
		group.attributes.remove("class");
	} else {
		group.attributes.insert("class".to_string(), combined);
	}
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

fn group_matches_name(group: &Element, name: &str) -> bool {
	match group.attributes.get("id") {
		Some(id) if id == name => return true,
		_ => {}
	}
	if let Some(label) = inkscape_label_value(group)
		&& label == name
	{
		return true;
	}
	false
}

fn normalize_shape_group_id(group: &mut Element, stem: &str) {
	group.attributes.insert("id".to_string(), stem.to_string());
	remove_inkscape_label_attribute(group);
}

fn inkscape_label_value(group: &Element) -> Option<&str> {
	if let Some(label) = group.attributes.get("inkscape:label") {
		return Some(label);
	}

	for (key, value) in &group.attributes {
		if local_name(key).eq_ignore_ascii_case("label") {
			return Some(value);
		}
	}

	None
}

fn remove_inkscape_label_attribute(group: &mut Element) {
	let keys: Vec<String> = group
		.attributes
		.keys()
		.filter(|key| local_name(key).eq_ignore_ascii_case("label"))
		.cloned()
		.collect();
	for key in keys {
		group.attributes.remove(&key);
	}
}

fn sanitize_shape_children(children: &mut [XMLNode]) {
	for node in children.iter_mut() {
		let XMLNode::Element(element) = node else {
			continue;
		};
		element.attributes.remove("id");
		// Inkscape occasionally emits nodetypes attributes that are not meaningful for rendering.
		remove_attributes_by_local_name(element, "nodetypes");
		if local_name(&element.name).eq_ignore_ascii_case("g") {
			if let Some(style) = element.attributes.get("style").cloned() {
				if let Some(retained) = retained_inline_style(&style) {
					element.attributes.insert("style".to_string(), retained);
				} else {
					element.attributes.remove("style");
				}
			}
		} else if let Some(style) = element.attributes.get("style").cloned() {
			if let Some(retained) = retained_inline_style(&style) {
				element.attributes.insert("style".to_string(), retained);
			} else {
				element.attributes.remove("style");
			}
		}
		sanitize_shape_children(&mut element.children);
	}
}

fn remove_attributes_by_local_name(element: &mut Element, attr_local: &str) {
	let keys: Vec<String> = element
		.attributes
		.keys()
		.filter(|key| local_name(key).eq_ignore_ascii_case(attr_local))
		.cloned()
		.collect();
	for key in keys {
		element.attributes.remove(&key);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn find_top_level_group_by_name_matches_inkscape_label() {
		let svg = Element::parse(
			r#"<svg xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xmlns="http://www.w3.org/2000/svg">
				<g id="g1" inkscape:label="folder"><path d="M0,0"/></g>
			</svg>"#
			.as_bytes(),
		)
		.expect("inline SVG should parse");

		let group = find_top_level_group_by_name(&svg, "folder").expect("group should be found");
		assert_eq!(inkscape_label_value(&group), Some("folder"));
	}

	#[test]
	fn sanitize_preserves_stroke_dasharray_in_inline_style() {
		let mut group = Element::parse(
			r#"<g id="g1"><path id="p1" style="stroke:#000; stroke-dasharray: 6, 2; fill:none" d="M0,0"/></g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut group);

		let XMLNode::Element(path) = &group.children[0] else {
			panic!("expected path element")
		};
		assert!(!path.attributes.contains_key("id"));
		assert_eq!(
			path.attributes.get("style").map(String::as_str),
			Some("stroke:#000;stroke-dasharray:6, 2;")
		);
	}

	#[test]
	fn sanitize_drops_stroke_dasharray_none() {
		let mut group = Element::parse(
			r#"<g id="g1"><path id="p1" style="stroke-dasharray:none; stroke:#000" d="M0,0"/></g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut group);

		let XMLNode::Element(path) = &group.children[0] else {
			panic!("expected path element")
		};
		assert_eq!(
			path.attributes.get("style").map(String::as_str),
			Some("stroke:#000;")
		);
	}

	#[test]
	fn sanitize_preserves_fill_opacity_and_dasharray() {
		let mut group = Element::parse(
			r#"<g id="g1"><path id="p1" style="fill-opacity:0; stroke-dasharray:3 1" d="M0,0"/></g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut group);

		let XMLNode::Element(path) = &group.children[0] else {
			panic!("expected path element")
		};
		assert_eq!(
			path.attributes.get("style").map(String::as_str),
			Some("fill-opacity:0;stroke-dasharray:3 1;")
		);
	}

	#[test]
	fn sanitize_preserves_group_dasharray_in_inline_style() {
		let mut group = Element::parse(
			r#"<g id="g1" style="stroke-dasharray: 4 2; stroke:#000">
				<path id="p1" d="M0,0"/>
			</g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut group);

		assert_eq!(
			group.attributes.get("style").map(String::as_str),
			Some("stroke:#000;stroke-dasharray:4 2;")
		);
	}

	#[test]
	fn sanitize_drops_black_hex_stroke_in_inline_style() {
		let mut group = Element::parse(
			r#"<g id="g1"><path id="p1" style="stroke:#000000; stroke-width:2" d="M0,0"/></g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut group);

		let XMLNode::Element(path) = &group.children[0] else {
			panic!("expected path element")
		};
		assert!(!path.attributes.contains_key("style"));
	}

	#[test]
	fn sanitize_preserves_stroke_none_in_inline_style() {
		let mut group = Element::parse(
			r#"<g id="g1"><path id="p1" style="stroke:none; stroke-width:2" d="M0,0"/></g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut group);

		let XMLNode::Element(path) = &group.children[0] else {
			panic!("expected path element")
		};
		assert_eq!(
			path.attributes.get("style").map(String::as_str),
			Some("stroke:none;")
		);
	}

	#[test]
	fn sanitize_preserves_group_stroke_none_in_inline_style() {
		let mut group = Element::parse(
			r#"<g id="g1" style="stroke:none; stroke-width:2"><path d="M0,0"/></g>"#.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut group);

		assert_eq!(
			group.attributes.get("style").map(String::as_str),
			Some("stroke:none;")
		);
	}

	#[test]
	fn sanitize_preserves_stroke_opacity_zero_in_inline_style() {
		let mut group = Element::parse(
			r#"<g id="g1"><path id="p1" style="stroke:#000000; stroke-opacity:0; stroke-width:2" d="M0,0"/></g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut group);

		let XMLNode::Element(path) = &group.children[0] else {
			panic!("expected path element")
		};
		assert_eq!(
			path.attributes.get("style").map(String::as_str),
			Some("stroke-opacity:0;")
		);
	}

	#[test]
	fn sanitize_preserves_non_black_fill_in_inline_style() {
		let mut group = Element::parse(
			r#"<g id="g1"><path id="p1" style="fill:#ff0000; stroke-width:2" d="M0,0"/></g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut group);

		let XMLNode::Element(path) = &group.children[0] else {
			panic!("expected path element")
		};
		assert_eq!(
			path.attributes.get("style").map(String::as_str),
			Some("fill:#ff0000;")
		);
	}

	#[test]
	fn sanitize_drops_black_and_none_fill_in_inline_style() {
		let mut black_fill = Element::parse(
			r#"<g id="g1"><path id="p1" style="fill:#000000; stroke-width:2" d="M0,0"/></g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut black_fill);

		let XMLNode::Element(black_path) = &black_fill.children[0] else {
			panic!("expected path element")
		};
		assert!(!black_path.attributes.contains_key("style"));

		let mut none_fill = Element::parse(
			r#"<g id="g2"><path id="p2" style="fill:none; stroke-width:2" d="M0,0"/></g>"#
				.as_bytes(),
		)
		.expect("inline group should parse");

		sanitize_shape_group_in_place(&mut none_fill);

		let XMLNode::Element(none_path) = &none_fill.children[0] else {
			panic!("expected path element")
		};
		assert!(!none_path.attributes.contains_key("style"));
	}
}
