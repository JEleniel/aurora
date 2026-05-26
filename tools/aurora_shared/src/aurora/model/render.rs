use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{Card, Model, ModelError};

impl Model {
	pub fn write_markdown(&self, path: &Path) -> Result<(), ModelError> {
		let mission_slug = super::sanitize_filename(&self.root_card.name);
		let mut readme_path = path.to_path_buf();
		readme_path.push(format!("README-{}-{}.md", self.root_card.id, mission_slug));

		let mission_md_path = path.join(format!("{}-{}.md", self.root_card.id, mission_slug));

		let mut markdown_paths_by_id: HashMap<String, PathBuf> = HashMap::new();
		markdown_paths_by_id.insert(self.root_card.id.clone(), mission_md_path.clone());
		for card in &self.cards {
			let card_slug = super::sanitize_filename(&card.name);
			let card_md_path = path
				.join(self.root_card.id.as_str())
				.join(super::sanitize_card_type_folder(&card.card_type))
				.join(format!("{}-{}.md", card.id, card_slug));
			markdown_paths_by_id.insert(card.id.clone(), card_md_path);
		}

		let mut markdown = String::from(super::MODEL_MARKDOWN_TEMPLATE);

		let mission_link = format!(
			"**[Mission Card]({}-{}.md)**",
			self.root_card.id, mission_slug
		);

		markdown = markdown
			.replace("{{id}}", self.root_card.id.as_str())
			.replace("{{name}}", self.root_card.name.as_str())
			.replace("{{mission_link}}", &mission_link)
			.replace("{{description}}", self.root_card.description.as_str());

		let mut views = String::new();
		let view_path = path.join(self.root_card.id.as_str()).join("Views");
		if view_path.exists() {
			let mut svg_files = Vec::new();
			for entry in std::fs::read_dir(&view_path)? {
				let entry = entry?;
				let is_svg = entry.file_type()?.is_file()
					&& entry.path().extension().and_then(|s| s.to_str()) == Some("svg");
				if is_svg {
					svg_files.push(entry.file_name().to_string_lossy().into_owned());
				}
			}
			svg_files.sort();
			for file_name in svg_files {
				views.push_str(&format!(
					"\n![{}]({}/Views/{})\n",
					file_name, self.root_card.id, file_name
				));
			}
		}
		if views.is_empty() {
			views.push_str("_No views available._");
		}
		markdown = markdown.replace("{{views}}", &views);

		let mut index = String::new();
		let mut card_types: Vec<String> = self.cards.iter().map(|c| c.card_type.clone()).collect();
		card_types.sort();
		card_types.dedup();
		for card_type in card_types {
			let cards_of_type: Vec<&Card> = self
				.cards
				.iter()
				.filter(|c| c.card_type == card_type)
				.collect();
			if cards_of_type.is_empty() {
				continue;
			}

			index.push_str(format!("### {}\n\n", card_type).as_str());
			for card in cards_of_type {
				let card_type_folder = super::sanitize_card_type_folder(&card.card_type);
				let card_slug = super::sanitize_filename(&card.name);
				let card_link = format!(
					"- **[{} - {}]({}/{}/{}-{}.md)**: {}\n\n",
					card.id,
					card.name,
					self.root_card.id,
					card_type_folder,
					card.id,
					card_slug,
					card.description
				);
				index.push_str(card_link.as_str());
			}
		}
		markdown = markdown.replace("{{index}}", &index);
		while markdown.contains("\n\n\n") {
			markdown = markdown.replace("\n\n\n", "\n\n");
		}

		std::fs::write(&readme_path, markdown)?;

		self.root_card.write_markdown(
			&mission_md_path,
			Some(&markdown_paths_by_id),
			self.audit_log.entries_for_target(&self.root_card.id),
		)?;

		for card in &self.cards {
			let mut card_path = path.to_path_buf();
			card_path.push(self.root_card.id.as_str());
			card_path.push(super::sanitize_card_type_folder(&card.card_type));
			std::fs::create_dir_all(&card_path)?;
			let card_slug = super::sanitize_filename(&card.name);
			card_path.push(format!("{}-{}.md", card.id, card_slug));
			card.write_markdown(
				&card_path,
				Some(&markdown_paths_by_id),
				self.audit_log.entries_for_target(&card.id),
			)?;
		}

		Ok(())
	}
}
