//! CLI argument parsing for svg_prep.

use std::path::PathBuf;

use clap::Parser;

const DEFAULT_ICONS_IN: &str = "assets/icons";
const DEFAULT_ICONS_OUT: &str = "assets/references/Icons.svg";
const DEFAULT_SHAPES: &str = "assets/references/Shapes.svg";
const DEFAULT_TEMPLATE: &str = ".github/agents/aurora/reference/SVGTemplate.svg";

/// CLI arguments for svg_prep.
#[derive(Debug, Parser)]
#[command(
	author,
	version,
	about = "Prepare reference Icons.svg and merged SVGTemplate defs"
)]
pub struct Cli {
	/// Input directory containing icon SVG files.
	#[arg(long, value_name = "DIR", default_value = DEFAULT_ICONS_IN)]
	pub icons_in: PathBuf,

	/// Output path for generated Icons.svg.
	#[arg(long, value_name = "FILE", default_value = DEFAULT_ICONS_OUT)]
	pub icons_out: PathBuf,

	/// Source Shapes.svg path used for template defs merge.
	#[arg(long, value_name = "FILE", default_value = DEFAULT_SHAPES)]
	pub shapes: PathBuf,

	/// Target SVGTemplate.svg path whose defs are replaced.
	#[arg(long, value_name = "FILE", default_value = DEFAULT_TEMPLATE)]
	pub template: PathBuf,
}

/// Parse CLI arguments.
pub fn parse_args() -> Cli {
	Cli::parse()
}
