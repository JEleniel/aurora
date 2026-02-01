//! Aurora standalone editor built with Dioxus.

mod app;
mod constants;
mod error;
mod graph;
mod logging;
mod model;
mod state;

use std::process::ExitCode;

use dioxus_desktop::Config;
use tracing::error;

use crate::error::EditorError;

fn main() -> ExitCode {
	if let Err(error) = run() {
		error!(error = %error, "aurora_editor terminated with an error");
		return ExitCode::from(1);
	}
	ExitCode::SUCCESS
}

fn run() -> Result<(), EditorError> {
	logging::init_logging()?;
	let config = Config::default();
	dioxus_desktop::launch::launch(app::App, Vec::new(), config);
	Ok(())
}
