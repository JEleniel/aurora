use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct OutputArgs {
	#[arg(short, long = "output", default_value = "docs/design/")]
	pub output_path: PathBuf,
	#[arg(short, long = "clear", default_value_t = true)]
	pub clear: bool,
}
