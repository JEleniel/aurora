use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use xmltree::{Element, XMLNode};

use crate::cli::BuildArgs;

use super::defs::{extract_defs_elements, replace_defs_section, sort_defs_by_id};
use super::icons::{extract_style_texts, load_icon_groups};
use super::io::{read_svg_file, write_svg_file, write_svgz_file};
use super::optimize::{optimize_icons_dir, optimize_shapes_dir};
use super::proof::build_icons_svg;
use super::shapes::{
	load_shape_defs, load_shape_groups, sanitize_shape_group_in_place, shape_dimensions,
};
use super::style::dedup_style_texts_semantic;
use super::types::IconGroup;
use super::util::{
	is_defs_tag, local_name, remove_attribute_by_local_name_in_place,
	strip_non_root_namespaces_in_place,
};

const MODEL_CONFIGURATION_FILE: &str = "Aurora.modelconfiguration.json";
const MODEL_CONFIGURATION_RELATIVE_PATH: &str =
	".github/agents/aurora/reference/Aurora.modelconfiguration.json";
const AVAILABLE_CARD_KEYS: [&str; 2] = ["available_cards", "available_icons"];

pub(super) fn run_build(args: &BuildArgs) -> Result<()> {
	// Default pipeline: masters -> optimized -> proofs & template.
	if args.masters_icons_in.is_dir() {
		optimize_icons_dir(&args.masters_icons_in, &args.icons_in).with_context(|| {
			format!(
				"Failed optimizing icons from {} into {}",
				args.masters_icons_in.display(),
				args.icons_in.display()
			)
		})?;
	}
	if args.masters_shapes_in.is_dir() {
		if args
			.shapes
			.extension()
			.and_then(OsStr::to_str)
			.is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
		{
			// Keep supporting legacy single-file shapes inputs by skipping directory optimization.
		} else {
			optimize_shapes_dir(&args.masters_shapes_in, &args.shapes).with_context(|| {
				format!(
					"Failed optimizing shapes from {} into {}",
					args.masters_shapes_in.display(),
					args.shapes.display()
				)
			})?;
		}
	}

	let icons = load_icon_groups(&args.icons_in)?;
	let mut icons_svg = build_icons_svg(&icons, "Icons");
	strip_non_root_namespaces_in_place(&mut icons_svg);
	remove_attribute_by_local_name_in_place(&mut icons_svg, "nodetypes");
	write_svg_file(&args.icons_proof, &icons_svg)?;

	let shapes = load_shapes_for_proof(&args.shapes)?;
	let mut shapes_svg = build_icons_svg(&shapes, "Shapes");
	strip_non_root_namespaces_in_place(&mut shapes_svg);
	remove_attribute_by_local_name_in_place(&mut shapes_svg, "nodetypes");
	write_svg_file(&args.shapes_proof, &shapes_svg)?;

	let mut combined_defs = extract_defs_elements(&icons_svg);
	let shape_defs = load_shape_defs(&args.shapes)?;
	combined_defs.extend(shape_defs);
	sort_defs_by_id(&mut combined_defs);

	let template_out_path = resolve_template_out_path(args);
	let mut template_svg = read_svg_file(&args.template)?;
	merge_icon_styles_into_template(&mut template_svg, &icons);
	replace_defs_section(&mut template_svg, combined_defs);
	strip_non_root_namespaces_in_place(&mut template_svg);
	remove_attribute_by_local_name_in_place(&mut template_svg, "nodetypes");
	write_svg_file(&template_out_path, &template_svg)?;
	write_svg_file(&args.template_proof, &template_svg)?;

	let template_svgz_path = args
		.template_svgz
		.clone()
		.unwrap_or_else(|| derive_svgz_path(&template_out_path));
	write_svgz_file(&template_svgz_path, &template_svg)?;

	// Use the input template path for resolving the model configuration location.
	sync_available_cards(&template_svg, &args.template)?;
	Ok(())
}

fn resolve_template_out_path(args: &BuildArgs) -> PathBuf {
	if let Some(explicit) = args.template_out.as_ref() {
		return explicit.clone();
	}
	derive_template_out_path(&args.template)
}

fn derive_template_out_path(template_in: &Path) -> PathBuf {
	// Default behavior:
	// - If the template is under assets/masters/, write to assets/templates/.
	// - Otherwise, write in place.
	let mut out = PathBuf::new();
	let mut iter = template_in.components().peekable();
	while let Some(component) = iter.next() {
		out.push(component.as_os_str());
		if component.as_os_str() == "assets" {
			if let Some(next) = iter.peek()
				&& next.as_os_str() == "masters"
			{
				let _ = iter.next();
				out.push("templates");
			}
		}
	}

	// If we didn't rewrite assets/masters, keep the original path.
	// This avoids changing behavior for callers that manage their own template location.
	if out == template_in {
		return template_in.to_path_buf();
	}
	out
}

