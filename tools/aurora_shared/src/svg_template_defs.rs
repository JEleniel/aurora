//! Read-only extraction of named icon and shape assets from Aurora SVG templates.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

use flate2::read::GzDecoder;
use thiserror::Error;

/// Named SVG asset IDs discovered in the template `<defs>` section.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SvgTemplateDefs {
	pub icon_ids: BTreeSet<String>,
	pub shape_ids: BTreeSet<String>,
}

impl SvgTemplateDefs {
	/// Load the Aurora SVG template from a model home and extract named defs.
	pub fn load_from_model_home(model_home: &Path) -> Result<Self, SvgTemplateDefsError> {
		let reference_dir = model_home.join("reference");
		let svgz_path = reference_dir.join("SVGTemplate.svgz");
		let svg_path = reference_dir.join("SVGTemplate.svg");
		let svg = read_svg_template(&svgz_path, &svg_path)?;
		Ok(Self::parse(&svg))
	}

	/// Parse a template string and extract icon and shape asset IDs from `<defs>`.
	pub fn parse(svg_template: &str) -> Self {
		let Some(defs_content) = defs_content(svg_template) else {
			return Self::default();
		};

		let mut defs = Self::default();
		let mut cursor = 0usize;
		while let Some(group_start) = find_next_group_start(defs_content, cursor) {
			let Some(group_end_rel) = defs_content[group_start..].find('>') else {
				break;
			};
			let group_end = group_start + group_end_rel;
			let group_tag = &defs_content[group_start..=group_end];
			if let Some(id) = extract_attribute_value(group_tag, "id") {
				if let Some(icon_id) = normalize_icon_id(id.as_str()) {
					defs.icon_ids.insert(icon_id);
				} else {
					defs.shape_ids.insert(id);
				}
			}
			cursor = group_end + 1;
		}

		defs
	}
}

fn read_svg_template(svgz_path: &Path, svg_path: &Path) -> Result<String, SvgTemplateDefsError> {
	if svgz_path.is_file() {
		let file = std::fs::File::open(svgz_path)?;
		let mut decoder = GzDecoder::new(file);
		let mut out = String::new();
		decoder.read_to_string(&mut out)?;
		return Ok(out);
	}

	if svg_path.is_file() {
		return Ok(std::fs::read_to_string(svg_path)?);
	}

	Err(SvgTemplateDefsError::TemplateMissing(format!(
		"{} (or {})",
		svgz_path.display(),
		svg_path.display()
	)))
}

fn defs_content(svg_template: &str) -> Option<&str> {
	let defs_start = svg_template.find("<defs")?;
	let defs_open_end_rel = svg_template[defs_start..].find('>')?;
	let content_start = defs_start + defs_open_end_rel + 1;
	let defs_close_rel = svg_template[content_start..].find("</defs>")?;
	let content_end = content_start + defs_close_rel;
	Some(&svg_template[content_start..content_end])
}

fn find_next_group_start(content: &str, from: usize) -> Option<usize> {
	let mut cursor = from;
	while let Some(rel) = content[cursor..].find("<g") {
		let idx = cursor + rel;
		let next = content[idx + 2..].chars().next();
		if matches!(
			next,
			Some(' ') | Some('\t') | Some('\n') | Some('\r') | Some('>')
		) {
			return Some(idx);
		}
		cursor = idx + 2;
	}
	None
}

fn extract_attribute_value(tag: &str, attribute: &str) -> Option<String> {
	for quote in ['"', '\''] {
		let needle = format!("{}={}", attribute, quote);
		if let Some(start_rel) = tag.find(needle.as_str()) {
			let start = start_rel + needle.len();
			let rest = &tag[start..];
			if let Some(end_rel) = rest.find(quote) {
				let value = &rest[..end_rel];
				if !value.is_empty() {
					return Some(value.to_string());
				}
			}
		}
	}
	None
}

fn normalize_icon_id(id: &str) -> Option<String> {
	id.strip_prefix("i-")
		.filter(|value| !value.is_empty())
		.map(ToString::to_string)
}

/// Errors raised while reading or extracting defs from the Aurora SVG template.
#[derive(Debug, Error)]
pub enum SvgTemplateDefsError {
	#[error("SVG template I/O error: {0}")]
	Io(#[from] std::io::Error),
	#[error("SVG template is missing: {0}")]
	TemplateMissing(String),
}

#[cfg(test)]
mod tests {
	use super::{SvgTemplateDefs, SvgTemplateDefsError};
	use flate2::Compression;
	use flate2::write::GzEncoder;
	use std::io::Write;
	use std::path::PathBuf;

	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	fn read_testdata(rel_path: &str) -> String {
		let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("src")
			.join("testdata")
			.join(rel_path);
		std::fs::read_to_string(&path)
			.unwrap_or_else(|e| panic!("failed to read testdata file {}: {e}", path.display()))
	}

	#[test]
	fn parse_extracts_icon_and_shape_ids_from_defs() {
		let template = read_testdata("svg/template_defs_fixture.svg");
		let defs = SvgTemplateDefs::parse(&template);

		assert_eq!(
			defs.shape_ids.into_iter().collect::<Vec<_>>(),
			vec![
				"double-rectangle".to_string(),
				"hexagon".to_string(),
				"rectangle".to_string(),
			]
		);
		assert_eq!(
			defs.icon_ids.into_iter().collect::<Vec<_>>(),
			vec!["gear".to_string(), "wrench".to_string()]
		);
	}

	#[test]
	fn load_from_model_home_reads_svgz_template() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = temp.path().join("aurora");
		let reference_dir = model_home.join("reference");
		std::fs::create_dir_all(&reference_dir)?;

		let template = read_testdata("svg/template_defs_fixture.svg");
		let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
		encoder.write_all(template.as_bytes())?;
		let data = encoder.finish()?;
		std::fs::write(reference_dir.join("SVGTemplate.svgz"), data)?;

		let defs = SvgTemplateDefs::load_from_model_home(&model_home)?;
		assert!(defs.shape_ids.contains("rectangle"));
		assert!(defs.shape_ids.contains("hexagon"));
		assert!(defs.icon_ids.contains("wrench"));
		assert!(defs.icon_ids.contains("gear"));
		Ok(())
	}

	#[test]
	fn load_from_model_home_reports_missing_template() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = temp.path().join("aurora");
		std::fs::create_dir_all(model_home.join("reference"))?;

		let error = SvgTemplateDefs::load_from_model_home(&model_home).unwrap_err();
		assert!(matches!(error, SvgTemplateDefsError::TemplateMissing(_)));
		Ok(())
	}
}
