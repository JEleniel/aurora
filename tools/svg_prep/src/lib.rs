mod cli;
mod svg;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

pub fn run() -> Result<()> {
	init_logging();
	info!("svg_prep starting");

	let args = cli::parse_args();
	info!("Arguments parsed successfully");

	svg::run(&args)?;
	info!("svg_prep completed successfully");

	Ok(())
}

fn init_logging() {
	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
	let _ = tracing_subscriber::fmt()
		.with_env_filter(filter)
		.with_target(false)
		.with_writer(std::io::stdout)
		.compact()
		.try_init();
}
