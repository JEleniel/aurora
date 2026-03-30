use xmltree::{Element, XMLNode};

use super::style::dedup_style_texts_semantic;
use super::types::IconGroup;
use super::util::format_number;

const PROOF_LABEL_FONT_SIZE_PX: f64 = 32.0;
const PROOF_LABEL_LINE_HEIGHT: f64 = 1.2;
const PROOF_LABEL_MAX_LINES: usize = 2;
const PROOF_LABEL_MIN_CHARS_PER_LINE: usize = 8;
const PROOF_LABEL_AVERAGE_CHAR_WIDTH_FACTOR: f64 = 0.55;
const PROOF_COLUMN_GAP_PX: f64 = 36.0;
const PROOF_LABEL_CLASS: &str = "aurora-proof-label";

// Shared shape styling emitted by the Aurora shape masters.
// This should live only in the root <style> block (not inside individual <g> definitions).
const SHARED_AURORA_SYMBOL_STYLE: &str = r#"g.aurora-symbol > circle,
g.aurora-symbol > ellipse,
g.aurora-symbol > line,
g.aurora-symbol > mpath,
g.aurora-symbol > path,
g.aurora-symbol > polygon,
g.aurora-symbol > polyline,
g.aurora-symbol > rect {
	fill:inherit;
	font-family: 'Noto Sans', Arial, Helvetica, sans-serif;
	font-size:16;
	line-height:1.2;
	stroke-linecap:round;
	stroke-linejoin:round;
	stroke-width:4;
	stroke:inherit;
}
text {
	text-anchor:middle;
}
.aurora-icon {}
.aurora-symbol {}"#;

pub(super) fn build_icons_svg(icons: &[IconGroup], root_id: &str) -> Element {
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
	let line_height_px = PROOF_LABEL_FONT_SIZE_PX * PROOF_LABEL_LINE_HEIGHT;
	let label_row_height = line_height_px * PROOF_LABEL_MAX_LINES as f64;
	let row_height = cell_height + label_row_height;
	let column_gap_total = PROOF_COLUMN_GAP_PX * columns.saturating_sub(1) as f64;

	let width = cell_width * columns as f64 + column_gap_total;
	let height = row_height * rows as f64;

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
		.insert("id".to_string(), root_id.to_string());
	root.attributes.insert(
		"xmlns".to_string(),
		"http://www.w3.org/2000/svg".to_string(),
	);
	root.attributes.insert(
		"xmlns:svg".to_string(),
		"http://www.w3.org/2000/svg".to_string(),
	);

	let mut style_texts: Vec<String> = Vec::new();
	style_texts.push(format!(
		"text.{PROOF_LABEL_CLASS} {{ line-height:{}; font-size:{}px; text-anchor:middle; dominant-baseline:middle; }}\ntext.{PROOF_LABEL_CLASS} tspan {{ dominant-baseline:middle; }}",
		format_number(PROOF_LABEL_LINE_HEIGHT),
		format_number(PROOF_LABEL_FONT_SIZE_PX)
	));
	if root_id == "Shapes" {
		// Shapes in proofs should be visible by default.
		style_texts.push("g.aurora-symbol { fill:none; stroke:#000; }".to_string());
		style_texts.push(SHARED_AURORA_SYMBOL_STYLE.to_string());
	}
	for icon in icons {
		for style in &icon.styles {
			let trimmed = style.trim();
			style_texts.extend((!trimmed.is_empty()).then(|| trimmed.to_string()));
		}
	}
	style_texts = dedup_style_texts_semantic(style_texts);
	if !style_texts.is_empty() {
		let mut style_el = Element::new("style");
		style_el
			.children
			.push(XMLNode::Text(style_texts.join("\n\n")));
		root.children.push(XMLNode::Element(style_el));
	}

	let mut defs = Element::new("defs");
	for icon in icons {
		for def in &icon.defs {
			defs.children.push(XMLNode::Element(def.clone()));
		}
	}
	for icon in icons {
		defs.children.push(XMLNode::Element(icon.group.clone()));
	}
	root.children.push(XMLNode::Element(defs));

	let mut background = Element::new("rect");
	background
		.attributes
		.insert("x".to_string(), "0".to_string());
	background
		.attributes
		.insert("y".to_string(), "0".to_string());
	background
		.attributes
		.insert("width".to_string(), format_number(width));
	background
		.attributes
		.insert("height".to_string(), format_number(height));
	background
		.attributes
		.insert("fill".to_string(), "#FFFFFF".to_string());
	root.children.push(XMLNode::Element(background));

	for (index, icon) in icons.iter().enumerate() {
		let x = (index % columns) as f64 * (cell_width + PROOF_COLUMN_GAP_PX);
		let y = (index / columns) as f64 * row_height;

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

		let mut label_node = Element::new("text");
		label_node
			.attributes
			.insert("x".to_string(), format_number(x + cell_width / 2.0));
		label_node.attributes.insert(
			"y".to_string(),
			format_number(y + cell_height + label_row_height / 2.0),
		);
		label_node
			.attributes
			.insert("class".to_string(), PROOF_LABEL_CLASS.to_string());

		let label = proof_label_for_icon_id(&icon.id);
		let max_chars_per_line = estimate_label_chars_per_line(cell_width);
		let wrapped_lines = wrap_proof_label(&label, max_chars_per_line, PROOF_LABEL_MAX_LINES);
		let wrapped_block_height = line_height_px * wrapped_lines.len().saturating_sub(1) as f64;
		let first_line_y = y + cell_height + label_row_height / 2.0 - wrapped_block_height / 2.0;

		for (line_index, line) in wrapped_lines.iter().enumerate() {
			let mut tspan = Element::new("tspan");
			tspan
				.attributes
				.insert("x".to_string(), format_number(x + cell_width / 2.0));
			tspan.attributes.insert(
				"y".to_string(),
				format_number(first_line_y + line_height_px * line_index as f64),
			);
			tspan.children.push(XMLNode::Text(line.clone()));
			label_node.children.push(XMLNode::Element(tspan));
		}

		root.children.push(XMLNode::Element(label_node));
	}

	root
}