fn load_shapes_for_proof(path: &Path) -> Result<Vec<IconGroup>> {
	if path.is_dir() {
		return load_shape_groups(path);
	}

	let svg = read_svg_file(path)?;
	let (width, height) = shape_dimensions(&svg);
	let mut shapes = Vec::new();
	for mut element in extract_defs_elements(&svg) {
		if !local_name(&element.name).eq_ignore_ascii_case("g") {
			continue;
		}
		let Some(id) = element.attributes.get("id").cloned() else {
			continue;
		};
		sanitize_shape_group_in_place(&mut element);
		shapes.push(IconGroup {
			id,
			group: element,
			defs: Vec::new(),
			styles: Vec::new(),
			width,
			height,
		});
	}
	Ok(shapes)
}

pub(super) fn derive_svgz_path(template_svg_path: &Path) -> PathBuf {
	if let Some(ext) = template_svg_path.extension().and_then(OsStr::to_str)
		&& ext.eq_ignore_ascii_case("svg")
	{
		return template_svg_path.with_extension("svgz");
	}
	let mut svgz = template_svg_path.to_path_buf();
	svgz.set_extension("svgz");
	svgz
}

fn merge_icon_styles_into_template(template_svg: &mut Element, icons: &[super::types::IconGroup]) {
	let mut merged: Vec<String> = Vec::new();
	let existing = extract_style_texts(template_svg);
	for style in existing {
		merged.push(style);
	}
	for icon in icons {
		for style in &icon.styles {
			merged.push(style.clone());
		}
	}
	merged.retain(|s| !s.trim().is_empty());
	merged = dedup_style_texts_semantic(merged);
	if merged.is_empty() {
		return;
	}

	// Remove any existing top-level <style> elements.
	template_svg.children.retain(|node| {
		if let XMLNode::Element(element) = node {
			!local_name(&element.name).eq_ignore_ascii_case("style")
		} else {
			true
		}
	});

	let mut style_el = Element::new("style");
	let combined = merged.join("\n\n");
	style_el.children.push(XMLNode::Text(combined));

	// Insert before the first <defs> if present, else at the top.
	let insert_at = template_svg
		.children
		.iter()
		.position(|node| match node {
			XMLNode::Element(element) => is_defs_tag(&element.name),
			_ => false,
		})
		.unwrap_or(0);
	template_svg
		.children
		.insert(insert_at, XMLNode::Element(style_el));
}

fn sync_available_cards(template_svg: &Element, template_path: &Path) -> Result<()> {
	let icon_names = extract_available_icon_names(template_svg);
	if icon_names.is_empty() {
		bail!(
			"No icon ids with the 'i-' prefix were found in {}",
			template_path.display()
		);
	}

	let model_configuration_path =
		resolve_model_configuration_path(template_path).with_context(|| {
			format!(
				"Could not locate {} near template path {} or relative to current directory",
				MODEL_CONFIGURATION_RELATIVE_PATH,
				template_path.display()
			)
		})?;

	let source = fs::read_to_string(&model_configuration_path).with_context(|| {
		format!(
			"Failed reading Aurora model configuration file: {}",
			model_configuration_path.display()
		)
	})?;
	let updated = replace_available_cards_array(&source, &icon_names)?;
	fs::write(&model_configuration_path, updated).with_context(|| {
		format!(
			"Failed writing Aurora model configuration file: {}",
			model_configuration_path.display()
		)
	})?;
	Ok(())
}

fn resolve_model_configuration_path(template_path: &Path) -> Option<PathBuf> {
	if let Some(parent) = template_path.parent() {
		let sibling = parent.join(MODEL_CONFIGURATION_FILE);
		if sibling.is_file() {
			return Some(sibling);
		}
	}

	let relative = Path::new(MODEL_CONFIGURATION_RELATIVE_PATH);

	for ancestor in template_path.ancestors() {
		let candidate = ancestor.join(relative);
		if candidate.is_file() {
			return Some(candidate);
		}
	}

	let cwd = std::env::current_dir().ok()?;
	let candidate = cwd.join(relative);
	if candidate.is_file() {
		return Some(candidate);
	}

	None
}

fn extract_available_icon_names(template_svg: &Element) -> Vec<String> {
	let mut names = Vec::new();

	for element in extract_defs_elements(template_svg) {
		if !local_name(&element.name).eq_ignore_ascii_case("g") {
			continue;
		}
		let Some(id) = element.attributes.get("id") else {
			continue;
		};
		let Some(name) = id.strip_prefix("i-") else {
			continue;
		};
		if names.iter().any(|existing| existing == name) {
			continue;
		}
		names.push(name.to_string());
	}

	names
}

