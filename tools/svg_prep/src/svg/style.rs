use std::collections::{BTreeMap, BTreeSet, HashSet};

/// Deduplicate style blocks by semantic equality rather than raw string equality.
///
/// This is intentionally a conservative CSS normalizer: it supports typical `selector { a:b; }`
/// rules that appear in the icon/shape sources. If parsing fails, it falls back to a whitespace-
/// normalized representation.
pub(super) fn dedup_style_texts_semantic(style_texts: Vec<String>) -> Vec<String> {
	let mut seen = HashSet::new();
	let mut out = Vec::new();

	for style in style_texts {
		let key = canonicalize_css(&style);
		if seen.insert(key) {
			out.push(style);
		}
	}

	out
}

fn canonicalize_css(css: &str) -> String {
	let stripped = strip_css_comments(css);
	match parse_rules(&stripped) {
		Some(rules) if !rules.is_empty() => canonicalize_rules(&rules),
		_ => collapse_whitespace(&stripped).to_string(),
	}
}

fn strip_css_comments(input: &str) -> String {
	let mut out = String::with_capacity(input.len());
	let bytes = input.as_bytes();
	let mut i = 0usize;

	while i < bytes.len() {
		if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
			i += 2;
			while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
				i += 1;
			}
			if i + 1 < bytes.len() {
				i += 2;
			}
			continue;
		}

		out.push(bytes[i] as char);
		i += 1;
	}

	out
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CssRuleKey {
	selectors: Vec<String>,
	decls: BTreeMap<String, Vec<String>>,
}

fn parse_rules(css: &str) -> Option<Vec<CssRuleKey>> {
	let mut rules = Vec::new();
	let mut cursor = 0usize;
	let bytes = css.as_bytes();

	while cursor < bytes.len() {
		let open = css[cursor..].find('{')? + cursor;
		let selector_part = css[cursor..open].trim();
		let close = find_matching_close_brace(css, open + 1)?;
		let body = &css[open + 1..close];

		if !selector_part.is_empty() {
			let selectors = normalize_selectors(selector_part);
			let decls = normalize_declarations(body);
			rules.push(CssRuleKey { selectors, decls });
		}

		cursor = close + 1;
	}

	Some(rules)
}

fn find_matching_close_brace(css: &str, start: usize) -> Option<usize> {
	let mut depth = 1usize;
	for (offset, ch) in css[start..].chars().enumerate() {
		match ch {
			'{' => depth += 1,
			'}' => {
				depth = depth.saturating_sub(1);
				if depth == 0 {
					return Some(start + offset);
				}
			}
			_ => {}
		}
	}
	None
}

fn normalize_selectors(selectors: &str) -> Vec<String> {
	let mut out: Vec<String> = selectors
		.split(',')
		.map(|part| collapse_whitespace(part).trim().to_string())
		.filter(|part| !part.is_empty())
		.collect();
	out.sort();
	out.dedup();
	out
}

fn normalize_declarations(body: &str) -> BTreeMap<String, Vec<String>> {
	let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

	for raw in body.split(';') {
		let decl = raw.trim();
		if decl.is_empty() {
			continue;
		}
		let Some((prop, val)) = decl.split_once(':') else {
			continue;
		};
		let prop = prop.trim().to_ascii_lowercase();
		let val = collapse_whitespace(val).trim().to_string();
		if prop.is_empty() || val.is_empty() {
			continue;
		}
		map.entry(prop).or_default().insert(val);
	}

	map.into_iter()
		.map(|(k, v)| (k, v.into_iter().collect()))
		.collect()
}

fn canonicalize_rules(rules: &[CssRuleKey]) -> String {
	let mut sorted = rules.to_vec();
	sorted.sort();

	let mut out = String::new();
	for rule in sorted {
		out.push_str(&rule.selectors.join(","));
		out.push('{');
		for (prop, values) in rule.decls {
			for value in values {
				out.push_str(&prop);
				out.push(':');
				out.push_str(&value);
				out.push(';');
			}
		}
		out.push('}');
	}

	out
}

fn collapse_whitespace(input: &str) -> String {
	let mut out = String::with_capacity(input.len());
	let mut prev_space = false;

	for ch in input.chars() {
		if ch.is_ascii_whitespace() {
			if !prev_space {
				out.push(' ');
				prev_space = true;
			}
			continue;
		}
		prev_space = false;
		out.push(ch);
	}

	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn dedup_style_texts_semantic_ignores_decl_order() {
		let styles = vec![
			".x{fill:#fff;stroke:#000}".to_string(),
			".x{stroke:#000; fill:#fff;}".to_string(),
		];
		let deduped = dedup_style_texts_semantic(styles);
		assert_eq!(deduped.len(), 1);
	}
}
