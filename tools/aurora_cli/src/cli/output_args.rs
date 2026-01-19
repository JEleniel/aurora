use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct OutputArgs {
	#[arg(short, long = "output", default_value = "docs/design/")]
	pub output_path: PathBuf,
}
