//! CLI argument parsing for aurora_editor.

use std::path::PathBuf;

use clap::Parser;

/// CLI arguments for aurora_editor.
#[derive(Debug, Parser)]
#[command(author, version, about = "Aurora standalone desktop editor")]
pub struct Cli {
	/// Model home path or ancestor containing an `aurora/` directory.
	#[arg(short = 'm', long, value_name = "PATH", default_value = ".")]
	pub model_home: PathBuf,

	/// Minimum log level to emit (trace|debug|info|warn|error).
	#[arg(short, long, value_name = "LEVEL", default_value = "info")]
	pub log: String,
}

/// Parse CLI arguments.
pub fn parse_args() -> Cli {
	Cli::parse()
}
