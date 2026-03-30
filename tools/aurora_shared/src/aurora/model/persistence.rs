pub(crate) fn sanitize_filename(name: &str) -> String {
	let mut out = String::new();
	let mut last_was_underscore = false;
	for ch in name.chars() {
		if ch.is_ascii_alphanumeric() {
			out.push(ch);
			last_was_underscore = false;
			continue;
		}
		if ch.is_whitespace() && !last_was_underscore {
			out.push('_');
			last_was_underscore = true;
		}
	}
	while out.contains("__") {
		out = out.replace("__", "_");
	}
	out.trim_matches('_').to_string()
}

pub(crate) fn sanitize_card_type_folder(card_type: &str) -> String {
	let mut out = String::new();
	let mut last_was_underscore = false;
	for ch in card_type.chars() {
		if ch.is_ascii_alphanumeric() || ch == '_' {
			out.push(ch);
			last_was_underscore = false;
			continue;
		}
		if ch.is_whitespace() && !last_was_underscore {
			out.push('_');
			last_was_underscore = true;
		}
	}
	while out.contains("__") {
		out = out.replace("__", "_");
	}
	out.trim_matches('_').to_string()
}
