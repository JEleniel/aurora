use std::fs;

use tempfile::TempDir;
use xmltree::{Element, XMLNode};

use super::run;
use crate::cli::{BuildArgs, Cli};

fn index_of(haystack: &str, needle: &str) -> usize {
	haystack
		.find(needle)
		.unwrap_or_else(|| panic!("Expected to find {needle:?} in output"))
}

fn tspan_text_lines(text_element: &Element) -> Vec<String> {
	text_element
		.children
		.iter()
		.filter_map(|node| match node {
			XMLNode::Element(element) if super::util::local_name(&element.name) == "tspan" => {
				Some(text_content(element))
			}
			_ => None,
		})
		.collect()
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

fn first_style_text(svg: &Element) -> Option<String> {
	svg.children.iter().find_map(|node| match node {
		XMLNode::Element(element) if super::util::local_name(&element.name) == "style" => {
			Some(text_content(element))
		}
		_ => None,
	})
}

fn assert_no_xmlns_attributes_below_root(svg: &Element) {
	for node in &svg.children {
		if let XMLNode::Element(element) = node {
			assert_no_xmlns_attributes_recursive(element);
		}
	}
}

fn assert_no_xmlns_attributes_recursive(element: &Element) {
	for (key, _) in &element.attributes {
		assert!(
			!key.eq_ignore_ascii_case("xmlns") && !key.to_ascii_lowercase().starts_with("xmlns:"),
			"Unexpected xmlns attribute on non-root element: {key}"
		);
	}
	for node in &element.children {
		if let XMLNode::Element(child) = node {
			assert_no_xmlns_attributes_recursive(child);
		}
	}
}

fn assert_no_attribute_local_name(svg: &Element, local: &str) {
	assert_no_attribute_local_name_recursive(svg, local);
}

fn assert_no_attribute_local_name_recursive(element: &Element, local: &str) {
	for (key, _) in &element.attributes {
		assert!(
			super::util::local_name(key).eq_ignore_ascii_case(local) == false,
			"Unexpected attribute '{key}' on element {}",
			element.name
		);
	}
	for node in &element.children {
		if let XMLNode::Element(child) = node {
			assert_no_attribute_local_name_recursive(child, local);
		}
	}
}

struct TestLayout {
	_dir: TempDir,
	masters_icons_dir: std::path::PathBuf,
	masters_shapes_dir: std::path::PathBuf,
	icons_dir: std::path::PathBuf,
	refs_dir: std::path::PathBuf,
	shapes_dir: std::path::PathBuf,
	template_path: std::path::PathBuf,
	icons_out: std::path::PathBuf,
	shapes_out: std::path::PathBuf,
	template_proof: std::path::PathBuf,
	model_configuration_path: std::path::PathBuf,
}

impl TestLayout {
	fn new() -> Self {
		let dir = TempDir::new().expect("temp dir should be created");
		let root = dir.path();
		let masters_icons_dir = root.join("assets/masters/icons");
		let masters_shapes_dir = root.join("assets/masters/shapes");
		let icons_dir = root.join("assets/optimized/icons");
		let proofs_dir = root.join("assets/proofs");
		let refs_dir = root.join("assets/references");
		let shapes_dir = root.join("assets/optimized/shapes");
		let model_configuration_dir = root.join(".github/aurora/reference");
		fs::create_dir_all(&masters_icons_dir).expect("masters icons dir should be created");
		fs::create_dir_all(&masters_shapes_dir).expect("masters shapes dir should be created");
		fs::create_dir_all(&icons_dir).expect("icons dir should be created");
		fs::create_dir_all(&shapes_dir).expect("shapes dir should be created");
		fs::create_dir_all(&proofs_dir).expect("proofs dir should be created");
		fs::create_dir_all(&refs_dir).expect("refs dir should be created");
		fs::create_dir_all(&model_configuration_dir)
			.expect("model configuration dir should be created");

		let template_path = refs_dir.join("SVGTemplate.svg");
		let icons_out = proofs_dir.join("Icons.svg");
		let shapes_out = proofs_dir.join("Shapes.svg");
		let template_proof = proofs_dir.join("SVGTemplate.svg");
		let model_configuration_path =
			model_configuration_dir.join("Aurora.modelconfiguration.json");

		Self {
			_dir: dir,
			masters_icons_dir,
			masters_shapes_dir,
			icons_dir,
			refs_dir,
			shapes_dir,
			template_path,
			icons_out,
			shapes_out,
			template_proof,
			model_configuration_path,
		}
	}

	fn run_build(&self) {
		let args = Cli {
			build: BuildArgs {
				masters_icons_in: self.masters_icons_dir.clone(),
				masters_shapes_in: self.masters_shapes_dir.clone(),
				icons_in: self.icons_dir.clone(),
				icons_proof: self.icons_out.clone(),
				shapes: self.shapes_dir.clone(),
				shapes_proof: self.shapes_out.clone(),
				template: self.template_path.clone(),
				template_out: None,
				template_proof: self.template_proof.clone(),
				template_svgz: None,
			},
			command: None,
		};

		run(&args).expect("svg_prep run should succeed");
	}

	fn write_model_configuration_with_available_cards(&self) {
		fs::write(
			&self.model_configuration_path,
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
	}
}

#[test]
fn build_pipeline_generates_icons_proof_sheet() {
	let layout = TestLayout::new();

	fs::write(
		layout.masters_icons_dir.join("beta.svg"),
		r#"<svg width="12" height="8" xmlns="http://www.w3.org/2000/svg">
			<title>Drop title</title>
			<g id="g-beta"><rect id="r1" x="0" y="0" width="12" height="8"/></g>
		</svg>"#,
	)
	.expect("beta icon should be written");
	fs::write(
		layout.masters_icons_dir.join("Alpha.svg"),
		r#"<svg viewBox="0 0 20 10" xmlns="http://www.w3.org/2000/svg">
			<metadata><source>ignore</source></metadata>
			<path id="p-alpha" d="M0,0 L20,10"/>
		</svg>"#,
	)
	.expect("alpha icon should be written");
	fs::write(
		layout.masters_icons_dir.join("alarm.svg"),
		r##"<svg width="12" height="12" xmlns="http://www.w3.org/2000/svg">
			<defs>
				<linearGradient id="SVGID_1_">
					<stop offset="0" stop-color="#000"/>
				</linearGradient>
			</defs>
			<path d="M0,0 L12,12" fill="url(#SVGID_1_)"/>
		</svg>"##,
	)
	.expect("alarm icon should be written");

	fs::write(
		layout.masters_shapes_dir.join("shape-b.svg"),
		r#"<svg xmlns="http://www.w3.org/2000/svg">
			<style>.aurora-symbol{}</style>
			<g id="shape-b"><path d="M0,0"/></g>
		</svg>"#,
	)
	.expect("shape-b master should be written");
	fs::write(
		layout.masters_shapes_dir.join("Shape-A.svg"),
		r#"<svg xmlns="http://www.w3.org/2000/svg">
			<style>.aurora-symbol{}</style>
			<g id="Shape-A"><path d="M1,1"/></g>
		</svg>"#,
	)
	.expect("Shape-A master should be written");

	fs::write(
		&layout.template_path,
		r#"<svg xmlns="http://www.w3.org/2000/svg"><style>.x{}</style><defs><g id="old"/></defs><g/></svg>"#,
	)
	.expect("template svg should be written");

	layout.write_model_configuration_with_available_cards();
	layout.run_build();

	let icons_output = fs::read_to_string(&layout.icons_out).expect("icons output should exist");
	assert_eq!(
		icons_output.matches("xmlns=\"").count(),
		1,
		"Icons proof should only declare the default xmlns on the root <svg>"
	);
	assert_eq!(
		icons_output.matches("xmlns:").count(),
		1,
		"Icons proof should only declare prefixed xmlns attributes on the root <svg>"
	);
	assert!(icons_output.contains("<defs>"));
	assert!(icons_output.contains("id=\"i-Alpha\""));
	assert!(icons_output.contains("id=\"i-alarm\""));
	assert!(icons_output.contains("id=\"i-beta\""));
	assert!(icons_output.contains("id=\"i-alarm-SVGID_1_\""));
	assert!(icons_output.contains("url(#i-alarm-SVGID_1_)"));
	assert!(!icons_output.contains("id=\"g-beta\""));
	assert!(!icons_output.contains("id=\"p-alpha\""));
	assert!(icons_output.contains("<rect"));
	assert!(icons_output.contains("fill=\"#FFFFFF\""));
	assert!(icons_output.contains("font-size:32px"));
	assert!(icons_output.contains("line-height:1.2"));
	assert!(icons_output.contains("aurora-proof-label"));
	assert!(!icons_output.contains("nodetypes"));

	let alpha_pos = index_of(&icons_output, "id=\"i-Alpha\"");
	let alarm_pos = index_of(&icons_output, "id=\"i-alarm\"");
	let beta_pos = index_of(&icons_output, "id=\"i-beta\"");
	assert!(alpha_pos < beta_pos);
	assert!(alarm_pos < beta_pos);
	assert_eq!(icons_output.matches("<use ").count(), 3);

	let icons_svg = Element::parse(icons_output.as_bytes()).expect("icons output should parse");
	assert_no_xmlns_attributes_below_root(&icons_svg);
	assert_no_attribute_local_name(&icons_svg, "nodetypes");
	let style_text = first_style_text(&icons_svg).expect("proof SVG should include a style block");
	assert!(style_text.contains("text.aurora-proof-label"));
	let text_nodes: Vec<&Element> = icons_svg
		.children
		.iter()
		.filter_map(|node| match node {
			XMLNode::Element(element) if super::util::local_name(&element.name) == "text" => {
				Some(element)
			}
			_ => None,
		})
		.collect();
	let rect_nodes: Vec<&Element> = icons_svg
		.children
		.iter()
		.filter_map(|node| match node {
			XMLNode::Element(element) if super::util::local_name(&element.name) == "rect" => {
				Some(element)
			}
			_ => None,
		})
		.collect();
	let use_nodes: Vec<&Element> = icons_svg
		.children
		.iter()
		.filter_map(|node| match node {
			XMLNode::Element(element) if super::util::local_name(&element.name) == "use" => {
				Some(element)
			}
			_ => None,
		})
		.collect();
	assert_eq!(rect_nodes.len(), 1);
	assert_eq!(
		rect_nodes[0].attributes.get("fill").map(String::as_str),
		Some("#FFFFFF")
	);
	assert_eq!(use_nodes.len(), 3);
	assert_eq!(text_nodes.len(), 3);
	assert_eq!(tspan_text_lines(text_nodes[0]), vec!["alarm"]);
	assert_eq!(tspan_text_lines(text_nodes[1]), vec!["Alpha"]);
	assert_eq!(tspan_text_lines(text_nodes[2]), vec!["beta"]);
	for text_node in text_nodes {
		assert_eq!(
			text_node.attributes.get("class").map(String::as_str),
			Some("aurora-proof-label")
		);
		assert!(
			!text_node.attributes.contains_key("style"),
			"proof label text should not use inline style attributes"
		);
		assert!(
			!text_node.attributes.contains_key("text-anchor"),
			"proof label text-anchor should be provided via the shared style block"
		);
		assert!(
			!text_node.attributes.contains_key("font-size"),
			"proof label font-size should be provided via the shared style block"
		);
		assert!(tspan_text_lines(text_node).len() <= 2);
	}

	let shapes_output = fs::read_to_string(&layout.shapes_out).expect("shapes output should exist");
	assert_eq!(
		shapes_output.matches("xmlns=\"").count(),
		1,
		"Shapes proof should only declare the default xmlns on the root <svg>"
	);
	assert_eq!(
		shapes_output.matches("xmlns:").count(),
		1,
		"Shapes proof should only declare prefixed xmlns attributes on the root <svg>"
	);
	assert!(shapes_output.contains("id=\"Shape-A\""));
	assert!(shapes_output.contains("id=\"shape-b\""));
	assert!(shapes_output.contains("href=\"#Shape-A\""));
	assert!(shapes_output.contains("href=\"#shape-b\""));
	assert_eq!(shapes_output.matches("<use ").count(), 2);
	assert!(shapes_output.contains("aurora-proof-label"));
	assert!(shapes_output.contains("g.aurora-symbol"));
	assert!(!shapes_output.contains("nodetypes"));

	let shapes_svg = Element::parse(shapes_output.as_bytes()).expect("shapes output should parse");
	assert_no_xmlns_attributes_below_root(&shapes_svg);
	assert_no_attribute_local_name(&shapes_svg, "nodetypes");
	assert_eq!(
		shapes_svg.attributes.get("width").map(String::as_str),
		Some("1476"),
		"With two 720px-wide shapes, the proof sheet should be sized in px (2 columns + 36px gap)"
	);
	assert_eq!(
		shapes_svg.attributes.get("height").map(String::as_str),
		Some("526.8"),
		"Shapes proof height should be 450px cell height plus label rows (2 lines)"
	);

	let defs_groups: Vec<&Element> = shapes_svg
		.children
		.iter()
		.filter_map(|node| match node {
			XMLNode::Element(element) if super::util::local_name(&element.name) == "defs" => {
				Some(element)
			}
			_ => None,
		})
		.flat_map(|defs| defs.children.iter())
		.filter_map(|node| match node {
			XMLNode::Element(element) if super::util::local_name(&element.name) == "g" => {
				Some(element)
			}
			_ => None,
		})
		.collect();
	assert!(
		!defs_groups.is_empty(),
		"Shapes proof should include <g> elements in <defs>"
	);
	for group in defs_groups {
		assert!(
			!group.attributes.contains_key("width") && !group.attributes.contains_key("height"),
			"Shape definition groups must not carry width/height attributes"
		);
	}

	let template_proof_output =
		fs::read_to_string(&layout.template_proof).expect("template proof should exist");
	assert!(template_proof_output.contains("id=\"i-Alpha\""));
}

#[test]
fn build_pipeline_does_not_overwrite_master_template_svg() {
	let layout = TestLayout::new();

	fs::write(
		layout.masters_icons_dir.join("Alpha.svg"),
		r#"<svg viewBox="0 0 20 10" xmlns="http://www.w3.org/2000/svg">
			<path d="M0,0 L20,10"/>
		</svg>"#,
	)
	.expect("alpha icon should be written");
	fs::write(
		layout.masters_shapes_dir.join("shape-a.svg"),
		r#"<svg xmlns="http://www.w3.org/2000/svg">
			<style>.aurora-symbol{}</style>
			<g id="shape-a"><path d="M0,0"/></g>
		</svg>"#,
	)
	.expect("shape master should be written");

	let master_template_path = layout._dir.path().join("assets/masters/SVGTemplate.svg");
	fs::write(
		&master_template_path,
		r#"<svg xmlns="http://www.w3.org/2000/svg"><style>.x{}</style><defs><g id="old"/></defs><g/></svg>"#,
	)
	.expect("master template svg should be written");
	layout.write_model_configuration_with_available_cards();

	let before = fs::read(&master_template_path).expect("master template should be readable");

	let args = Cli {
		build: BuildArgs {
			masters_icons_in: layout.masters_icons_dir.clone(),
			masters_shapes_in: layout.masters_shapes_dir.clone(),
			icons_in: layout.icons_dir.clone(),
			icons_proof: layout.icons_out.clone(),
			shapes: layout.shapes_dir.clone(),
			shapes_proof: layout.shapes_out.clone(),
			template: master_template_path.clone(),
			template_out: None,
			template_proof: layout.template_proof.clone(),
			template_svgz: None,
		},
		command: None,
	};

	run(&args).expect("svg_prep run should succeed");

	let after = fs::read(&master_template_path).expect("master template should be readable");
	assert_eq!(
		before, after,
		"Master template input should not be overwritten by the build pipeline"
	);

	let merged_template_path = layout._dir.path().join("assets/templates/SVGTemplate.svg");
	assert!(
		merged_template_path.is_file(),
		"Expected merged template output at {}",
		merged_template_path.display()
	);
	let merged_output =
		fs::read_to_string(&merged_template_path).expect("merged template should be readable");
	assert!(merged_output.contains("id=\"i-Alpha\""));
}

#[test]
fn build_pipeline_dedups_semantically_equivalent_style_blocks() {
	let layout = TestLayout::new();

	fs::write(
		layout.masters_icons_dir.join("alpha.svg"),
		r#"<svg width="12" height="8" xmlns="http://www.w3.org/2000/svg">
			<style>.dup{fill:#fff;stroke:#000}</style>
			<path d="M0,0 L1,1"/>
		</svg>"#,
	)
	.expect("alpha icon should be written");

	fs::write(
		layout.masters_icons_dir.join("beta.svg"),
		r#"<svg width="12" height="8" xmlns="http://www.w3.org/2000/svg">
			<style>.dup{stroke:#000; fill:#fff;}</style>
			<path d="M0,0 L1,1"/>
		</svg>"#,
	)
	.expect("beta icon should be written");

	fs::write(
		layout.masters_shapes_dir.join("shape-a.svg"),
		r#"<svg xmlns="http://www.w3.org/2000/svg"><g id="shape-a"><path d="M0,0"/></g></svg>"#,
	)
	.expect("shape master should be written");

	fs::write(
		&layout.template_path,
		r#"<svg xmlns="http://www.w3.org/2000/svg"><style>.template{a:b}</style><defs/></svg>"#,
	)
	.expect("template svg should be written");

	layout.write_model_configuration_with_available_cards();
	layout.run_build();

	let template_output = fs::read_to_string(&layout.template_path).expect("template should exist");
	let template_svg = Element::parse(template_output.as_bytes()).expect("template should parse");
	let style_text =
		first_style_text(&template_svg).expect("template should include a style block");

	assert_eq!(style_text.matches(".dup{").count(), 1);
}

#[test]
fn build_pipeline_updates_template_defs_and_available_cards() {
	let layout = TestLayout::new();

	fs::write(
		layout.masters_icons_dir.join("alarm.svg"),
		r##"<svg width="12" height="12" xmlns="http://www.w3.org/2000/svg">
			<defs>
				<linearGradient id="SVGID_1_"/>
			</defs>
			<path d="M0,0" fill="url(#SVGID_1_)"/>
		</svg>"##,
	)
	.expect("alarm icon should be written");
	fs::write(
		layout.masters_shapes_dir.join("shape-b.svg"),
		r#"<svg xmlns="http://www.w3.org/2000/svg"><g id="shape-b"><path d="M0,0"/></g></svg>"#,
	)
	.expect("shape-b master should be written");
	fs::write(
		layout.masters_shapes_dir.join("Shape-A.svg"),
		r#"<svg xmlns="http://www.w3.org/2000/svg"><g id="Shape-A"><path d="M1,1"/></g></svg>"#,
	)
	.expect("Shape-A master should be written");
	fs::write(
		&layout.template_path,
		r#"<svg xmlns="http://www.w3.org/2000/svg"><style>.x{}</style><defs><g id="old"/></defs><g/></svg>"#,
	)
	.expect("template svg should be written");

	layout.write_model_configuration_with_available_cards();
	layout.run_build();

	let template_output = fs::read_to_string(&layout.template_path).expect("template should exist");
	assert!(!template_output.contains("id=\"old\""));

	let i_alarm = index_of(&template_output, "id=\"i-alarm\"");
	let alarm_gradient = index_of(&template_output, "id=\"i-alarm-SVGID_1_\"");
	let shape_a = index_of(&template_output, "id=\"Shape-A\"");
	let shape_b = index_of(&template_output, "id=\"shape-b\"");
	assert!(i_alarm < alarm_gradient);
	assert!(alarm_gradient < shape_a);
	assert!(shape_a < shape_b);

	let model_configuration_output = fs::read_to_string(&layout.model_configuration_path)
		.expect("model configuration should exist");
	assert!(model_configuration_output.contains("\"available_cards\": ["));
	assert!(model_configuration_output.contains("\"alarm\""));
	assert!(!model_configuration_output.contains("\"legacy\""));
	assert!(!model_configuration_output.contains("\"alarm-SVGID_1_\""));

	let template_svgz_path = layout.template_path.with_extension("svgz");
	let svgz_bytes = fs::read(&template_svgz_path).expect("template svgz should exist");
	assert!(!svgz_bytes.is_empty());
}

#[test]
fn build_pipeline_imports_shape_group_by_inkscape_label() {
	let layout = TestLayout::new();

	fs::write(
		layout.icons_dir.join("alarm.svg"),
		r#"<svg width="12" height="12" xmlns="http://www.w3.org/2000/svg">
			<path d="M0,0"/>
		</svg>"#,
	)
	.expect("alarm icon should be written");

	let shapes_dir = layout.refs_dir.join("shapes");
	fs::create_dir_all(&shapes_dir).expect("shapes dir should be created");
	fs::write(
		shapes_dir.join("folder.svg"),
		r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape">
			<g id="g1" inkscape:label="folder"><path id="p1" style="fill-opacity:0" d="M0,0"/></g>
		</svg>"#,
	)
	.expect("shape should be written");

	fs::write(
		&layout.template_path,
		r#"<svg xmlns="http://www.w3.org/2000/svg"><defs/></svg>"#,
	)
	.expect("template svg should be written");
	layout.write_model_configuration_with_available_cards();

	// Point the build at the shapes directory so it loads each SVG and extracts its group.
	let args = Cli {
		build: BuildArgs {
			masters_icons_in: layout.masters_icons_dir.clone(),
			masters_shapes_in: layout.masters_shapes_dir.clone(),
			icons_in: layout.icons_dir.clone(),
			icons_proof: layout.icons_out.clone(),
			shapes: shapes_dir,
			shapes_proof: layout.shapes_out.clone(),
			template: layout.template_path.clone(),
			template_out: None,
			template_proof: layout.template_proof.clone(),
			template_svgz: None,
		},
		command: None,
	};

	run(&args).expect("svg_prep run should succeed");

	let template_output = fs::read_to_string(&layout.template_path).expect("template should exist");
	assert!(template_output.contains("id=\"folder\""));
	assert!(!template_output.contains("id=\"g1\""));
	assert!(!template_output.contains("inkscape:label=\"folder\""));
}

#[test]
fn run_returns_error_for_invalid_icon_svg() {
	let layout = TestLayout::new();

	fs::write(layout.icons_dir.join("bad.svg"), "<svg><broken></svg>")
		.expect("invalid svg should be written");
	fs::write(
		layout.masters_shapes_dir.join("shape-a.svg"),
		"<svg xmlns=\"http://www.w3.org/2000/svg\"><g id=\"shape-a\"/></svg>",
	)
	.expect("shapes should be written");
	fs::write(&layout.template_path, "<svg><defs/></svg>").expect("template should be written");
	layout.write_model_configuration_with_available_cards();

	let args = Cli {
		build: BuildArgs {
			masters_icons_in: layout.masters_icons_dir.clone(),
			masters_shapes_in: layout.masters_shapes_dir.clone(),
			icons_in: layout.icons_dir.clone(),
			icons_proof: layout.icons_out.clone(),
			shapes: layout.shapes_dir.clone(),
			shapes_proof: layout.shapes_out.clone(),
			template: layout.template_path.clone(),
			template_out: None,
			template_proof: layout.template_proof.clone(),
			template_svgz: None,
		},
		command: None,
	};

	let result = run(&args);
	assert!(result.is_err());
}

#[test]
fn proof_label_replaces_underscores_and_wraps_to_two_lines() {
	let label = super::proof::proof_label_for_icon_id("i-service_dependency_map");
	assert_eq!(label, "service dependency map");

	let lines = super::proof::wrap_proof_label(&label, 16, 2);
	assert_eq!(lines, vec!["service", "dependency map"]);
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
	super::defs::sort_defs_by_id(&mut defs);

	let ids: Vec<String> = defs
		.iter()
		.map(|el| el.attributes.get("id").cloned().unwrap_or_default())
		.collect();
	assert_eq!(ids, vec!["Alpha", "alpha", "beta"]);
}
