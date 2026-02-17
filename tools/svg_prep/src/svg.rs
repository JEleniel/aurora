//! SVG transformation routines for `svg_prep`.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use tracing::{debug, info};
use xmltree::{Element, EmitterConfig, XMLNode};

use crate::cli::Cli;

const EXCLUDED_TAGS: [&str; 6] = ["svg", "defs", "metadata", "title", "desc", "style"];
const MODEL_CONFIGURATION_FILE: &str = "Aurora.modelconfiguration.json";
const MODEL_CONFIGURATION_RELATIVE_PATH: &str =
	".github/agents/aurora/reference/Aurora.modelconfiguration.json";
const AVAILABLE_CARD_KEYS: [&str; 2] = ["available_cards", "available_icons"];

#[derive(Debug, Clone)]
struct IconGroup {
	id: String,
	group: Element,
	width: f64,
	height: f64,
}

pub fn run(args: &Cli) -> Result<()> {
	info!("Loading icon files from {}", args.icons_in.display());
	let icons = load_icon_groups(&args.icons_in)?;
	info!("Loaded {} icon file(s)", icons.len());

	info!("Building icon reference SVG");
	let icons_svg = build_icons_svg(&icons);
	info!("Writing icon reference SVG to {}", args.icons_out.display());
	write_svg_file(&args.icons_out, &icons_svg)?;

	info!("Extracting defs from icon reference SVG and shapes file");
	let mut combined_defs = extract_defs_elements(&icons_svg);
	let shapes_svg = read_svg_file(&args.shapes)?;
	combined_defs.extend(extract_defs_elements(&shapes_svg));
	debug!("Combined defs count before sort: {}", combined_defs.len());
	sort_defs_by_id(&mut combined_defs);
	debug!("Combined defs sorted by case-insensitive id");

	info!("Updating template defs in {}", args.template.display());
	let mut template_svg = read_svg_file(&args.template)?;
	replace_defs_section(&mut template_svg, combined_defs);
	write_svg_file(&args.template, &template_svg)?;
	info!("Template SVG updated successfully");

	info!("Synchronizing available_cards from template icon defs");
	sync_available_cards(&template_svg, &args.template)?;
	info!("available_cards synchronization complete");

	Ok(())
}

fn sync_available_cards(template_svg: &Element, template_path: &Path) -> Result<()> {
	let icon_names = extract_available_icon_names(template_svg);
	if icon_names.is_empty() {
		bail!(
			"No icon ids with the 'i-' prefix were found in {}",
			template_path.display()
		);
	}

	let model_configuration_path = resolve_model_configuration_path(template_path).with_context(|| {
		format!(
			"Could not locate {} near template path {} or relative to current directory",
			MODEL_CONFIGURATION_RELATIVE_PATH,
			template_path.display()
		)
	})?;
	info!(
		"Writing {} icon name(s) to {}",
		icon_names.len(),
		model_configuration_path.display()
	);

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
	debug!("Aurora model configuration file updated");

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
			if *byte == b'"' {
				in_string = false;
			}
			continue;
		}

		match *byte {
			b'"' => in_string = true,
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
		output.push('"');
		if index + 1 != icon_names.len() {
			output.push(',');
		}
		output.push('\n');
	}
	output.push_str("\t]");
	output
}

fn load_icon_groups(icons_dir: &Path) -> Result<Vec<IconGroup>> {
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

	files.sort_by(|a, b| {
		let a_name = a.file_name().and_then(OsStr::to_str).unwrap_or_default();
		let b_name = b.file_name().and_then(OsStr::to_str).unwrap_or_default();
		let a_ci = a_name.to_ascii_lowercase();
		let b_ci = b_name.to_ascii_lowercase();
		a_ci.cmp(&b_ci).then(a_name.cmp(b_name))
	});

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
		let children = extract_icon_children(&svg);
		let (width, height) = icon_dimensions(&svg);

		let mut group = Element::new("g");
		group
			.attributes
			.insert("id".to_string(), format!("i-{stem}"));
		group.children = children;

		icons.push(IconGroup {
			id: format!("i-{stem}"),
			group,
			width,
			height,
		});
	}

	Ok(icons)
}

