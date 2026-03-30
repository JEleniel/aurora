use std::ffi::OsStr;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use super::{Card, Model, ModelError};

#[derive(Debug)]
struct PreparedFileWrite {
	final_path: PathBuf,
	temp_path: PathBuf,
	contents: String,
}

#[derive(Debug)]
struct CommittedFileWrite {
	final_path: PathBuf,
	previous_contents: Option<Vec<u8>>,
}

impl Model {
	pub fn write(&self) -> Result<(), ModelError> {
		let ext = self
			.root_card
			.source_path
			.extension()
			.unwrap_or(OsStr::new("json"))
			.to_str()
			.ok_or(ModelError::InvalidFilename)?;

		let mission_path = self.model_home.join(format!(
			"{}-{}.{}",
			self.root_card.id,
			super::sanitize_filename(&self.root_card.name),
			ext
		));
		let prepared_root = self.root_card.prepare_write(&mission_path)?;

		let mut writes = vec![PreparedFileWrite::new(
			mission_path,
			prepared_root.serialized,
			0,
		)?];
		let mut prepared_cards: Vec<Card> = Vec::with_capacity(self.cards.len());
		prepared_cards.push(prepared_root.card.clone());

		for (index, card) in self.cards.iter().enumerate() {
			let mut card_path = self.mission_home.clone();
			card_path.push(super::sanitize_card_type_folder(&card.card_type));
			card_path.push(format!(
				"{}-{}.{}",
				card.id,
				super::sanitize_filename(&card.name),
				ext
			));

			let prepared = card.prepare_write(&card_path)?;
			prepared_cards.push(prepared.card.clone());
			writes.push(PreparedFileWrite::new(
				card_path,
				prepared.serialized,
				index + 1,
			)?);
		}

		let validation_model = Model {
			root_card: prepared_root.card,
			cards: prepared_cards.into_iter().skip(1).collect(),
			audit_log: self.audit_log.clone(),
			model_home: self.model_home.clone(),
			mission_home: self.mission_home.clone(),
		};
		let errors = validation_model.validate();
		if !errors.is_empty() {
			return Err(ModelError::ValidationErrors(errors));
		}

		write_transactionally(writes)?;
		Ok(())
	}

	pub fn write_markdown(&self, path: &Path) -> Result<(), ModelError> {
		let mission_slug = super::sanitize_filename(&self.root_card.name);
		let mut readme_path = path.to_path_buf();
		readme_path.push(format!("README-{}-{}.md", self.root_card.id, mission_slug));

		let mission_md_path = path.join(format!("{}-{}.md", self.root_card.id, mission_slug));

		let mut markdown_paths_by_id: std::collections::HashMap<String, PathBuf> =
			std::collections::HashMap::new();
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
			for entry in fs::read_dir(&view_path)? {
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
			fs::create_dir_all(&card_path)?;
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

impl PreparedFileWrite {
	fn new(final_path: PathBuf, contents: String, index: usize) -> Result<Self, ModelError> {
		let temp_path = temp_path_for(&final_path, index)?;
		Ok(Self {
			final_path,
			temp_path,
			contents,
		})
	}
}

fn temp_path_for(final_path: &Path, index: usize) -> Result<PathBuf, ModelError> {
	let parent = final_path.parent().ok_or(ModelError::InvalidFilename)?;
	let file_name = final_path
		.file_name()
		.and_then(|name| name.to_str())
		.ok_or(ModelError::InvalidFilename)?;
	Ok(parent.join(format!(".{file_name}.tmp-{}-{index}", std::process::id())))
}

pub(crate) fn write_single_file_transactionally(
	final_path: PathBuf,
	contents: String,
) -> Result<(), std::io::Error> {
	let prepared =
		PreparedFileWrite::new(final_path, contents, 0).map_err(model_error_to_io_error)?;
	write_transactionally(vec![prepared])
}

fn write_transactionally(writes: Vec<PreparedFileWrite>) -> Result<(), std::io::Error> {
	let mut staged_temp_paths: Vec<PathBuf> = Vec::with_capacity(writes.len());
	for write in &writes {
		if let Some(parent) = write.final_path.parent() {
			fs::create_dir_all(parent)?;
		}
		fs::write(&write.temp_path, &write.contents)?;
		staged_temp_paths.push(write.temp_path.clone());
	}

	let mut committed: Vec<CommittedFileWrite> = Vec::with_capacity(writes.len());
	for write in &writes {
		let previous_contents = if write.final_path.is_file() {
			Some(fs::read(&write.final_path)?)
		} else {
			None
		};

		if previous_contents.is_some() {
			fs::remove_file(&write.final_path)?;
		}

		if let Err(error) = fs::rename(&write.temp_path, &write.final_path) {
			if let Some(previous) = &previous_contents {
				let _ = fs::write(&write.final_path, previous);
			}
			cleanup_temp_files(&staged_temp_paths);
			rollback_writes(&committed);
			return Err(error);
		}

		committed.push(CommittedFileWrite {
			final_path: write.final_path.clone(),
			previous_contents,
		});
	}

	cleanup_temp_files(&staged_temp_paths);
	Ok(())
}

fn cleanup_temp_files(paths: &[PathBuf]) {
	for path in paths {
		if path.exists() {
			let _ = fs::remove_file(path);
		}
	}
}

fn rollback_writes(committed: &[CommittedFileWrite]) {
	for write in committed.iter().rev() {
		match &write.previous_contents {
			Some(previous_contents) => {
				let _ = fs::write(&write.final_path, previous_contents);
			}
			None => {
				let _ = fs::remove_file(&write.final_path);
			}
		}
	}
}

fn model_error_to_io_error(error: ModelError) -> std::io::Error {
	match error {
		ModelError::ReadError(error) => error,
		ModelError::InvalidFilename => Error::new(ErrorKind::InvalidInput, "invalid filename"),
		other => Error::other(other.to_string()),
	}
}

#[cfg(test)]
#[path = "model_write_support_tests.rs"]
mod model_write_support_tests;