pub(super) fn proof_label_for_icon_id(icon_id: &str) -> String {
	icon_id
		.strip_prefix("i-")
		.unwrap_or(icon_id)
		.replace('_', " ")
}

fn estimate_label_chars_per_line(cell_width: f64) -> usize {
	let estimated =
		(cell_width / (PROOF_LABEL_FONT_SIZE_PX * PROOF_LABEL_AVERAGE_CHAR_WIDTH_FACTOR)).floor();
	(estimated as usize).max(PROOF_LABEL_MIN_CHARS_PER_LINE)
}

fn place_word_at_line_start(
	current_line: &mut String,
	remaining_words: &mut Vec<String>,
	word: String,
	effective_max_chars: usize,
) {
	if word.chars().count() <= effective_max_chars {
		current_line.push_str(&word);
		remaining_words.remove(0);
		return;
	}

	let head: String = word.chars().take(effective_max_chars).collect();
	let tail: String = word.chars().skip(effective_max_chars).collect();
	current_line.push_str(&head);
	remaining_words.remove(0);
	remaining_words.extend((!tail.is_empty()).then_some(tail));
}

pub(super) fn wrap_proof_label(label: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
	if max_lines == 0 {
		return Vec::new();
	}

	let effective_max_chars = max_chars.max(1);
	let mut remaining_words: Vec<String> = label
		.split_whitespace()
		.filter(|word| !word.is_empty())
		.map(str::to_string)
		.collect();

	if remaining_words.is_empty() {
		return vec![String::new()];
	}

	let mut lines = Vec::new();
	let mut current_line = String::new();
	let mut truncated = false;

	while let Some(word) = remaining_words.first().cloned() {
		let current_len = current_line.chars().count();
		let word_len = word.chars().count();

		if current_len == 0 {
			place_word_at_line_start(
				&mut current_line,
				&mut remaining_words,
				word,
				effective_max_chars,
			);
			continue;
		}

		if current_len + 1 + word_len <= effective_max_chars {
			current_line.push(' ');
			current_line.push_str(&word);
			remaining_words.remove(0);
			continue;
		}

		lines.push(std::mem::take(&mut current_line));
		if lines.len() == max_lines {
			truncated = true;
			break;
		}
	}

	if !current_line.is_empty() && lines.len() < max_lines {
		lines.push(current_line);
	}

	if !remaining_words.is_empty() {
		truncated = true;
	}

	if lines.is_empty() {
		lines.push(String::new());
	}

	if truncated
		&& let Some(last_line) = lines.last_mut()
		&& !last_line.ends_with('…')
	{
		if last_line.chars().count() >= effective_max_chars {
			let keep = effective_max_chars.saturating_sub(1);
			let shortened: String = last_line.chars().take(keep).collect();
			*last_line = format!("{shortened}…");
		} else {
			last_line.push('…');
		}
	}

	lines.truncate(max_lines);
	lines
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn proof_label_replaces_underscores() {
		let label = proof_label_for_icon_id("i-service_dependency_map");
		assert_eq!(label, "service dependency map");
	}
}
