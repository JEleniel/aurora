use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use xmltree::{Element, XMLNode};

use super::io::{read_svg_file, write_svg_file_compact};
use super::refs::{extract_url_ids, rewrite_references_in_element};
use super::shapes::{find_or_wrap_shape_group, sanitize_shape_group_in_place};
use super::util::{is_svg_file, local_name, strip_all_namespaces_in_place};

pub(super) fn optimize_icons_dir(input_dir: &Path, output_dir: &Path) -> Result<()> {
	optimize_svg_dir(input_dir, output_dir, SvgOptimizationKind::Icon)
}

pub(super) fn optimize_shapes_dir(input_dir: &Path, output_dir: &Path) -> Result<()> {
	optimize_svg_dir(input_dir, output_dir, SvgOptimizationKind::Shape)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SvgOptimizationKind {
	Icon,
	Shape,
}

fn optimize_svg_dir(input_dir: &Path, output_dir: &Path, kind: SvgOptimizationKind) -> Result<()> {
	if !input_dir.is_dir() {
		bail!("Input directory does not exist: {}", input_dir.display());
	}
	fs::create_dir_all(output_dir).with_context(|| {
		format!(
			"Failed creating output directory for optimized SVGs: {}",
			output_dir.display()
		)
	})?;

	let mut files = Vec::new();
	for entry in fs::read_dir(input_dir)
		.with_context(|| format!("Failed reading input directory: {}", input_dir.display()))?
	{
		let entry = entry.with_context(|| {
			format!(
				"Failed reading an entry from input directory: {}",
				input_dir.display()
			)
		})?;
		let path = entry.path();
		if path.is_file() && is_svg_file(&path) {
			files.push(path);
		}
	}

	files.sort_by(|a, b| {
		let a_name = a.file_name().and_then(OsStr::to_str).unwrap_or_default();
		let b_name = b.file_name().and_then(OsStr::to_str).unwrap_or_default();
		let a_ci = a_name.to_ascii_lowercase();
		let b_ci = b_name.to_ascii_lowercase();
		a_ci.cmp(&b_ci).then(a_name.cmp(b_name))
	});

	for path in files {
		let stem = path
			.file_stem()
			.and_then(OsStr::to_str)
			.map(str::to_owned)
			.with_context(|| {
				format!(
					"Failed extracting filename stem for SVG file: {}",
					path.display()
				)
			})?;
		let mut svg = read_svg_file(&path)?;

		match kind {
			SvgOptimizationKind::Icon => optimize_icon_svg_in_place(&mut svg, &stem)?,
			SvgOptimizationKind::Shape => optimize_shape_svg_in_place(&mut svg, &stem)?,
		}

		strip_inkscape_metadata(&mut svg);
		strip_all_namespaces_in_place(&mut svg);

		let out_path = output_dir.join(
			path.file_name()
				.and_then(OsStr::to_str)
				.unwrap_or("output.svg"),
		);
		write_svg_file_compact(&out_path, &svg)?;
	}

	Ok(())
}

fn optimize_icon_svg_in_place(svg: &mut Element, stem: &str) -> Result<()> {
	remove_layer_groups(svg);
	unwrap_trivial_root_groups(svg);
	let id_prefix = format!("i-{stem}");
	let referenced_ids = collect_referenced_ids(svg);
	remove_unreferenced_ids(svg, &referenced_ids, &[]);
	let id_map = prefix_remaining_ids(svg, &id_prefix);
	rewrite_references_in_element(svg, &id_map);
	Ok(())
}

fn optimize_shape_svg_in_place(svg: &mut Element, stem: &str) -> Result<()> {
	remove_layer_groups(svg);
	let mut group = find_or_wrap_shape_group(svg, stem);
	sanitize_shape_group_in_place(&mut group);

	// Replace root children with a single cleaned group.
	svg.children.clear();
	svg.children.push(XMLNode::Element(group));

	Ok(())
}

fn strip_inkscape_metadata(svg: &mut Element) {
	strip_inkscape_attributes(svg);
	strip_inkscape_children(svg);
}

fn strip_inkscape_attributes(element: &mut Element) {
	let mut to_remove = Vec::new();
	for key in element.attributes.keys() {
		if key.starts_with("inkscape:")
			|| key.starts_with("sodipodi:")
			|| key.eq_ignore_ascii_case("xmlns:inkscape")
			|| key.eq_ignore_ascii_case("xmlns:sodipodi")
			|| key.eq_ignore_ascii_case("xmlns:xlink")
		{
			to_remove.push(key.clone());
		}
		if key.eq_ignore_ascii_case("xlink:href") {
			to_remove.push(key.clone());
		}
	}
	for key in to_remove {
		if key.eq_ignore_ascii_case("xlink:href") {
			if let Some(value) = element.attributes.remove(&key) {
				element.attributes.insert("href".to_string(), value);
			}
		} else {
			element.attributes.remove(&key);
		}
	}

	for node in &mut element.children {
		if let XMLNode::Element(child) = node {
			strip_inkscape_attributes(child);
		}
	}
}

fn strip_inkscape_children(element: &mut Element) {
	element.children.retain(|node| match node {
		XMLNode::Element(child) => {
			let local = local_name(&child.name);
			!local.eq_ignore_ascii_case("namedview")
				&& !local.eq_ignore_ascii_case("metadata")
				&& !local.eq_ignore_ascii_case("desc")
				&& !local.eq_ignore_ascii_case("title")
		}
		XMLNode::Comment(_) => false,
		_ => true,
	});

	for node in &mut element.children {
		if let XMLNode::Element(child) = node {
			strip_inkscape_children(child);
		}
	}
}

fn remove_layer_groups(svg: &mut Element) {
	unwrap_matching_groups(svg, |el| {
		if !local_name(&el.name).eq_ignore_ascii_case("g") {
			return false;
		}
		if let Some(mode) = el.attributes.get("inkscape:groupmode")
			&& mode == "layer"
		{
			return true;
		}
		if let Some(id) = el.attributes.get("id") {
			return id.to_ascii_lowercase().starts_with("layer");
		}
		false
	});
}

fn unwrap_trivial_root_groups(svg: &mut Element) {
	// If the SVG has exactly one top-level group and nothing else important, unwrap it.
	let mut groups = Vec::new();
	let mut other = 0usize;
	for node in &svg.children {
		match node {
			XMLNode::Element(el) if local_name(&el.name).eq_ignore_ascii_case("g") => {
				groups.push(el)
			}
			XMLNode::Element(el) if local_name(&el.name).eq_ignore_ascii_case("defs") => other += 1,
			XMLNode::Element(el) if local_name(&el.name).eq_ignore_ascii_case("style") => {
				other += 1
			}
			XMLNode::Element(_) => other += 1,
			_ => {}
		}
	}
	if groups.len() == 1 && other == 1 {
		// One group + one defs/style; too risky to unwrap.
		return;
	}
	if groups.len() == 1 && other == 0 {
		let group = groups[0].clone();
		svg.children.clear();
		svg.children.extend(group.children);
	}
}

fn unwrap_matching_groups<F>(element: &mut Element, predicate: F)
where
	F: Fn(&Element) -> bool + Copy,
{
	let mut new_children = Vec::new();
	for node in std::mem::take(&mut element.children) {
		match node {
			XMLNode::Element(mut child) => {
				unwrap_matching_groups(&mut child, predicate);
				if predicate(&child) {
					new_children.extend(child.children);
				} else {
					new_children.push(XMLNode::Element(child));
				}
			}
			other => new_children.push(other),
		}
	}
	element.children = new_children;
}

fn collect_referenced_ids(element: &Element) -> Vec<String> {
	let mut ids = Vec::new();
	collect_referenced_ids_recursive(element, &mut ids);
	ids.sort();
	ids.dedup();
	ids
}

fn collect_referenced_ids_recursive(element: &Element, out: &mut Vec<String>) {
	for (name, value) in &element.attributes {
		let local = local_name(name);
		if local.eq_ignore_ascii_case("href")
			&& let Some(id) = value.strip_prefix('#')
		{
			out.push(id.to_string());
		}
		for id in extract_url_ids(value) {
			out.push(id);
		}
	}
	for node in &element.children {
		if let XMLNode::Element(child) = node {
			collect_referenced_ids_recursive(child, out);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use xmltree::EmitterConfig;

	#[test]
	fn optimize_shape_strips_inkscape_and_sodipodi_namespaces() {
		let mut svg = Element::parse(
			r#"<svg xmlns="http://www.w3.org/2000/svg"
				xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
				xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"
				viewBox="0 0 720 450">
				<sodipodi:namedview id="namedview" />
				<g id="shape-a" inkscape:label="shape-a">
					<path id="p" style="stroke-dasharray:4 2;stroke:#000" d="M0,0" />
				</g>
			</svg>"#
				.as_bytes(),
		)
		.expect("inline svg should parse");

		optimize_shape_svg_in_place(&mut svg, "shape-a").expect("shape optimization should work");
		strip_inkscape_metadata(&mut svg);
		strip_all_namespaces_in_place(&mut svg);

		let mut out = Vec::new();
		svg.write_with_config(
			&mut out,
			EmitterConfig::new()
				.perform_indent(false)
				.write_document_declaration(false),
		)
		.expect("optimized svg should serialize");
		let text = String::from_utf8(out).expect("svg should be utf-8");
		assert!(!text.contains("xmlns:inkscape"));
		assert!(!text.contains("xmlns:sodipodi"));
		assert!(text.contains("xmlns=\"http://www.w3.org/2000/svg\""));
	}
}

fn remove_unreferenced_ids(element: &mut Element, referenced: &[String], keep_ids: &[String]) {
	let keep = keep_ids
		.iter()
		.any(|id| element.attributes.get("id") == Some(id));
	if !keep
		&& let Some(id) = element.attributes.get("id")
		&& !referenced.iter().any(|x| x == id)
	{
		element.attributes.remove("id");
	}
	for node in &mut element.children {
		if let XMLNode::Element(child) = node {
			remove_unreferenced_ids(child, referenced, keep_ids);
		}
	}
}

fn prefix_remaining_ids(element: &mut Element, prefix: &str) -> HashMap<String, String> {
	let mut map = HashMap::new();
	prefix_remaining_ids_recursive(element, prefix, &mut map);
	map
}

fn prefix_remaining_ids_recursive(
	element: &mut Element,
	prefix: &str,
	map: &mut HashMap<String, String>,
) {
	if let Some(existing_id) = element.attributes.get("id").cloned() {
		let prefixed_id = format!("{prefix}-{existing_id}");
		element
			.attributes
			.insert("id".to_string(), prefixed_id.clone());
		map.insert(existing_id, prefixed_id);
	}
	for node in &mut element.children {
		if let XMLNode::Element(child) = node {
			prefix_remaining_ids_recursive(child, prefix, map);
		}
	}
}