fn build_icons_svg(icons: &[IconGroup]) -> Element {
	let mut root = Element::new("svg");

	let cell_width = icons
		.iter()
		.map(|icon| icon.width)
		.fold(0.0, f64::max)
		.max(1.0);
	let cell_height = icons
		.iter()
		.map(|icon| icon.height)
		.fold(0.0, f64::max)
		.max(1.0);

	let columns = if icons.is_empty() {
		1usize
	} else {
		(icons.len() as f64).sqrt().ceil() as usize
	};
	let rows = if icons.is_empty() {
		1usize
	} else {
		icons.len().div_ceil(columns)
	};

	let width = cell_width * columns as f64;
	let height = cell_height * rows as f64;

	root.attributes
		.insert("width".to_string(), format_number(width));
	root.attributes
		.insert("height".to_string(), format_number(height));
	root.attributes.insert(
		"viewBox".to_string(),
		format!("0 0 {} {}", format_number(width), format_number(height)),
	);
	root.attributes
		.insert("version".to_string(), "1.1".to_string());
	root.attributes
		.insert("id".to_string(), "Icons".to_string());
	root.attributes.insert(
		"xmlns".to_string(),
		"http://www.w3.org/2000/svg".to_string(),
	);
	root.attributes.insert(
		"xmlns:svg".to_string(),
		"http://www.w3.org/2000/svg".to_string(),
	);

	let mut defs = Element::new("defs");
	for icon in icons {
		defs.children.push(XMLNode::Element(icon.group.clone()));
	}
	root.children.push(XMLNode::Element(defs));

	for (index, icon) in icons.iter().enumerate() {
		let x = (index % columns) as f64 * cell_width;
		let y = (index / columns) as f64 * cell_height;

		let mut use_node = Element::new("use");
		use_node
			.attributes
			.insert("href".to_string(), format!("#{}", icon.id));
		use_node
			.attributes
			.insert("x".to_string(), format_number(x));
		use_node
			.attributes
			.insert("y".to_string(), format_number(y));
		use_node
			.attributes
			.insert("width".to_string(), format_number(cell_width));
		use_node
			.attributes
			.insert("height".to_string(), format_number(cell_height));
		root.children.push(XMLNode::Element(use_node));
	}

	root
}

fn extract_icon_children(root: &Element) -> Vec<XMLNode> {
	let mut output = Vec::new();
	for node in &root.children {
		if let XMLNode::Element(element) = node {
			if let Some(cleaned) = sanitize_element(element) {
				output.push(XMLNode::Element(cleaned));
			}
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
				if let Some(next) = sanitize_element(child) {
					cleaned.children.push(XMLNode::Element(next));
				}
			}
			_ => cleaned.children.push(node.clone()),
		}
	}

	Some(cleaned)
}

