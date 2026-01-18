//! Library entry points for aurora_cli.

mod bump;
mod compact;
mod error;
mod json_utils;
mod loader;
mod model;
mod output;
mod render;
mod validator;

pub use bump::{Level as BumpLevel, bump_paths};
use clap::{Parser, Subcommand};
pub use compact::export_compact_model;
pub use error::AuroraCliError;
pub use loader::load_model;
pub use model::{Card, Model};
pub use output::OutputPaths;
pub use render::{CardRenderSummary, ViewRenderSummary, render_cards, render_views};
use std::path::PathBuf;
use std::result::Result;
pub use validator::{ValidationOutcome, validate_model, validation_report_markdown};

#[derive(Debug)]
enum CommandStatus {
	Success,
	ValidationFailed,
}

/// Run the CLI logic. Returns an exit code (0 success, 2 validation failed, 1 other error).
pub fn run() -> Result<i32, AuroraCliError> {
	const EXIT_VALIDATION_FAILED: i32 = 2;

	#[derive(Parser, Debug)]
	#[command(
		name = "aurora_cli",
		version,
		about = "Aurora model validation and rendering toolkit"
	)]
	struct Cli {
		#[arg(long = "model-path", default_value = "docs/design/aurora/")]
		model_path: PathBuf,
		#[arg(long = "output-root", default_value = "docs/design/")]
		output_root: PathBuf,
		#[command(subcommand)]
		command: Command,
	}

	#[derive(Subcommand, Debug)]
	enum Command {
		Validate,
		RenderCards,
		RenderViews,
		RenderAll,
		Compact,
		BumpPatch {
			/// Paths to card files or directories
			paths: Vec<PathBuf>,
			/// Editor identity to record in audit history
			#[arg(long = "editor")]
			editor: Option<String>,
		},
		BumpMinor {
			/// Paths to card files or directories
			paths: Vec<PathBuf>,
			/// Editor identity to record in audit history
			#[arg(long = "editor")]
			editor: Option<String>,
		},
		BumpMajor {
			/// Paths to card files or directories
			paths: Vec<PathBuf>,
			/// Editor identity to record in audit history
			#[arg(long = "editor")]
			editor: Option<String>,
		},
	}

	let cli = Cli::parse();
	let model = load_model(&cli.model_path)?;
	// Create mission-specific output folder under the provided output root
	let mission_root = cli.output_root.join(model.mission().id.clone());
	let outputs = OutputPaths::new(&mission_root)?;
	let status = match cli.command {
		Command::Validate => run_validate(&model, &outputs)?,
		Command::RenderCards => run_render_cards(&model, &outputs)?,
		Command::RenderViews => run_render_views(&model, &outputs)?,
		Command::RenderAll => run_all(&model, &outputs)?,
		Command::Compact => run_compact(&model, &outputs)?,
		Command::BumpPatch { paths, editor } => {
			let ps: Vec<PathBuf> = paths.into_iter().collect();
			bump_paths(&ps, BumpLevel::Patch, editor, true)?;
			println!("bump-patch completed for {} path(s)", ps.len());
			CommandStatus::Success
		}
		Command::BumpMinor { paths, editor } => {
			let ps: Vec<PathBuf> = paths.into_iter().collect();
			bump_paths(&ps, BumpLevel::Minor, editor, true)?;
			println!("bump-minor completed for {} path(s)", ps.len());
			CommandStatus::Success
		}
		Command::BumpMajor { paths, editor } => {
			let ps: Vec<PathBuf> = paths.into_iter().collect();
			bump_paths(&ps, BumpLevel::Major, editor, true)?;
			println!("bump-major completed for {} path(s)", ps.len());
			CommandStatus::Success
		}
	};

	match status {
		CommandStatus::Success => Ok(0),
		CommandStatus::ValidationFailed => Ok(EXIT_VALIDATION_FAILED),
	}
}

fn run_validate(model: &Model, outputs: &OutputPaths) -> Result<CommandStatus, AuroraCliError> {
	let outcome = validate_and_report(model, outputs)?;
	if outcome.is_success() {
		println!("Validation passed for {} card(s)", outcome.card_count);
		Ok(CommandStatus::Success)
	} else {
		println!(
			"Validation failed; see {}",
			outputs.validation_report().display()
		);
		Ok(CommandStatus::ValidationFailed)
	}
}

fn run_render_cards(model: &Model, outputs: &OutputPaths) -> Result<CommandStatus, AuroraCliError> {
	let outcome = validate_and_report(model, outputs)?;
	if !outcome.is_success() {
		println!(
			"Validation failed; see {}",
			outputs.validation_report().display()
		);
		return Ok(CommandStatus::ValidationFailed);
	}

	// Clean previous generated artifacts before rendering
	outputs.clean()?;
	let summary = render_cards(model, outputs)?;
	println!(
		"Rendered {} card file(s); index at {}",
		summary.cards_written,
		outputs.index_readme().display()
	);
	Ok(CommandStatus::Success)
}

fn run_render_views(model: &Model, outputs: &OutputPaths) -> Result<CommandStatus, AuroraCliError> {
	let outcome = validate_and_report(model, outputs)?;
	if !outcome.is_success() {
		println!(
			"Validation failed; see {}",
			outputs.validation_report().display()
		);
		return Ok(CommandStatus::ValidationFailed);
	}

	// Clean previous generated artifacts before rendering
	outputs.clean()?;
	let summary = render_views(model, outputs)?;
	println!(
		"Rendered {} view(s); skipped {} empty view(s)",
		summary.rendered, summary.skipped
	);
	Ok(CommandStatus::Success)
}

fn run_all(model: &Model, outputs: &OutputPaths) -> Result<CommandStatus, AuroraCliError> {
	let outcome = validate_and_report(model, outputs)?;
	if !outcome.is_success() {
		println!(
			"Validation failed; see {}",
			outputs.validation_report().display()
		);
		return Ok(CommandStatus::ValidationFailed);
	}
	// Clean previous generated artifacts before rendering
	outputs.clean()?;
	let view_summary = render_views(model, outputs)?;
	let card_summary = render_cards(model, outputs)?;
	println!(
		"Rendered {} card(s) and {} view(s); skipped {} view(s)",
		card_summary.cards_written, view_summary.rendered, view_summary.skipped
	);
	Ok(CommandStatus::Success)
}

fn run_compact(model: &Model, outputs: &OutputPaths) -> Result<CommandStatus, AuroraCliError> {
	let outcome = validate_and_report(model, outputs)?;
	if !outcome.is_success() {
		println!(
			"Validation failed; see {}",
			outputs.validation_report().display()
		);
		return Ok(CommandStatus::ValidationFailed);
	}
	export_compact_model(model, outputs)?;
	println!(
		"Wrote compact model to {}",
		outputs.compact_model_path().display()
	);
	Ok(CommandStatus::Success)
}

fn validate_and_report(
	model: &Model,
	outputs: &OutputPaths,
) -> Result<ValidationOutcome, AuroraCliError> {
	let outcome = validate_model(model)?;
	let report = validation_report_markdown(model, &outcome);
	outputs.write(outputs.validation_report(), &report)?;
	Ok(outcome)
}
