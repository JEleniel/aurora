//! Library entry points for aurora_cli.
mod aurora;
mod cli;
mod logging;

use crate::{
	aurora::Aurora,
	cli::{Cli, Command},
};
use clap::Parser;
use log::debug;
use logging::Logging;
use std::path::{Path, PathBuf};

pub use aurora::AuroraError;

fn find_git_root(start: &Path) -> Option<PathBuf> {
	let mut current: Option<&Path> = Some(start);
	while let Some(dir) = current {
		if dir.join(".git").exists() {
			return Some(dir.to_path_buf());
		}
		current = dir.parent();
	}
	None
}

fn resolve_input_path(input_path: &Path, current_dir: &Path) -> Result<PathBuf, std::io::Error> {
	if input_path.is_absolute() {
		return input_path.canonicalize();
	}

	let from_cwd = current_dir.join(input_path);
	match from_cwd.canonicalize() {
		Ok(canonical) => Ok(canonical),
		Err(err) => {
			if let Some(git_root) = find_git_root(current_dir) {
				let from_repo_root = git_root.join(input_path);
				if let Ok(canonical) = from_repo_root.canonicalize() {
					return Ok(canonical);
				}
			}
			Err(err)
		}
	}
}

/// Run the CLI logic. Returns an exit code (0 success, 2 validation failed, 1 other error).
pub fn run() -> Result<String, AuroraError> {
	let mut cli = Cli::parse();

	Logging::init(cli.log_level)?;

	let current_dir = std::env::current_dir()?;
	cli.input_path = resolve_input_path(&cli.input_path, &current_dir)?;

	debug!("Using input path: {}", cli.input_path.display());
	let mut aurora = Aurora::load(&cli.input_path)?;

	let results = match cli.command {
		Command::Validate => aurora.validate()?,
		Command::RenderAurora(args) => aurora.render_models(&args)?,
		Command::RenderViews(args) => aurora.render_views(&args)?,
		Command::RenderAll(args) => aurora.render_all(&args)?,
		Command::Compact(args) => aurora.compact(&args)?,
		Command::BumpPatch(args) => aurora.bump_patch(&args)?,
		Command::BumpMinor(args) => aurora.bump_minor(&args)?,
		Command::BumpMajor(args) => aurora.bump_major(&args)?,
	};
	if !results.is_empty() {
		Ok(results.iter().map(|r| format!("{}\n", r)).collect())
	} else {
		Ok("\nNo output generated.\n".to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::resolve_input_path;
	use std::{
		fs,
		path::{Path, PathBuf},
		time::{SystemTime, UNIX_EPOCH},
	};

	fn unique_temp_dir(prefix: &str) -> PathBuf {
		let nanos = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_nanos();
		std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), nanos))
	}

	fn touch(path: &Path) {
		fs::write(path, "{}").expect("write temp file");
	}

	#[test]
	fn resolve_input_path_falls_back_to_git_root() {
		let root = unique_temp_dir("aurora_cli_resolve_path");
		let git_dir = root.join(".git");
		let nested = root.join("tools").join("aurora_cli");
		let model_home = root.join("docs").join("design").join("aurora");

		fs::create_dir_all(&git_dir).expect("create fake .git");
		fs::create_dir_all(&nested).expect("create nested dir");
		fs::create_dir_all(&model_home).expect("create model dir");
		touch(&model_home.join("Aurora.schema.json"));
		touch(&model_home.join("Aurora.compact.schema.json"));

		let resolved =
			resolve_input_path(Path::new("docs/design/aurora"), &nested).expect("resolve path");
		assert_eq!(resolved, model_home.canonicalize().expect("canonicalize"));

		fs::remove_dir_all(&root).expect("cleanup temp root");
	}
}