fn replace_available_cards_array(source: &str, icon_names: &[String]) -> Result<String> {
	let (key_name, key, key_pos) = AVAILABLE_CARD_KEYS
		.iter()
		.find_map(|name| {
			let key = format!("\"{name}\"");
			source.find(&key).map(|position| (*name, key, position))
		})
		.with_context(|| {
			"Missing \"available_cards\" or \"available_icons\" property in Aurora model configuration file"
				.to_string()
		})?;

	let colon_pos = source[key_pos + key.len()..]
		.find(':')
		.map(|offset| key_pos + key.len() + offset)
		.with_context(|| format!("Malformed \"{key_name}\" property (missing ':')"))?;

	let bytes = source.as_bytes();
	let mut array_start = colon_pos + 1;
	while array_start < bytes.len() && bytes[array_start].is_ascii_whitespace() {
		array_start += 1;
	}
	if array_start >= bytes.len() || bytes[array_start] != b'[' {
		bail!("Malformed \"{key_name}\" property (missing '[')");
	}

	let array_end = find_matching_bracket(source, array_start, key_name)?;
	let mut output = String::with_capacity(source.len() + icon_names.len() * 8);
	output.push_str(&source[..array_start]);
	output.push_str(&format_available_cards(icon_names));
	output.push_str(&source[array_end + 1..]);
	Ok(output)
}

fn find_matching_bracket(source: &str, start: usize, key_name: &str) -> Result<usize> {
	let bytes = source.as_bytes();
	let mut in_string = false;
	let mut escaped = false;
	let mut depth = 0usize;

	for (index, byte) in bytes.iter().enumerate().skip(start) {
		if in_string {
			if escaped {
				escaped = false;
				continue;
			}
			if *byte == b'\\' {
				escaped = true;
				continue;
			}
			if *byte == b'\"' {
				in_string = false;
			}
			continue;
		}

		match *byte {
			b'\"' => in_string = true,
			b'[' => depth += 1,
			b']' => {
				if depth == 0 {
					bail!("Malformed JSON while finding {key_name} array");
				}
				depth -= 1;
				if depth == 0 {
					return Ok(index);
				}
			}
			_ => {}
		}
	}

	bail!("Unterminated {key_name} array in Aurora model configuration file")
}

fn format_available_cards(icon_names: &[String]) -> String {
	let mut output = String::from("[\n");
	for (index, name) in icon_names.iter().enumerate() {
		output.push_str("\t\t\"");
		output.push_str(name);
		output.push('\"');
		if index + 1 != icon_names.len() {
			output.push(',');
		}
		output.push('\n');
	}
	output.push_str("\t]");
	output
}

#[cfg(test)]
mod tests {
	use super::*;

	fn parse_inline(svg: &str) -> Element {
		Element::parse(svg.as_bytes()).expect("inline SVG should parse")
	}

	#[test]
	fn replace_available_cards_array_rewrites_only_target_array() {
		let source = r##"{
	"$schema": "../schemas/Aurora.modelconfiguration.schema.json",
	"available_cards": ["old", "older"],
	"available_icons": ["legacy-icon"],
	"cards": [],
	"views": []
}
"##;

		let icons = vec!["alarm".to_string(), "wrench".to_string()];
		let updated = replace_available_cards_array(source, &icons)
			.expect("available_cards replacement should succeed");

		assert!(updated.contains("\"available_cards\": [\n\t\t\"alarm\",\n\t\t\"wrench\"\n\t]"));
		assert!(updated.contains("\"available_icons\": [\"legacy-icon\"]"));
		assert!(!updated.contains("\"old\""));
	}

	#[test]
	fn replace_available_cards_array_falls_back_to_available_icons() {
		let source = r##"{
	"$schema": "../schemas/Aurora.modelconfiguration.schema.json",
	"cards": [],
	"available_icons": ["old", "older"],
	"views": []
}
"##;

		let icons = vec!["alarm".to_string(), "wrench".to_string()];
		let updated = replace_available_cards_array(source, &icons)
			.expect("available_icons fallback replacement should succeed");

		assert!(updated.contains("\"available_icons\": [\n\t\t\"alarm\",\n\t\t\"wrench\"\n\t]"));
		assert!(!updated.contains("\"old\""));
	}

	#[test]
	fn extract_available_icon_names_uses_only_i_prefix_ids() {
		let template = parse_inline(
			r#"<svg xmlns="http://www.w3.org/2000/svg"><defs>
				<g id="i-alarm"><path d="M0,0"/></g>
				<linearGradient id="i-alarm-SVGID_1_"/>
				<symbol id="shape-rectangle"><path d="M0,0"/></symbol>
				<g id="i-wrench"><path d="M0,0"/></g>
				<g id="trapezoid"><path d="M0,0"/></g>
			</defs></svg>"#,
		);

		let names = extract_available_icon_names(&template);
		assert_eq!(names, vec!["alarm", "wrench"]);
	}
}
