//! Output path management and deterministic filesystem writes.

use crate::error::AuroraCliError;
use pathdiff::diff_paths;
use percent_encoding::{AsciiSet, CONTROLS, percent_encode};
use std::fs;
use std::path::{Component, Path, PathBuf};

const LINK_SET: &AsciiSet = &CONTROLS
	.add(b' ')
	.add(b'(')
	.add(b')')
	.add(b'[')
	.add(b']')
	.add(b'{')
	.add(b'}')
	.add(b'`')
	.add(b'"')
	.add(b'\'');

/// Derived filesystem paths for generated artifacts.
#[derive(Debug, Clone)]
pub struct OutputPaths {
	root: PathBuf,
	cards_dir: PathBuf,
	views_dir: PathBuf,
	index_readme: PathBuf,
	validation_report: PathBuf,
	compact_model: PathBuf,
}

impl OutputPaths {
	pub fn new(root: &Path) -> Result<Self, AuroraCliError> {
		if !root.exists() {
			fs::create_dir_all(root).map_err(|source| AuroraCliError::Io {
				path: root.to_path_buf(),
				source,
			})?;
		}
		let canonical = canonicalize(root)?;
		let cards_dir = canonical.join("cards");
		let views_dir = canonical.join("views");
		fs::create_dir_all(&cards_dir).map_err(|source| AuroraCliError::Io {
			path: cards_dir.clone(),
			source,
		})?;
		fs::create_dir_all(&views_dir).map_err(|source| AuroraCliError::Io {
			path: views_dir.clone(),
			source,
		})?;
		Ok(Self {
			root: canonical.clone(),
			cards_dir,
			views_dir,
			index_readme: canonical.join("README.md"),
			validation_report: canonical.join("validation-report.md"),
			compact_model: canonical.join("model-compact.json"),
		})
	}

	pub fn root(&self) -> &Path {
		&self.root
	}

	pub fn cards_dir(&self) -> &Path {
		&self.cards_dir
	}

	pub fn views_dir(&self) -> &Path {
		&self.views_dir
	}

	pub fn index_readme(&self) -> &Path {
		&self.index_readme
	}

	pub fn validation_report(&self) -> &Path {
		&self.validation_report
	}

	pub fn compact_model_path(&self) -> &Path {
		&self.compact_model
	}

	/// Remove existing generated artifacts (cards and views) and recreate directories.
	pub fn clean(&self) -> Result<(), AuroraCliError> {
		if self.cards_dir.exists() {
			fs::remove_dir_all(&self.cards_dir).map_err(|source| AuroraCliError::Io {
				path: self.cards_dir.clone(),
				source,
			})?;
		}
		if self.views_dir.exists() {
			fs::remove_dir_all(&self.views_dir).map_err(|source| AuroraCliError::Io {
				path: self.views_dir.clone(),
				source,
			})?;
		}
		// remove top-level readme and validation report if present
		if self.index_readme.exists() {
			fs::remove_file(&self.index_readme).map_err(|source| AuroraCliError::Io {
				path: self.index_readme.clone(),
				source,
			})?;
		}
		if self.validation_report.exists() {
			fs::remove_file(&self.validation_report).map_err(|source| AuroraCliError::Io {
				path: self.validation_report.clone(),
				source,
			})?;
		}
		if self.compact_model.exists() {
			fs::remove_file(&self.compact_model).map_err(|source| AuroraCliError::Io {
				path: self.compact_model.clone(),
				source,
			})?;
		}
		fs::create_dir_all(&self.cards_dir).map_err(|source| AuroraCliError::Io {
			path: self.cards_dir.clone(),
			source,
		})?;
		fs::create_dir_all(&self.views_dir).map_err(|source| AuroraCliError::Io {
			path: self.views_dir.clone(),
			source,
		})?;
		Ok(())
	}

	pub fn write(&self, target: &Path, contents: &str) -> Result<(), AuroraCliError> {
		ensure_within_root(&self.root, target)?;
		if let Some(parent) = target.parent() {
			fs::create_dir_all(parent).map_err(|source| AuroraCliError::Io {
				path: parent.to_path_buf(),
				source,
			})?;
		}
		let temp_path = target.with_extension("tmp");
		fs::write(&temp_path, contents).map_err(|source| AuroraCliError::Io {
			path: temp_path.clone(),
			source,
		})?;
		fs::rename(&temp_path, target).map_err(|source| AuroraCliError::Io {
			path: target.to_path_buf(),
			source,
		})?;
		Ok(())
	}
}

pub fn relative_markdown_link(from: &Path, to: &Path) -> Result<String, AuroraCliError> {
	let base = from.parent().unwrap_or(from);
	let relative = diff_paths(to, base).ok_or_else(|| AuroraCliError::RelativePath {
		from: base.to_path_buf(),
		to: to.to_path_buf(),
	})?;
	let mut parts = Vec::new();
	for component in relative.components() {
		match component {
			Component::CurDir => continue,
			Component::ParentDir => parts.push(String::from("..")),
			Component::Normal(seg) => parts.push(encode_segment(&seg.to_string_lossy())),
			Component::RootDir | Component::Prefix(_) => {
				return Err(AuroraCliError::OutputEscapesRoot {
					root: base.to_path_buf(),
					attempted: to.to_path_buf(),
				});
			}
		}
	}
	let result = if parts.is_empty() {
		String::from(".")
	} else {
		parts.join("/")
	};
	Ok(result)
}

pub fn ensure_within_root(root: &Path, target: &Path) -> Result<(), AuroraCliError> {
	let diff = diff_paths(target, root).ok_or_else(|| AuroraCliError::RelativePath {
		from: root.to_path_buf(),
		to: target.to_path_buf(),
	})?;
	for component in diff.components() {
		if matches!(component, Component::ParentDir) {
			return Err(AuroraCliError::OutputEscapesRoot {
				root: root.to_path_buf(),
				attempted: target.to_path_buf(),
			});
		}
	}
	Ok(())
}

fn encode_segment(segment: &str) -> String {
	percent_encode(segment.as_bytes(), LINK_SET).to_string()
}

fn canonicalize(path: &Path) -> Result<PathBuf, AuroraCliError> {
	std::fs::canonicalize(path).map_err(|source| AuroraCliError::Canonicalize {
		path: path.to_path_buf(),
		source,
	})
}
