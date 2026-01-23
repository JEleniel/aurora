//! In-memory representation of Aurora cards and models.
mod card;
mod compact_card;
mod compact_model;
mod readme_template;
mod view_definitions;
mod view_template;

pub use card::*;
pub use compact_model::{CompactModel, CompactModelBorrowed, CompactModelError};
use jsonschema::Validator;
use log::debug;
use regex::Regex;
use std::{
	collections::{HashMap, HashSet},
	fs,
	io::Write,
	path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{
	aurora::model::{card::prefixes, readme_template::MODEL_TEMPLATE},
	cli::{bump_args::BumpArgs, output_args::OutputArgs},
};

/// Fully materialized Aurora model state.
#[derive(Debug, Clone)]
pub struct Model {
	pub mission_card: Card,
	pub compact_model: Option<CompactModel>,
	pub cards: HashMap<String, Card>,
	pub adjacency: HashMap<String, Vec<String>>,
	pub card_schema_errors: Vec<String>,
	pub card_invariant_errors: Vec<String>,
}

impl Model {
	pub fn sanitize_name(name: &str) -> String {
		let mut sanitized: String = name
			.trim()
			.chars()
			.map(|ch| match ch {
				' ' => '_',
				'\t' | '\n' | '\r' => '_',
				c if c.is_ascii_alphanumeric() || c == '_' || c == '-' => c,
				_ => '_',
			})
			.collect();
		if sanitized.is_empty() {
			sanitized.push_str("Mission");
		}
		sanitized
	}

	pub fn load(
		path: &Path,
		card_validator: &Validator,
		compact_validator: &Validator,
		mission_filter: Option<&str>,
	) -> Result<Vec<Self>, ModelError> {
		let mut models: Vec<Model> = Self::load_models(path, card_validator, mission_filter)?;
		if models.is_empty() {
			return Err(ModelError::ModelNotFound);
		}

		for model in models.iter_mut() {
			let mission_path = path.join(&model.mission_card.id);
			if !mission_path.is_dir() {
				return Err(ModelError::ModelPathNotFound(
					mission_path.display().to_string(),
				));
			}
			Self::load_cards(model, &mission_path, card_validator)?;
			model.compact_model =
				Self::load_compact(path, &model.mission_card.id, compact_validator)?;
		}

		Ok(models)
	}

	pub fn bump_patch(&mut self, args: &BumpArgs) -> Result<(), ModelError> {
		let card = self
			.cards
			.get_mut(&args.card_id)
			.ok_or(ModelError::CardNotFound(args.card_id.clone()))?;
		let user = whoami::account().unwrap_or_else(|_| "unknown".to_string());
		card.bump_patch(&args.editor.clone().unwrap_or(user))?;
		Ok(())
	}

	pub fn bump_minor(&mut self, args: &BumpArgs) -> Result<(), ModelError> {
		let card = self
			.cards
			.get_mut(&args.card_id)
			.ok_or(ModelError::CardNotFound(args.card_id.clone()))?;
		let user = whoami::account().unwrap_or_else(|_| "unknown".to_string());
		card.bump_minor(&args.editor.clone().unwrap_or(user))?;
		Ok(())
	}

	pub fn bump_major(&mut self, args: &BumpArgs) -> Result<(), ModelError> {
		let card = self
			.cards
			.get_mut(&args.card_id)
			.ok_or(ModelError::CardNotFound(args.card_id.clone()))?;
		let user = whoami::account().unwrap_or_else(|_| "unknown".to_string());
		card.bump_major(&args.editor.clone().unwrap_or(user))?;
		Ok(())
	}

	pub fn render_markdown(&self, args: &OutputArgs) -> Result<(), ModelError> {
		if args.clear {
			Self::clear_output_dir(&args.output_path)?;
		}

		let mut markdown: String = MODEL_TEMPLATE.to_string();

		markdown = markdown.replace("{{id}}", &self.mission_card.id);
		markdown = markdown.replace("{{name}}", &self.mission_card.name);
		markdown = markdown.replace("{{description}}", &self.mission_card.description);

		let views_found = self.find_views(args);
		markdown = markdown.replace("{{views}}", &views_found);

		let cards = self.build_card_index();
		markdown = markdown.replace("{{cards}}", &cards);

		debug!("Writing README for model {}", self.mission_card.id);
		let readme_path = args
			.output_path
			.join(format!("README-{}.md", self.mission_card.id));
		let mut file = std::fs::File::create(&readme_path)?;
		file.write_all(markdown.as_bytes())?;
		debug!("Wrote README to {}", readme_path.display());

		self.write_cards_markdown(args)?;

		Ok(())
	}

	pub fn render_views(&self, args: &OutputArgs) -> Result<(), ModelError> {
		for (view_name, view_definition) in view_definitions::get_definitions() {
			let view_path = args.output_path.join(&self.mission_card.id);
			fs::create_dir_all(&view_path)?;

			let view_path = args
				.output_path
				.join(&self.mission_card.id)
				.join(format!("{}.view.md", view_name.replace(" ", "_")));
			debug!(
				"Rendering view {} for model {}",
				view_name, self.mission_card.id
			);

			let mut view = view_template::VIEW_TEMPLATE.to_string();
			view = view.replace("{{id}}", &self.mission_card.id);
			view = view.replace("{{name}}", &self.mission_card.name);
			view = view.replace("{{view_name}}", view_name);

			// Gether all the cards to be included in the view
			let mut cards: Vec<&Card> = Vec::new();
			for card_type in view_definition.root_card_types {
				if card_type == self.mission_card.card_type {
					cards.push(&self.mission_card);
				}
				cards.append(&mut self.add_cards_by_type(card_type)?);
			}
			for card_type in view_definition.include_card_types {
				if card_type == self.mission_card.card_type {
					cards.push(&self.mission_card);
				}
				cards.append(&mut self.add_cards_by_type(card_type)?);
			}

			let mut view_cards: HashMap<String, &Card> = HashMap::new();
			for card in cards {
				view_cards.entry(card.id.clone()).or_insert(card);
			}
			self.expand_view_cards_with_special_children(&mut view_cards)?;
			let mut cards: Vec<&Card> = view_cards.values().copied().collect();
			cards.sort_by(|left, right| left.id.cmp(&right.id));

			let mut boundary_members: HashSet<String> = HashSet::new();
			let mut rendered_boundaries: HashSet<String> = HashSet::new();
			let mut boundaries = String::new();
			for card in &cards {
				if Self::is_boundary_card(card) {
					let boundary = self.render_boundary_subgraph(
						card,
						&view_cards,
						&mut boundary_members,
						&mut rendered_boundaries,
						1,
					)?;
					if !boundary.is_empty() {
						boundaries.push_str(&boundary);
					}
				}
			}

			let nodes = Self::render_view_nodes(&cards, &boundary_members);
			let edges = self.render_view_edges(&cards, &view_cards);
			let classes = Self::render_classes(&cards);

			view = view.replace("{{boundaries}}", &boundaries);
			view = view.replace("{{nodes}}", &nodes);
			view = view.replace("{{edges}}", &edges);
			view = view.replace("{{class_mappings}}", &classes);

			debug!("Writing view file for model {}", &view_path.display());
			let mut file = std::fs::File::create(&view_path)?;
			file.write_all(view.as_bytes())?;
			debug!("Wrote view to {}", view_path.display());
		}

		Ok(())
	}

	fn expand_view_cards_with_special_children<'a>(
		&'a self,
		view_cards: &mut HashMap<String, &'a Card>,
	) -> Result<(), ModelError> {
		let mut changed = true;
		while changed {
			changed = false;
			let view_ids: Vec<String> = view_cards.keys().cloned().collect();
			for view_id in view_ids {
				let Some(card) = view_cards.get(&view_id).copied() else {
					continue;
				};
				let Some(links) = &card.links else {
					continue;
				};
				for link in links {
					if !Self::is_special_child_id(&link.target) {
						continue;
					}
					let special_card = self
						.cards
						.get(&link.target)
						.ok_or(ModelError::CardNotFound(link.target.clone()))?;
					if !Self::is_boundary_card(special_card) && !Self::is_note_card(special_card) {
						continue;
					}
					if view_cards.contains_key(&special_card.id) {
						continue;
					}
					view_cards.insert(special_card.id.clone(), special_card);
					changed = true;
				}
			}
		}
		Ok(())
	}

	fn is_special_child_id(card_id: &str) -> bool {
		card_id.starts_with("BND") || card_id.starts_with("BOU") || card_id.starts_with("NOT")
	}

	fn is_boundary_card(card: &Card) -> bool {
		card.card_type == "Boundary" || card.id.starts_with("BND") || card.id.starts_with("BOU")
	}

	fn is_note_card(card: &Card) -> bool {
		card.card_type == "Note" || card.id.starts_with("NOT")
	}

	fn boundary_is_recursive(card: &Card) -> bool {
		card.attributes
			.as_ref()
			.and_then(|attributes| attributes.get("recursive"))
			.and_then(|value| value.as_bool())
			.unwrap_or(false)
	}

	fn escape_mermaid_label(label: &str) -> String {
		label.replace('"', "\\\"")
	}

	fn get_boundary_target<'a>(
		&self,
		boundary: &Card,
		view_cards: &'a HashMap<String, &'a Card>,
	) -> Option<&'a Card> {
		let links = boundary.links.as_ref()?;
		let mut target_id = links
			.iter()
			.find(|link| link.relationship == "contains")
			.map(|link| link.target.as_str());
		if target_id.is_none() {
			target_id = links.first().map(|link| link.target.as_str());
		}
		target_id.and_then(|id| view_cards.get(id).copied())
	}

	fn collect_recursive_cards<'a>(
		&self,
		start: &'a Card,
		view_cards: &'a HashMap<String, &'a Card>,
		visited: &mut HashSet<String>,
		collected: &mut Vec<&'a Card>,
	) {
		if !visited.insert(start.id.clone()) {
			return;
		}
		collected.push(start);
		if let Some(children) = self.adjacency.get(&start.id) {
			for child_id in children {
				if let Some(child) = view_cards.get(child_id) {
					self.collect_recursive_cards(child, view_cards, visited, collected);
				}
			}
		}
	}

	fn render_boundary_subgraph<'a>(
		&self,
		boundary: &'a Card,
		view_cards: &'a HashMap<String, &'a Card>,
		boundary_members: &mut HashSet<String>,
		rendered_boundaries: &mut HashSet<String>,
		indent_level: usize,
	) -> Result<String, ModelError> {
		if rendered_boundaries.contains(&boundary.id) {
			return Ok(String::new());
		}
		rendered_boundaries.insert(boundary.id.clone());
		boundary_members.insert(boundary.id.clone());

		let Some(target) = self.get_boundary_target(boundary, view_cards) else {
			return Ok(String::new());
		};

		let recursive = Self::boundary_is_recursive(boundary);
		let mut members: Vec<&Card> = Vec::new();
		if recursive {
			let mut visited: HashSet<String> = HashSet::new();
			self.collect_recursive_cards(target, view_cards, &mut visited, &mut members);
		} else {
			members.push(target);
		}

		let indent = "\t".repeat(indent_level);
		let inner_indent = "\t".repeat(indent_level + 1);
		let mut result = String::new();
		result.push_str(&format!(
			"{}subgraph {}[\"{}\"]\n",
			indent,
			boundary.id,
			Self::escape_mermaid_label(&boundary.name)
		));

		for member in members {
			if Self::is_boundary_card(member) {
				let nested = self.render_boundary_subgraph(
					member,
					view_cards,
					boundary_members,
					rendered_boundaries,
					indent_level + 1,
				)?;
				if !nested.is_empty() {
					result.push_str(&nested);
				}
				continue;
			}

			if boundary_members.contains(&member.id) {
				continue;
			}
			boundary_members.insert(member.id.clone());
			result.push_str(&format!(
				"{}{}{}",
				inner_indent,
				member.id,
				member.render_view_node()
			));
		}

		result.push_str(&format!("{}end\n", indent));
		Ok(result)
	}

	fn render_view_nodes(cards: &[&Card], boundary_members: &HashSet<String>) -> String {
		let mut nodes = String::new();
		for card in cards {
			if Self::is_boundary_card(card) || boundary_members.contains(&card.id) {
				continue;
			}
			nodes.push_str(&format!("\t{}{}\n", card.id, card.render_view_node()));
		}
		nodes
	}

	fn render_view_edges(&self, cards: &[&Card], view_cards: &HashMap<String, &Card>) -> String {
		let mut edges = String::new();
		for card in cards {
			if Self::is_boundary_card(card) {
				continue;
			}
			if let Some(links) = &card.links {
				for link in links {
					let Some(target) = view_cards.get(&link.target) else {
						continue;
					};
					if Self::is_boundary_card(target) {
						continue;
					}
					let dotted = Self::is_note_card(target);
					edges.push_str(&Self::render_link(card, target, &link.relationship, dotted));
				}
			}
		}
		edges
	}

	fn add_cards_by_type(&self, card_type: &str) -> Result<Vec<&Card>, ModelError> {
		let mut cards: Vec<&Card> = Vec::new();
		for card in self.get_cards_by_type(card_type) {
			cards.push(card);
			let special_cards = self.add_special_cards(card)?;
			for special_card in special_cards {
				cards.push(special_card);
			}
		}
		Ok(cards)
	}

	fn add_special_cards(&self, card: &Card) -> Result<Vec<&Card>, ModelError> {
		let mut cards: Vec<&Card> = Vec::new();
		if let Some(links) = &card.links {
			for link in links {
				if Self::is_special_child_id(&link.target) {
					let special_card = self
						.cards
						.get(&link.target)
						.ok_or(ModelError::CardNotFound(link.target.clone()))?;
					if Self::is_boundary_card(special_card) || Self::is_note_card(special_card) {
						cards.push(special_card);
					}
				}
			}
		};
		Ok(cards)
	}

	fn get_cards_by_type(&self, card_type: &str) -> Vec<&Card> {
		let mut cards_of_type: Vec<&Card> = Vec::new();
		for card in self.cards.values() {
			if card.card_type == card_type {
				cards_of_type.push(card);
			}
		}
		cards_of_type
	}

	fn write_cards_markdown(&self, args: &OutputArgs) -> Result<(), ModelError> {
		Ok(for card in self.cards.values() {
			let card_path = args
				.output_path
				.join(&self.mission_card.id)
				.join(&card.card_type);
			std::fs::create_dir_all(&card_path)?;
			let card_file_path = card_path.join(format!("{}.md", card.id));
			card.render(&card_file_path)?;
		})
	}

	fn build_card_index(&self) -> String {
		let mut cards: String = String::new();
		let mut current_card_type: String = String::new();
		for card in self.cards.values() {
			if card.card_type != current_card_type {
				current_card_type = card.card_type.clone();
				cards.push_str(format!("## {}\n\n", current_card_type).as_str());
			}
			match &card.card_subtype {
				Some(subtype) => {
					cards.push_str(
						format!(
							"- [{} ({}): {}]({}/{}/{}.md)\n",
							card.id,
							subtype.clone(),
							card.name,
							self.mission_card.id,
							card.card_type,
							card.id
						)
						.as_str(),
					);
				}
				None => {
					cards.push_str(
						format!(
							"- [{}: {}]({}/{}/{}.md)\n",
							card.id,
							card.name,
							self.mission_card.id,
							card::prefixes::get_prefix(&card.card_type)
								.unwrap_or("UNK".to_string()),
							card.id
						)
						.as_str(),
					);
				}
			}
		}
		cards
	}

	fn find_views(&self, args: &OutputArgs) -> String {
		let mut views: Vec<(String, String)> = Vec::new();
		let mut views_found: String = String::new();
		for view_name in view_definitions::get_definitions().keys() {
			views.push((
				view_name.to_string(),
				format!("{}.view.md", view_name.replace(" ", "_")),
			));
		}
		for (view_name, view_file) in views {
			let view_path = args
				.output_path
				.join(&self.mission_card.id)
				.join(&view_file);
			if view_path.exists() {
				views_found.push_str(
					format!(
						"- [{}]({}/{})\n",
						view_name, self.mission_card.id, view_file
					)
					.as_str(),
				);
			}
		}
		views_found
	}

	fn render_link(source: &Card, target: &Card, relationship: &str, dotted: bool) -> String {
		if dotted {
			format!("\t{} -. {} .-> {};\n", source.id, relationship, target.id,)
		} else {
			format!("\t{} -- {} --> {};\n", source.id, relationship, target.id)
		}
	}

	fn render_classes(cards: &Vec<&Card>) -> String {
		let mut classes: HashMap<String, Vec<String>> = HashMap::new();
		for card in cards {
			let class_name = format!("cls_{}", Self::normalize_class_name(&card.card_type));
			classes
				.entry(class_name)
				.or_insert_with(Vec::new)
				.push(card.id.clone());
		}

		let mut class_names: Vec<&String> = classes.keys().collect();
		class_names.sort();
		let mut result: String = String::new();
		for class_name in class_names {
			if let Some(members) = classes.get(class_name) {
				let mut sorted_members = members.clone();
				sorted_members.sort();
				result.push_str(
					format!("\tclass {} {};\n", sorted_members.join(","), class_name).as_str(),
				);
			}
		}
		result
	}

	fn normalize_class_name(card_type: &str) -> String {
		card_type.trim().to_lowercase().replace(' ', "_")
	}

	pub(crate) fn clear_output_dir(output_root: &Path) -> Result<(), ModelError> {
		let mut pending_paths: Vec<PathBuf> = Vec::new();
		pending_paths.push(output_root.to_path_buf());

		while let Some(path) = pending_paths.pop() {
			for entry in path.read_dir()? {
				let entry = entry?;
				let entry_path = entry.path();
				if entry_path.is_dir() {
					pending_paths.push(entry_path);
				} else if entry_path.is_file() {
					if let Some(file_name) = entry_path.file_name() {
						if file_name.to_str().unwrap_or("").ends_with(".md") {
							debug!("Removing markdown file {}", entry_path.display());
							std::fs::remove_file(entry_path)?;
						}
					}
				}
			}
		}

		Ok(())
	}

	fn load_models(
		path: &Path,
		card_validator: &Validator,
		mission_filter: Option<&str>,
	) -> Result<Vec<Self>, ModelError> {
		let regex_mission = Regex::new(r"^(MIS-\d{3})-([A-Za-z0-9_]+)\.json$")?;
		let mut models: Vec<Self> = Vec::new();
		for entry in path.read_dir()? {
			let entry = entry?;
			let entry_path = entry.path();
			if !entry_path.is_file() {
				continue;
			}
			let file_name = entry_path
				.file_name()
				.and_then(|name| name.to_str())
				.ok_or_else(|| ModelError::InvalidFileName(entry_path.display().to_string()))?;
			let captures = match regex_mission.captures(file_name) {
				Some(caps) => caps,
				None => continue,
			};
			let mission_id = captures.get(1).unwrap().as_str().to_string();
			if let Some(filter) = mission_filter {
				if filter != mission_id {
					continue;
				}
			}
			let file_sanitized = captures.get(2).unwrap().as_str().to_string();

			let raw = fs::read_to_string(&entry_path)?;
			let line_index = LineIndex::new(&raw);
			let mission_value: serde_json::Value = serde_json::from_str(&raw)?;
			let mission_card = Card::load(&entry_path)?;

			let mut schema_errors = Self::collect_schema_errors(
				&raw,
				&line_index,
				card_validator,
				&mission_value,
				&entry_path,
				&mission_id,
				"Mission",
				&mission_id,
			);
			Self::enforce_identifier_rule(
				&raw,
				&line_index,
				&entry_path,
				&mission_id,
				"Mission",
				&mission_id,
				"id",
				&mission_id,
				&mission_card.id,
				&mut schema_errors,
			);
			if mission_card.card_type != "Mission" {
				let message = format!(
					"Mission card type must be 'Mission' but found '{}'.",
					mission_card.card_type
				);
				schema_errors.push(format_error_message(
					&entry_path,
					&mission_id,
					"Mission",
					&mission_id,
					find_field_offset(&raw, "card_type").map(|offset| line_index.line_col(offset)),
					&message,
				));
			}
			let sanitized_name = Self::sanitize_name(&mission_card.name);
			if sanitized_name != file_sanitized {
				let message = format!(
					"Mission file name requires sanitized title '{}', found '{}'.",
					sanitized_name, file_sanitized
				);
				schema_errors.push(format_error_message(
					&entry_path,
					&mission_id,
					"Mission",
					&mission_id,
					find_field_offset(&raw, "name").map(|offset| line_index.line_col(offset)),
					&message,
				));
			}

			let model = Self {
				mission_card,
				cards: HashMap::new(),
				adjacency: HashMap::new(),
				card_schema_errors: schema_errors,
				card_invariant_errors: Vec::new(),
				compact_model: None,
			};
			models.push(model);
		}

		Ok(models)
	}

	fn collect_schema_errors(
		source: &str,
		line_index: &LineIndex,
		validator: &Validator,
		document: &serde_json::Value,
		path: &Path,
		mission_id: &str,
		card_type: &str,
		card_id: &str,
	) -> Vec<String> {
		let evaluation = validator.evaluate(document);
		let mut results: Vec<String> = Vec::new();
		for error in evaluation.iter_errors() {
			let pointer = error.instance_location.to_string();
			let pointer_display = format_pointer(&pointer);
			let message = format!(
				"JSON schema validation failed at {}: {}",
				pointer_display, error.error
			);
			let line_col = pointer_position(source, &pointer, line_index);
			results.push(format_error_message(
				path, mission_id, card_type, card_id, line_col, &message,
			));
		}

		results
	}

	fn enforce_identifier_rule(
		source: &str,
		line_index: &LineIndex,
		path: &Path,
		mission_id: &str,
		card_type: &str,
		card_id: &str,
		field_name: &str,
		expected: &str,
		actual: &str,
		errors: &mut Vec<String>,
	) {
		if expected == actual {
			return;
		}
		let offset = find_field_offset(source, field_name);
		let line_col = offset.map(|offset| line_index.line_col(offset));
		let message = format!(
			"{} must match '{}', found '{}'.",
			field_name, expected, actual
		);
		errors.push(format_error_message(
			path, mission_id, card_type, card_id, line_col, &message,
		));
	}

	fn validate_card_layout(
		source: &str,
		line_index: &LineIndex,
		card_path: &Path,
		mission_id: &str,
		directory_card_type: &str,
		file_name: &str,
		card: &Card,
		errors: &mut Vec<String>,
	) {
		let expected_sanitized_name = Self::sanitize_name(&card.name);
		let expected_file_name = format!("{}-{}.json", card.id, expected_sanitized_name);
		if file_name != expected_file_name {
			let message = format!(
				"Card file name must be '{}', found '{}'.",
				expected_file_name, file_name
			);
			errors.push(format_error_message(
				card_path,
				mission_id,
				directory_card_type,
				&card.id,
				find_field_offset(source, "name").map(|offset| line_index.line_col(offset)),
				&message,
			));
		}

		if card.card_type != directory_card_type {
			Self::enforce_identifier_rule(
				source,
				line_index,
				card_path,
				mission_id,
				&card.card_type,
				&card.id,
				"card_type",
				directory_card_type,
				&card.card_type,
				errors,
			);
		}

		if let Some(expected_prefix) = prefixes::get_prefix(&card.card_type) {
			if !card.id.starts_with(&expected_prefix) {
				let line_col =
					find_field_offset(source, "id").map(|offset| line_index.line_col(offset));
				let message = format!(
					"Card id '{}' must start with prefix '{}' for card type '{}'.",
					card.id, expected_prefix, card.card_type
				);
				errors.push(format_error_message(
					card_path,
					mission_id,
					&card.card_type,
					&card.id,
					line_col,
					&message,
				));
			}
		}
	}

	fn load_cards(
		model: &mut Model,
		mission_path: &Path,
		card_validator: &Validator,
	) -> Result<(), ModelError> {
		for type_entry in mission_path.read_dir()? {
			let type_entry = type_entry?;
			let type_path = type_entry.path();
			if !type_path.is_dir() {
				continue;
			}
			let Some(card_type_dir) = type_path.file_name().and_then(|name| name.to_str()) else {
				continue;
			};
			let card_type_dir = card_type_dir.to_string();

			for card_entry in type_path.read_dir()? {
				let card_entry = card_entry?;
				let card_path = card_entry.path();
				if !card_path.is_file()
					|| card_path.extension().and_then(|ext| ext.to_str()) != Some("json")
				{
					continue;
				}
				let file_name = card_path
					.file_name()
					.and_then(|name| name.to_str())
					.ok_or_else(|| ModelError::InvalidFileName(card_path.display().to_string()))?;
				let file_stem = card_path
					.file_stem()
					.and_then(|stem| stem.to_str())
					.unwrap_or("")
					.to_string();

				let raw = fs::read_to_string(&card_path)?;
				let line_index = LineIndex::new(&raw);
				let card_value: serde_json::Value = serde_json::from_str(&raw)?;
				let card_id_hint = card_value
					.get("id")
					.and_then(|value| value.as_str())
					.unwrap_or(file_stem.as_str())
					.to_string();
				let mut schema_errors = Self::collect_schema_errors(
					&raw,
					&line_index,
					card_validator,
					&card_value,
					&card_path,
					&model.mission_card.id,
					&card_type_dir,
					&card_id_hint,
				);
				if !schema_errors.is_empty() {
					model.card_schema_errors.append(&mut schema_errors);
				}

				let card = Card::from_value(&card_path, card_value)?;
				Self::validate_card_layout(
					&raw,
					&line_index,
					&card_path,
					&model.mission_card.id,
					&card_type_dir,
					file_name,
					&card,
					&mut model.card_schema_errors,
				);
				model.add_card(card);
			}
		}

		Ok(())
	}

	fn load_compact(
		path: &Path,
		model_id: &str,
		compact_validator: &Validator,
	) -> Result<Option<CompactModel>, ModelError> {
		let path = path.join(format!("AGENT-{}.json", model_id));
		if path.exists() {
			Ok(Some(CompactModel::load(&path, compact_validator)?))
		} else {
			Ok(None)
		}
	}

	fn add_card(&mut self, card: Card) {
		self.cards.insert(card.id.clone(), card.clone());
		if let Some(links) = &card.links {
			for link in links {
				self.adjacency
					.entry(card.id.clone())
					.or_insert_with(Vec::new)
					.push(link.target.clone());
			}
		}
	}
}