fn icon_dimensions(root: &Element) -> (f64, f64) {
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

fn parse_dimension(value: &str) -> Option<f64> {
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

fn parse_view_box(value: &str) -> Option<(f64, f64)> {
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

fn extract_defs_elements(svg: &Element) -> Vec<Element> {
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

fn replace_defs_section(svg: &mut Element, defs_children: Vec<Element>) {
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

fn sort_defs_by_id(elements: &mut [Element]) {
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

fn is_svg_file(path: &Path) -> bool {
	path.extension()
		.and_then(OsStr::to_str)
		.is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
}

fn is_excluded_tag(name: &str) -> bool {
	let local = local_name(name);
	EXCLUDED_TAGS
		.iter()
		.any(|candidate| local.eq_ignore_ascii_case(candidate))
}

fn is_defs_tag(name: &str) -> bool {
	local_name(name).eq_ignore_ascii_case("defs")
}

fn local_name(name: &str) -> &str {
	name.rsplit(':').next().unwrap_or(name)
}

fn read_svg_file(path: &Path) -> Result<Element> {
	let content =
		fs::read(path).with_context(|| format!("Failed reading SVG file: {}", path.display()))?;
	Element::parse(content.as_slice())
		.with_context(|| format!("Failed parsing SVG file: {}", path.display()))
}

fn write_svg_file(path: &Path, svg: &Element) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)
			.with_context(|| format!("Failed creating output directory: {}", parent.display()))?;
	}

	let mut output = Vec::new();
	svg.write_with_config(
		&mut output,
		EmitterConfig::new()
			.perform_indent(true)
			.write_document_declaration(true),
	)
	.with_context(|| format!("Failed serializing SVG output: {}", path.display()))?;
	if output.is_empty() {
		bail!("Serialization produced an empty SVG: {}", path.display());
	}
	fs::write(path, output).with_context(|| format!("Failed writing SVG file: {}", path.display()))
}

fn format_number(value: f64) -> String {
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

#[cfg(test)]
mod tests {
	use std::fs;

	use tempfile::TempDir;

	use super::*;

	fn parse_inline(svg: &str) -> Element {
		Element::parse(svg.as_bytes()).expect("inline SVG should parse")
	}

	fn index_of(haystack: &str, needle: &str) -> usize {
		haystack
			.find(needle)
			.unwrap_or_else(|| panic!("Expected to find {needle:?} in output"))
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

	#[test]
	fn defs_sort_is_case_insensitive_and_deterministic() {
		let mut a = Element::new("g");
		a.attributes.insert("id".into(), "beta".into());
		let mut b = Element::new("g");
		b.attributes.insert("id".into(), "Alpha".into());
		let mut c = Element::new("g");
		c.attributes.insert("id".into(), "alpha".into());

		let mut defs = vec![a, b, c];
		sort_defs_by_id(&mut defs);

		let ids: Vec<String> = defs
			.iter()
			.map(|el| el.attributes.get("id").cloned().unwrap_or_default())
			.collect();
		assert_eq!(ids, vec!["Alpha", "alpha", "beta"]);
	}

	#[test]
	fn run_generates_icons_and_template_outputs() {
		let dir = TempDir::new().expect("temp dir should be created");
		let root = dir.path();
		let icons_dir = root.join("assets/icons");
		let refs_dir = root.join("assets/references");
		let model_configuration_dir = root.join(".github/agents/aurora/reference");
		fs::create_dir_all(&icons_dir).expect("icons dir should be created");
		fs::create_dir_all(&refs_dir).expect("refs dir should be created");
		fs::create_dir_all(&model_configuration_dir)
			.expect("model configuration dir should be created");

		fs::write(
			icons_dir.join("beta.svg"),
			r#"<svg width="12" height="8" xmlns="http://www.w3.org/2000/svg">
				<title>Drop title</title>
				<g id="g-beta"><rect id="r1" x="0" y="0" width="12" height="8"/></g>
			</svg>"#,
		)
		.expect("beta icon should be written");
		fs::write(
			icons_dir.join("Alpha.svg"),
			r#"<svg viewBox="0 0 20 10" xmlns="http://www.w3.org/2000/svg">
				<metadata><source>ignore</source></metadata>
				<path id="p-alpha" d="M0,0 L20,10"/>
			</svg>"#,
		)
		.expect("alpha icon should be written");

		let shapes_path = refs_dir.join("Shapes.svg");
		let template_path = refs_dir.join("SVGTemplate.svg");
		let icons_out = refs_dir.join("Icons.svg");
		let model_configuration_path =
			model_configuration_dir.join("Aurora.modelconfiguration.json");

		fs::write(
			&shapes_path,
			r#"<svg xmlns="http://www.w3.org/2000/svg"><defs>
				<g id="shape-b"><path d="M0,0"/></g>
				<g id="Shape-A"><path d="M1,1"/></g>
			</defs></svg>"#,
		)
		.expect("shapes svg should be written");

		fs::write(
			&template_path,
			r#"<svg xmlns="http://www.w3.org/2000/svg"><style>.x{}</style><defs><g id="old"/></defs><g/></svg>"#,
		)
		.expect("template svg should be written");

		fs::write(
			&model_configuration_path,
			r##"{
	"$schema": "../schemas/Aurora.modelconfiguration.schema.json",
	"available_cards": ["legacy"],
	"cards": [
		{
			"acronym": "MIS",
			"card_type": "Mission"
		}
	],
	"views": []
}
"##,
		)
		.expect("model configuration json should be written");

		let args = Cli {
			icons_in: icons_dir,
			icons_out: icons_out.clone(),
			shapes: shapes_path,
			template: template_path.clone(),
		};

		run(&args).expect("svg_prep run should succeed");

		let icons_output = fs::read_to_string(&icons_out).expect("icons output should exist");
		assert!(icons_output.contains("<defs>"));
		assert!(icons_output.contains("id=\"i-Alpha\""));
		assert!(icons_output.contains("id=\"i-beta\""));
		assert!(!icons_output.contains("id=\"g-beta\""));
		assert!(!icons_output.contains("id=\"p-alpha\""));

		let alpha_pos = index_of(&icons_output, "id=\"i-Alpha\"");
		let beta_pos = index_of(&icons_output, "id=\"i-beta\"");
		assert!(alpha_pos < beta_pos);
		assert_eq!(icons_output.matches("<use ").count(), 2);

		let template_output = fs::read_to_string(&template_path).expect("template should exist");
		assert!(!template_output.contains("id=\"old\""));
		let i_alpha = index_of(&template_output, "id=\"i-Alpha\"");
		let i_beta = index_of(&template_output, "id=\"i-beta\"");
		let shape_a = index_of(&template_output, "id=\"Shape-A\"");
		let shape_b = index_of(&template_output, "id=\"shape-b\"");
		assert!(i_alpha < i_beta);
		assert!(i_beta < shape_a);
		assert!(shape_a < shape_b);

		let model_configuration_output =
			fs::read_to_string(&model_configuration_path).expect("model configuration should exist");
		assert!(model_configuration_output.contains("\"available_cards\": ["));
		assert!(model_configuration_output.contains("\"Alpha\""));
		assert!(model_configuration_output.contains("\"beta\""));
		assert!(!model_configuration_output.contains("\"legacy\""));
		assert!(!model_configuration_output.contains("\"shape-b\""));
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
				<symbol id="shape-rectangle"><path d="M0,0"/></symbol>
				<g id="i-wrench"><path d="M0,0"/></g>
				<g id="trapezoid"><path d="M0,0"/></g>
			</defs></svg>"#,
		);

		let names = extract_available_icon_names(&template);
		assert_eq!(names, vec!["alarm", "wrench"]);
	}

	#[test]
	fn run_returns_error_for_invalid_icon_svg() {
		let dir = TempDir::new().expect("temp dir should be created");
		let root = dir.path();
		let icons_dir = root.join("assets/icons");
		let refs_dir = root.join("assets/references");
		fs::create_dir_all(&icons_dir).expect("icons dir should be created");
		fs::create_dir_all(&refs_dir).expect("refs dir should be created");

		fs::write(icons_dir.join("bad.svg"), "<svg><broken></svg>")
			.expect("invalid svg should be written");
		let shapes_path = refs_dir.join("Shapes.svg");
		let template_path = refs_dir.join("SVGTemplate.svg");
		let icons_out = refs_dir.join("Icons.svg");
		fs::write(&shapes_path, "<svg><defs/></svg>").expect("shapes should be written");
		fs::write(&template_path, "<svg><defs/></svg>").expect("template should be written");

		let args = Cli {
			icons_in: icons_dir,
			icons_out,
			shapes: shapes_path,
			template: template_path,
		};

		let result = run(&args);
		assert!(result.is_err());
	}
}
