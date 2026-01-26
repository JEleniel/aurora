use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use aurora_shared::{
	AuroraModel, Card, DiagnosticSeverity, ModelHome, RenderSummary, ValidationReport,
	discover_model_homes, load_model, render_all, render_markdown, render_views, validate_model,
	write_compact_model,
};
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

const DEFAULT_INPUT: &str = "docs/design/aurora";

#[derive(Debug, Parser)]
#[command(
	author,
	version,
	about = "Aurora model validation and rendering toolkit"
)]
struct Cli {
	/// Input path pointing to a mission card, model home, or ancestor directory.
	#[arg(short, long, value_name = "PATH", default_value = DEFAULT_INPUT)]
	input: PathBuf,

	/// Minimum log level to emit (trace|debug|info|warn|error).
	#[arg(short, long, value_name = "LEVEL", default_value = "info")]
	log: String,

	#[command(subcommand)]
	command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
	/// Validate one or more models and print diagnostics.
	Validate,

	/// Render individual card markdown files for each model.
	RenderAurora {
		#[arg(short, long, value_name = "DIR")]
		output: PathBuf,
	},

	/// Render tabular relationship views for each model.
	RenderViews {
		#[arg(short, long, value_name = "DIR")]
		output: PathBuf,
	},

	/// Run both markdown and relationship renders for each model.
	RenderAll {
		#[arg(short, long, value_name = "DIR")]
		output: PathBuf,
	},

	/// Generate (or refresh) the compact agent export.
	Compact {
		#[arg(short, long, value_name = "FILE")]
		output: Option<PathBuf>,
	},

	/// Increment the model audit trail patch version (not yet implemented).
	BumpPatch,

	/// Increment the model audit trail minor version (not yet implemented).
	BumpMinor,

	/// Increment the model audit trail major version (not yet implemented).
	BumpMajor,
}

fn main() -> Result<()> {
	let cli = Cli::parse();
	init_tracing(&cli.log)?;

	let homes = discover_model_homes(&cli.input)
		.with_context(|| format!("failed to discover models from {}", cli.input.display()))?;
	if homes.is_empty() {
		bail!("no Aurora models were found under {}", cli.input.display());
	}

	match cli.command {
		Commands::Validate => run_validate(&homes),
		Commands::RenderAurora { output } => run_render(&homes, &output, RenderMode::Markdown),
		Commands::RenderViews { output } => run_render(&homes, &output, RenderMode::Views),
		Commands::RenderAll { output } => run_render(&homes, &output, RenderMode::All),
		Commands::Compact { output } => run_compact(&homes, output),
		Commands::BumpPatch => unsupported_bump("patch"),
		Commands::BumpMinor => unsupported_bump("minor"),
		Commands::BumpMajor => unsupported_bump("major"),
	}
}

fn init_tracing(level: &str) -> Result<()> {
	let filter = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new(level))
		.with_context(|| format!("invalid log level '{level}'"))?;
	let builder = tracing_subscriber::fmt()
		.with_env_filter(filter)
		.with_writer(std::io::stderr)
		.with_target(false);
	let _ = builder.try_init();
	Ok(())
}

fn run_validate(homes: &[ModelHome]) -> Result<()> {
	let mut failures = false;
	for home in homes {
		info!(path = %home.root().display(), "Validating model home");
		let models = load_mission_models(home)?;
		for model in models {
			let label = model_label(&model);
			info!(path = %home.root().display(), mission = %label, "Validating mission model");
			let report = validate_model(&model);
			emit_report(home, &model, &report);
			if report.has_errors() {
				failures = true;
			}
		}
	}

	if failures {
		bail!("validation failed");
	}
	Ok(())
}

fn run_render(homes: &[ModelHome], base_output: &Path, mode: RenderMode) -> Result<()> {
	let mut failures = false;
	for home in homes {
		info!(path = %home.root().display(), "Rendering model home");
		let models = load_mission_models(home)?;
		for model in models {
			let label = model_label(&model);
			info!(path = %home.root().display(), mission = %label, "Rendering mission model");
			let report = validate_model(&model);
			emit_report(home, &model, &report);
			if report.has_errors() {
				failures = true;
				continue;
			}

			let mut model_dir = base_output.join(model_output_dir(&model));
			if let Some(mission_dir) = mission_output_dir(&model) {
				let candidate = base_output.join(&mission_dir);
				if candidate.exists() {
					model_dir = candidate;
				}
			}
			fs::create_dir_all(&model_dir).with_context(|| {
				format!(
					"failed to create render output directory {}",
					model_dir.display()
				)
			})?;

			let summary = mode
				.execute(&model, &model_dir)
				.with_context(|| format!("failed to render model at {}", home.root().display()))?;
			print_render_summary(&summary);
		}
	}

	if failures {
		bail!("render aborted due to validation errors");
	}
	Ok(())
}