#[derive(Debug)]
struct LineIndex {
	offsets: Vec<usize>,
}

impl LineIndex {
	fn new(source: &str) -> Self {
		let mut offsets = vec![0];
		for (idx, ch) in source.char_indices() {
			if ch == '\n' {
				offsets.push(idx + 1);
			}
		}
		Self { offsets }
	}

	fn line_col(&self, offset: usize) -> (usize, usize) {
		let line_idx = match self.offsets.binary_search(&offset) {
			Ok(idx) => idx,
			Err(idx) => idx.saturating_sub(1),
		};
		let start = *self.offsets.get(line_idx).unwrap_or(&0);
		(line_idx + 1, offset.saturating_sub(start) + 1)
	}
}

fn format_error_message(
	path: &Path,
	mission_id: &str,
	card_type: &str,
	card_id: &str,
	line_col: Option<(usize, usize)>,
	message: &str,
) -> String {
	let location = line_col
		.map(|(line, col)| format!("@{}:{}", line, col))
		.unwrap_or_else(|| "@?:?".to_string());
	format!(
		"{}: [{}:{}:{}{}] {}",
		path.display(),
		mission_id,
		card_type,
		card_id,
		location,
		message
	)
}

fn pointer_position(source: &str, pointer: &str, line_index: &LineIndex) -> Option<(usize, usize)> {
	let tokens = pointer_tokens(pointer);
	json_pointer_offset_tokens(source, &tokens)
		.or_else(|| {
			tokens
				.last()
				.and_then(|token| find_field_offset(source, token))
		})
		.map(|offset| line_index.line_col(offset))
}

