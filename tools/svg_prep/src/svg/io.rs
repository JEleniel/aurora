use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, bail};
use flate2::Compression;
use flate2::write::GzEncoder;
use xmltree::{Element, EmitterConfig};

pub(super) fn read_svg_file(path: &Path) -> Result<Element> {
	let content =
		fs::read(path).with_context(|| format!("Failed reading SVG file: {}", path.display()))?;
	Element::parse(content.as_slice())
		.with_context(|| format!("Failed parsing SVG file: {}", path.display()))
}

pub(super) fn write_svg_file(path: &Path, svg: &Element) -> Result<()> {
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

pub(super) fn write_svg_file_compact(path: &Path, svg: &Element) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)
			.with_context(|| format!("Failed creating output directory: {}", parent.display()))?;
	}

	let mut output = Vec::new();
	svg.write_with_config(
		&mut output,
		EmitterConfig::new()
			.perform_indent(false)
			.write_document_declaration(false),
	)
	.with_context(|| format!("Failed serializing SVG output: {}", path.display()))?;
	if output.is_empty() {
		bail!("Serialization produced an empty SVG: {}", path.display());
	}
	fs::write(path, output).with_context(|| format!("Failed writing SVG file: {}", path.display()))
}

pub(super) fn write_svgz_file(path: &Path, svg: &Element) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)
			.with_context(|| format!("Failed creating output directory: {}", parent.display()))?;
	}

	let mut raw = Vec::new();
	svg.write_with_config(
		&mut raw,
		EmitterConfig::new()
			.perform_indent(false)
			.write_document_declaration(true),
	)
	.with_context(|| format!("Failed serializing SVGZ output: {}", path.display()))?;
	if raw.is_empty() {
		bail!("Serialization produced an empty SVG: {}", path.display());
	}

	let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
	encoder
		.write_all(&raw)
		.with_context(|| format!("Failed compressing SVG template: {}", path.display()))?;
	let compressed = encoder
		.finish()
		.with_context(|| format!("Failed finishing SVGZ output: {}", path.display()))?;
	fs::write(path, compressed)
		.with_context(|| format!("Failed writing SVGZ file: {}", path.display()))
}