fn run_compact(homes: &[ModelHome], output: Option<PathBuf>) -> Result<()> {
	let mut models_by_home = Vec::new();
	let mut model_count = 0usize;
	for home in homes {
		let models = load_mission_models(home)?;
		model_count = model_count.saturating_add(models.len());
		models_by_home.push((home.clone(), models));
	}

	if output.is_some() && model_count > 1 {
		bail!("--output can only be used when targeting a single mission model");
	}

	let mut failures = false;
	for (home, models) in models_by_home {
		info!(path = %home.root().display(), "Writing compact model");
		for model in models {
			let label = model_label(&model);
			info!(path = %home.root().display(), mission = %label, "Writing mission compact model");
			let report = validate_model(&model);
			emit_report(&home, &model, &report);
			if report.has_errors() {
				failures = true;
				continue;
			}

			let path = write_compact_model(&model, output.clone()).with_context(|| {
				format!(
					"failed to write compact export for {}",
					home.root().display()
				)
			})?;
			println!("Wrote compact model to {}", path.display());
		}
	}

	if failures {
		bail!("compact export aborted due to validation errors");
	}
	Ok(())
}

fn unsupported_bump(level: &str) -> Result<()> {
	bail!("Version bump commands are not implemented yet (requested {level} bump).");
}

fn emit_report(home: &ModelHome, model: &AuroraModel, report: &ValidationReport) {
	if report.diagnostics.is_empty() {
		return;
	}
	println!(
		"\nValidation report for {} :: {}",
		home.root().display(),
		model_label(model)
	);
	for diag in &report.diagnostics {
		println!(
			"  [{}] {}: {}",
			severity_label(diag.severity),
			diag.code,
			diag.message
		);
		if let Some(card_id) = &diag.card_id {
			println!("      card: {card_id}");
		}
		if let Some(path) = &diag.path {
			println!("      path: {path}");
		}
	}
}

fn severity_label(severity: DiagnosticSeverity) -> &'static str {
	match severity {
		DiagnosticSeverity::Error => "ERROR",
		DiagnosticSeverity::Warning => "WARN",
		DiagnosticSeverity::Info => "INFO",
	}
}

fn model_output_dir(model: &AuroraModel) -> String {
	if let Some(card) = mission_card(model) {
		return format!("{}-{}", card.id, sanitize(&card.name));
	}
	"model".to_string()
}

fn mission_output_dir(model: &AuroraModel) -> Option<String> {
	mission_card(model).map(|card| card.id.clone())
}

fn mission_card(model: &AuroraModel) -> Option<&Card> {
	model.iter_cards().find(|card| card.card_type == "Mission")
}

fn model_label(model: &AuroraModel) -> String {
	if let Some(card) = mission_card(model) {
		return format!("{} ({})", card.id, card.name);
	}
	"Unknown mission".to_string()
}

fn sanitize(value: &str) -> String {
	value
		.chars()
		.map(|ch| match ch {
			'A'..='Z' | 'a'..='z' | '0'..='9' | '-' => ch,
			_ => '_',
		})
		.collect()
}

fn print_render_summary(summary: &RenderSummary) {
	println!(
		"Rendered {} cards and {} views into {}",
		summary.cards_written,
		summary.views_written,
		summary.output_dir.display()
	);
}

fn load_mission_models(home: &ModelHome) -> Result<Vec<AuroraModel>> {
	let model = load_model(home)
		.with_context(|| format!("failed to load model at {}", home.root().display()))?;
	Ok(model.split_by_mission())
}

#[derive(Debug, Clone, Copy)]
enum RenderMode {
	Markdown,
	Views,
	All,
}

impl RenderMode {
	fn execute(self, model: &AuroraModel, output: &Path) -> aurora_shared::Result<RenderSummary> {
		match self {
			RenderMode::Markdown => render_markdown(model, output),
			RenderMode::Views => render_views(model, output),
			RenderMode::All => render_all(model, output),
		}
	}
}
