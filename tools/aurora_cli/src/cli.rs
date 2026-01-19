mod bump_args;
mod output_args;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::cli::{bump_args::BumpArgs, output_args::OutputArgs};

#[derive(Parser, Debug)]
#[command(
	name = "aurora_cli",
	version,
	about = "Aurora model validation and rendering toolkit"
)]
pub struct Cli {
	#[arg(short, long = "input", default_value = "docs/design/aurora/")]
	pub input_path: PathBuf,
	#[arg(short, long="log", default_value_t = log::Level::Info)]
	pub log_level: log::Level,
	#[command(subcommand)]
	pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
	Validate,
	RenderCards(OutputArgs),
	RenderViews(OutputArgs),
	RenderAll(OutputArgs),
	Compact(OutputArgs),
	BumpPatch(BumpArgs),
	BumpMinor(BumpArgs),
	BumpMajor(BumpArgs),
}
