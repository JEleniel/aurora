//! Library entry points for aurora_cli.
mod aurora;
mod cli;
mod logging;

use crate::{
	aurora::Aurora,
	cli::{Cli, Command},
};
use clap::Parser;
use logging::Logging;

pub use aurora::AuroraError;

/// Run the CLI logic. Returns an exit code (0 success, 2 validation failed, 1 other error).
pub fn run() -> Result<String, AuroraError> {
	let cli = Cli::parse();

	Logging::init(cli.log_level)?;

	let mut aurora = Aurora::load(&cli.input_path)?;

	let results = match cli.command {
		Command::Validate => aurora.validate()?,
		Command::RenderAurora(args) => aurora.render_models(&args)?,
		Command::RenderViews(args) => aurora.render_cards(&args)?,
		Command::RenderAll(args) => aurora.render_all(&args)?,
		Command::Compact(args) => aurora.compact(&args)?,
		Command::BumpPatch(args) => aurora.bump_patch(&args)?,
		Command::BumpMinor(args) => aurora.bump_minor(&args)?,
		Command::BumpMajor(args) => aurora.bump_major(&args)?,
	};
	Ok(results.iter().map(|r| format!("{}\n", r)).collect())
}
