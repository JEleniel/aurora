use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use aurora_shared::{
	AuroraModel, Card, DiagnosticSeverity, ModelHome, RenderSummary, ValidationReport,
	discover_model_homes, load_model, render_all, render_all_with_instructions, render_markdown,
	render_views, render_views_with_instructions, validate_model_with_instructions,
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
	/// Default: docs/design/aurora
	#[arg(short, long, value_name = "PATH", default_value = DEFAULT_INPUT)]
	input: PathBuf,

	/// Override the instructions directory used for validation and rendering.
	/// Expected to contain details/Card_Definitions.md, details/Matrix_View.md,
	/// details/Relationships_Matrix.md, and details/View_Definitions.md.
	#[arg(long, value_name = "DIR")]
	instructions_root: Option<PathBuf>,

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
		/// Output directory for card markdown (recommended: docs/design/).
		#[arg(short, long, value_name = "DIR")]
		output: PathBuf,
	},

	/// Render tabular relationship views for each model.
	RenderViews {
		/// Output directory for rendered views (recommended: docs/design/).
		#[arg(short, long, value_name = "DIR")]
		output: PathBuf,
	},

	/// Run both markdown and relationship renders for each model.
	RenderAll {
		/// Output directory for rendered artifacts (recommended: docs/design/).
		#[arg(short, long, value_name = "DIR")]
		output: PathBuf,
	},

	/// Generate (or refresh) the compact agent export.
	Compact {
		/// Output file path (defaults to <model home>/AGENT-<MISSION_ID>.jsjson).
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
	let instructions_root = normalize_instructions_root(&cli.instructions_root)?;

	let homes = discover_model_homes(&cli.input)
		.with_context(|| format!("failed to discover models from {}", cli.input.display()))?;
	if homes.is_empty() {
		bail!("no Aurora models were found under {}", cli.input.display());
	}

	match cli.command {
		Commands::Validate => run_validate(&homes, instructions_root.as_deref()),
		Commands::RenderAurora { output } => run_render(
			&homes,
			&output,
			RenderMode::Markdown,
			instructions_root.as_deref(),
		),
		Commands::RenderViews { output } => run_render(
			&homes,
			&output,
			RenderMode::Views,
			instructions_root.as_deref(),
		),
		Commands::RenderAll { output } => run_render(
			&homes,
			&output,
			RenderMode::All,
			instructions_root.as_deref(),
		),
		Commands::Compact { output } => run_compact(&homes, output, instructions_root.as_deref()),
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

fn run_validate(homes: &[ModelHome], instructions_root: Option<&Path>) -> Result<()> {
	let mut failures = false;
	let mut totals = DiagnosticTotals::default();
	let mut models_checked = 0usize;
	for home in homes {
		info!(path = %home.root().display(), "Validating model home");
		let instructions_hint = instructions_root_hint(home, instructions_root);
		log_instructions_root(home, instructions_hint.as_deref());
		let models = load_mission_models(home)?;
		for model in models {
			models_checked = models_checked.saturating_add(1);
			let label = model_label(&model);
			info!(path = %home.root().display(), mission = %label, "Validating mission model");
			let report = validate_model_with_instructions(&model, instructions_root);
			totals.record(&report);
			emit_report(home, &model, &report, instructions_hint.as_deref());
			if report.has_errors() {
				failures = true;
			}
		}
	}

	if failures {
		println!(
			"Validation completed with errors: {} mission model(s), {} error(s), {} warning(s), {} info message(s).",
			models_checked, totals.errors, totals.warnings, totals.infos
		);
		bail!("validation failed");
	}
	println!(
		"Validation succeeded for {} mission model(s) with {} warning(s) and {} info message(s).",
		models_checked, totals.warnings, totals.infos
	);
	Ok(())
}

fn run_render(
	homes: &[ModelHome],
	base_output: &Path,
	mode: RenderMode,
	instructions_root: Option<&Path>,
) -> Result<()> {
	let mut failures = false;
	let mut totals = DiagnosticTotals::default();
	let mut models_rendered = 0usize;
	let mut models_total = 0usize;
	for home in homes {
		info!(path = %home.root().display(), "Rendering model home");
		let instructions_hint = instructions_root_hint(home, instructions_root);
		log_instructions_root(home, instructions_hint.as_deref());
		let models = load_mission_models(home)?;
		for model in models {
			models_total = models_total.saturating_add(1);
			let label = model_label(&model);
			info!(path = %home.root().display(), mission = %label, "Rendering mission model");
			let report = validate_model_with_instructions(&model, instructions_root);
			totals.record(&report);
			emit_report(home, &model, &report, instructions_hint.as_deref());
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
				.execute(&model, &model_dir, instructions_root)
				.with_context(|| format!("failed to render model at {}", home.root().display()))?;
			print_render_summary(&summary);
			models_rendered = models_rendered.saturating_add(1);
		}
	}

	if failures {
		println!(
			"Render completed with errors: {}/{} mission model(s) rendered, {} error(s), {} warning(s), {} info message(s).",
			models_rendered, models_total, totals.errors, totals.warnings, totals.infos
		);
		bail!("render aborted due to validation errors");
	}
	println!(
		"Render completed: {}/{} mission model(s) rendered with {} warning(s) and {} info message(s).",
		models_rendered, models_total, totals.warnings, totals.infos
	);
	Ok(())
}

fn run_compact(
	homes: &[ModelHome],
	output: Option<PathBuf>,
	instructions_root: Option<&Path>,
) -> Result<()> {
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
	let mut totals = DiagnosticTotals::default();
	let mut models_written = 0usize;
	for (home, models) in models_by_home {
		info!(path = %home.root().display(), "Writing compact model");
		let instructions_hint = instructions_root_hint(&home, instructions_root);
		log_instructions_root(&home, instructions_hint.as_deref());
		for model in models {
			let label = model_label(&model);
			info!(path = %home.root().display(), mission = %label, "Writing mission compact model");
			let report = validate_model_with_instructions(&model, instructions_root);
			totals.record(&report);
			emit_report(&home, &model, &report, instructions_hint.as_deref());
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
			models_written = models_written.saturating_add(1);
		}
	}

	if failures {
		println!(
			"Compact export completed with errors: {}/{} mission model(s) written, {} error(s), {} warning(s), {} info message(s).",
			models_written, model_count, totals.errors, totals.warnings, totals.infos
		);
		bail!("compact export aborted due to validation errors");
	}
	println!(
		"Compact export completed: {}/{} mission model(s) written with {} warning(s) and {} info message(s).",
		models_written, model_count, totals.warnings, totals.infos
	);
	Ok(())
}

fn unsupported_bump(level: &str) -> Result<()> {
	bail!("Version bump commands are not implemented yet (requested {level} bump).");
}

fn emit_report(
	home: &ModelHome,
	model: &AuroraModel,
	report: &ValidationReport,
	instructions_root: Option<&Path>,
) {
	if report.diagnostics.is_empty() {
		return;
	}
	println!(
		"\nValidation report for {} :: {}",
		home.root().display(),
		model_label(model)
	);
	if let Some(path) = instructions_root {
		println!("      instructions: {}", path.display());
	} else {
		println!("      instructions: <not found>");
	}
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

#[derive(Debug, Default, Clone, Copy)]
struct DiagnosticTotals {
	errors: usize,
	warnings: usize,
	infos: usize,
}

impl DiagnosticTotals {
	fn record(&mut self, report: &ValidationReport) {
		for diag in &report.diagnostics {
			match diag.severity {
				DiagnosticSeverity::Error => self.errors = self.errors.saturating_add(1),
				DiagnosticSeverity::Warning => self.warnings = self.warnings.saturating_add(1),
				DiagnosticSeverity::Info => self.infos = self.infos.saturating_add(1),
			}
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

fn normalize_instructions_root(root: &Option<PathBuf>) -> Result<Option<PathBuf>> {
	let Some(path) = root else {
		return Ok(None);
	};
	if !path.is_dir() {
		bail!("Instructions root {} is not a directory", path.display());
	}
	let canonical = fs::canonicalize(path)
		.with_context(|| format!("failed to resolve instructions root {}", path.display()))?;
	Ok(Some(canonical))
}

fn instructions_root_hint(home: &ModelHome, override_root: Option<&Path>) -> Option<PathBuf> {
	if let Some(path) = override_root {
		return Some(path.to_path_buf());
	}
	if let Ok(value) = env::var("AURORA_INSTRUCTIONS_ROOT") {
		let trimmed = value.trim();
		if !trimmed.is_empty() {
			let path = PathBuf::from(trimmed);
			if path.is_dir() {
				return Some(path);
			}
		}
	}
	let mut current = Some(home.root().to_path_buf());
	while let Some(dir) = current {
		let candidate = dir.join(".github/instructions");
		if candidate.is_dir() {
			return Some(candidate);
		}
		current = dir.parent().map(|parent| parent.to_path_buf());
	}
	None
}

fn log_instructions_root(home: &ModelHome, instructions_root: Option<&Path>) {
	match instructions_root {
		Some(path) => {
			info!(
				path = %home.root().display(),
				instructions = %path.display(),
				"Using instructions registry (card, matrix, relationships, views)"
			);
		}
		None => {
			info!(
				path = %home.root().display(),
				"No .github/instructions found for matrix relationship validation"
			);
		}
	}
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
	fn execute(
		self,
		model: &AuroraModel,
		output: &Path,
		instructions_root: Option<&Path>,
	) -> aurora_shared::Result<RenderSummary> {
		match self {
			RenderMode::Markdown => render_markdown(model, output),
			RenderMode::Views => match instructions_root {
				Some(root) => render_views_with_instructions(model, output, root),
				None => render_views(model, output),
			},
			RenderMode::All => match instructions_root {
				Some(root) => render_all_with_instructions(model, output, root),
				None => render_all(model, output),
			},
		}
	}
}
