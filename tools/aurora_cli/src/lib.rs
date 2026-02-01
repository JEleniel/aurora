mod cli;
pub mod constants;

use crate::cli::Commands;
use aurora_shared::Aurora;
use std::path::PathBuf;
use thiserror::Error;
use tracing::info;
use tracing_subscriber::EnvFilter;

pub fn run() -> Result<(), RuntimeError> {
	let cli = cli::parse_args();

	init_tracing(&cli.log)?;

	let aurora = Aurora::try_load(&cli.input)?;

	match cli.command {
		Commands::Validate => validate(&aurora),
		Commands::RenderAurora { output } => render_markdown(&aurora, &output),
		Commands::RenderViews { output } => render_views(&aurora, &output),
		Commands::RenderAll { output } => render_all(&aurora, &output),
		Commands::Compact { output } => run_compact(&aurora, &output),
	}?;

	Ok(())
}

fn validate(aurora: &Aurora) -> Result<(), RuntimeError> {
	info!(
		"Validating {} model(s) from {}",
		aurora.models.len(),
		aurora.model_home.display()
	);

	let errors = aurora.validate();
	if errors.len() == 0 {
		info!("No validation errors found.");
	}
	for error in errors {
		info!("{}", error);
	}

	let warnings = aurora.check_registry();
	if warnings.len() == 0 {
		info!("No warnings found.");
	}
	for warning in warnings {
		info!("{}", warning);
	}

	Ok(())
}

fn render_markdown(aurora: &Aurora, output_dir: &PathBuf) -> Result<(), RuntimeError> {
	info!(
		"Rendering markdown for {} model(s) into {}",
		aurora.models.len(),
		output_dir.display()
	);
	aurora.write_markdown(&output_dir.to_path_buf())?;

	Ok(())
}

fn render_views(aurora: &Aurora, output_dir: &PathBuf) -> Result<(), RuntimeError> {
	info!(
		"Rendering views for {} model(s) into {}",
		aurora.models.len(),
		output_dir.display()
	);
	aurora_shared::render(aurora, &output_dir.to_path_buf())?;

	Ok(())
}

fn render_all(aurora: &Aurora, output_dir: &PathBuf) -> Result<(), RuntimeError> {
	info!(
		"Rendering all outputs for {} model(s) into {}",
		aurora.models.len(),
		output_dir.display()
	);

	// Note: The views have to be rendered first so the index picks them up
	render_views(aurora, output_dir)?;
	render_markdown(aurora, output_dir)?;

	Ok(())
}

fn run_compact(aurora: &Aurora, output: &PathBuf) -> Result<(), RuntimeError> {
	info!(
		"Writing compact exports for {} model(s)",
		aurora.models.len()
	);
	aurora.write_compact(output)?;

	Ok(())
}

fn init_tracing(level: &str) -> Result<(), RuntimeError> {
	let filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(level))?;
	let builder = tracing_subscriber::fmt()
		.with_env_filter(filter)
		.with_writer(std::io::stderr)
		.with_target(false);
	let _ = builder.try_init();
	Ok(())
}

#[derive(Debug, Error)]
pub enum RuntimeError {
	#[error("An anyhow error has occurred: {0}")]
	Anyhow(#[from] anyhow::Error),
	#[error("An Aurora error has occurred: {0}")]
	Aurora(#[from] aurora_shared::AuroraError),
	#[error("An error occurred parsing the environment: {0}")]
	EnvParseError(#[from] tracing_subscriber::filter::ParseError),
	#[error("A Rendering error has occurred: {0}")]
	Rendering(#[from] aurora_shared::render_error::RenderError),
}
