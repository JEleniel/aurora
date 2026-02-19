//! CLI argument parsing for svg_prep.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

const DEFAULT_BUILD_ICONS_IN: &str = "assets/optimized/icons";
const DEFAULT_BUILD_ICONS_PROOF: &str = "assets/proofs/Icons.svg";
const DEFAULT_BUILD_SHAPES_IN: &str = "assets/optimized/shapes";
const DEFAULT_BUILD_SHAPES_PROOF: &str = "assets/proofs/Shapes.svg";
const DEFAULT_BUILD_TEMPLATE: &str = "assets/masters/SVGTemplate.svg";
const DEFAULT_BUILD_TEMPLATE_PROOF: &str = "assets/proofs/SVGTemplate.svg";
const DEFAULT_BUILD_TEMPLATE_SVGZ: &str = "assets/templates/SVGTemplate.svgz";

const DEFAULT_BUILD_MASTERS_ICONS_IN: &str = "assets/masters/icons";
const DEFAULT_BUILD_MASTERS_SHAPES_IN: &str = "assets/masters/shapes";

const DEFAULT_OPTIMIZE_ICONS_OUT: &str = "assets/optimized/icons";
const DEFAULT_OPTIMIZE_ICONS_PROOF: &str = "assets/proofs/Icons.svg";
const DEFAULT_OPTIMIZE_SHAPES_OUT: &str = "assets/optimized/shapes";
const DEFAULT_OPTIMIZE_SHAPES_PROOF: &str = "assets/proofs/Shapes.svg";

/// CLI entrypoint for svg_prep.
#[derive(Debug, Parser)]
#[command(
	author,
	version,
	about = "Prepare Aurora SVG reference assets and template defs"
)]
pub struct Cli {
	#[command(flatten)]
	pub build: BuildArgs,

	#[command(subcommand)]
	pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
	/// Optimize a set of source icons into Aurora's normalized icon format.
	OptimizeIcons(OptimizeIconsArgs),
	/// Optimize a set of source shapes into Aurora's normalized shape format.
	OptimizeShapes(OptimizeShapesArgs),
}

#[derive(Debug, Args)]
pub struct BuildArgs {
	/// Input directory containing master (unoptimized) icon SVG files.
	///
	/// If this directory exists, the default build will optimize icons from here into `--icons-in`.
	#[arg(long, value_name = "DIR", default_value = DEFAULT_BUILD_MASTERS_ICONS_IN)]
	pub masters_icons_in: PathBuf,

	/// Input directory containing master (unoptimized) shape SVG files.
	///
	/// If this directory exists, the default build will optimize shapes from here into `--shapes`.
	#[arg(long, value_name = "DIR", default_value = DEFAULT_BUILD_MASTERS_SHAPES_IN)]
	pub masters_shapes_in: PathBuf,

	/// Input directory containing icon SVG files.
	#[arg(long, value_name = "DIR", default_value = DEFAULT_BUILD_ICONS_IN)]
	pub icons_in: PathBuf,

	/// Output path for generated icon proof sheet (Icons.svg).
	#[arg(long, value_name = "FILE", default_value = DEFAULT_BUILD_ICONS_PROOF)]
	pub icons_proof: PathBuf,

	/// Source shapes input. If a directory, reads all *.svg files within.
	#[arg(long, value_name = "PATH", default_value = DEFAULT_BUILD_SHAPES_IN)]
	pub shapes: PathBuf,

	/// Output path for generated shape proof sheet (Shapes.svg).
	#[arg(long, value_name = "FILE", default_value = DEFAULT_BUILD_SHAPES_PROOF)]
	pub shapes_proof: PathBuf,

	/// Input SVGTemplate.svg path used as the base template.
	///
	/// Note: When this points inside `assets/masters/`, the default build will not
	/// overwrite the input file; it will write the merged template into
	/// `assets/templates/` unless `--template-out` is specified.
	#[arg(long, value_name = "FILE", default_value = DEFAULT_BUILD_TEMPLATE)]
	pub template: PathBuf,

	/// Output path for the merged SVG template (SVGTemplate.svg).
	///
	/// Default: derived from `--template`.
	/// - If `--template` is in `assets/masters/`, defaults to `assets/templates/SVGTemplate.svg`.
	/// - Otherwise, defaults to writing in place (same as `--template`).
	#[arg(long, value_name = "FILE")]
	pub template_out: Option<PathBuf>,

	/// Output path for a copy of the generated SVG template for review (SVGTemplate.svg).
	#[arg(long, value_name = "FILE", default_value = DEFAULT_BUILD_TEMPLATE_PROOF)]
	pub template_proof: PathBuf,

	/// Optional output path for the gzipped template (SVGTemplate.svgz).
	#[arg(long, value_name = "FILE", default_value = DEFAULT_BUILD_TEMPLATE_SVGZ)]
	pub template_svgz: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct OptimizeIconsArgs {
	/// Input directory containing source (unoptimized) icon SVG files.
	#[arg(long, value_name = "DIR", default_value = DEFAULT_BUILD_MASTERS_ICONS_IN)]
	pub input: PathBuf,

	/// Output directory to write optimized icon SVG files.
	#[arg(long, value_name = "DIR", default_value = DEFAULT_OPTIMIZE_ICONS_OUT)]
	pub output: PathBuf,

	/// Output path for generated icon proof sheet (Icons.svg).
	#[arg(long, value_name = "FILE", default_value = DEFAULT_OPTIMIZE_ICONS_PROOF)]
	pub proof: PathBuf,
}

#[derive(Debug, Args)]
pub struct OptimizeShapesArgs {
	/// Input directory containing source (unoptimized) shape SVG files.
	#[arg(long, value_name = "DIR", default_value = DEFAULT_BUILD_MASTERS_SHAPES_IN)]
	pub input: PathBuf,

	/// Output directory to write optimized shape SVG files.
	#[arg(long, value_name = "DIR", default_value = DEFAULT_OPTIMIZE_SHAPES_OUT)]
	pub output: PathBuf,

	/// Output path for generated shapes proof sheet (Shapes.svg).
	#[arg(long, value_name = "FILE", default_value = DEFAULT_OPTIMIZE_SHAPES_PROOF)]
	pub proof: PathBuf,
}

/// Parse CLI arguments.
pub fn parse_args() -> Cli {
	Cli::parse()
}
