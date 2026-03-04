//! CLI argument parsing for aurora_cli.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::constants::{DEFAULT_INPUT, DEFAULT_OUTPUT};

/// CLI arguments for aurora_cli.
#[derive(Debug, Parser)]
#[command(
	author,
	version,
	about = "Aurora model validation and rendering toolkit"
)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,

	/// Input path pointing to a mission card, model home, or ancestor directory.
	/// Default: docs/design/aurora
	#[arg(
		short,
		long,
		value_name = "PATH",
		default_value = DEFAULT_INPUT,
		global = true
	)]
	pub input: PathBuf,

	/// Minimum log level to emit (trace|debug|info|warn|error).
	#[arg(
		short,
		long,
		value_name = "LEVEL",
		default_value = "info",
		global = true
	)]
	pub log: String,
}

/// CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
	/// Validate one or more models and print diagnostics.
	Validate,

	/// Upgrade model files in place to the latest supported schema.
	Upgrade,

	/// Render individual card markdown files for each model.
	RenderAurora {
		/// Output directory for card markdown (recommended: docs/design/).
		#[arg(short, long, value_name = "DIR", default_value = DEFAULT_OUTPUT)]
		output: PathBuf,
	},

	/// Render diagram views for each model.
	RenderViews {
		/// Output directory for rendered views (recommended: docs/design/).
		#[arg(short, long, value_name = "DIR", default_value = DEFAULT_OUTPUT)]
		output: PathBuf,
	},

	/// Run both markdown and diagram renders for each model.
	RenderAll {
		/// Output directory for rendered artifacts (recommended: docs/design/).
		#[arg(short, long, value_name = "DIR", default_value = DEFAULT_OUTPUT)]
		output: PathBuf,
	},

	/// Generate (or refresh) the compact agent export.
	Compact {
		/// Output directory (defaults to the model home).
		/// Writes `<OUTPUT>/<MISSION_ID>/Compact.json` for each mission.
		#[arg(short, long, value_name = "DIR", default_value=DEFAULT_INPUT)]
		output: PathBuf,
	},
}

/// Parse CLI arguments.
pub fn parse_args() -> Cli {
	Cli::parse()
}