fn pointer_tokens(pointer: &str) -> Vec<String> {
	if pointer.is_empty() {
		return Vec::new();
	}
	pointer
		.trim_start_matches('/')
		.split('/')
		.filter(|segment| !segment.is_empty())
		.map(decode_pointer_token)
		.collect()
}

fn json_pointer_offset_tokens(source: &str, tokens: &[String]) -> Option<usize> {
	if tokens.is_empty() {
		return Some(0);
	}
	let mut search_start = 0usize;
	let mut target: Option<usize> = None;
	for token in tokens {
		if token.chars().all(|ch| ch.is_ascii_digit()) {
			continue;
		}
		let needle = format!("\"{}\"", token);
		let haystack = &source[search_start..];
		let relative = haystack.find(&needle)?;
		let absolute = search_start + relative;
		target = Some(absolute);
		search_start = absolute + needle.len();
	}
	target
}

fn decode_pointer_token(token: &str) -> String {
	token.replace("~1", "/").replace("~0", "~")
}

fn find_field_offset(source: &str, field: &str) -> Option<usize> {
	let needle = format!("\"{}\"", field);
	source.find(&needle)
}

fn format_pointer(pointer: &str) -> String {
	let tokens = pointer_tokens(pointer);
	if tokens.is_empty() {
		"$".to_string()
	} else {
		format!("$.{}", tokens.join("."))
	}
}

#[derive(Debug, Error)]
pub enum ModelError {
	#[error("An error occurred while loading a compact model: {0}")]
	CompactModelError(#[from] CompactModelError),
	#[error("Invalid file name: {0}")]
	InvalidFileName(String),
	#[error("No model found at specified path {0}")]
	PathNotFound(String),
	#[error("Schema file not found")]
	SchemaNotFound,
	#[error("Compact schema file not found")]
	CompactSchemaNotFound,
	#[error("Invalid schema: {0}")]
	InvalidSchema(String),
	#[error("Model root file not found")]
	ModelNotFound,
	#[error("Schema file too large, max {0} GB ({1} GB): {2}")]
	SchemaTooLarge(usize, usize, String),
	#[error("Io error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Regex error: {0}")]
	RegexError(#[from] regex::Error),
	#[error("JSON error: {0}")]
	JsonError(#[from] serde_json::Error),
	#[error("Card error: {0}")]
	CardError(#[from] CardError),
	#[error("Card not found: {0}")]
	CardNotFound(String),
	#[error("Cards not found")]
	CardsNotFound,
	#[error("model path not found: {0}")]
	ModelPathNotFound(String),
}
