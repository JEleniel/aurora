mod cli;
mod session;

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Context;
pub use session::*;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

/// Run the Aurora editor scaffold.
pub fn run() -> Result<(), RuntimeError> {
	let cli = cli::parse_args();
	let log_path = default_log_path()?;
	init_tracing(&cli.log, &log_path)?;
	let startup = Instant::now();

	info!(model_home = %cli.model_home.display(), log_path = %log_path.display(), "aurora_editor starting");
	let runtime = build_runtime()?;
	runtime.block_on(async move {
		let session = EditorSession::open(&cli.model_home)?;
		info!(
			model_home = %session.model_home().display(),
			root_count = session.roots().len(),
			elapsed_ms = startup.elapsed().as_millis(),
			"aurora_editor interactive session ready"
		);
		Ok::<(), RuntimeError>(())
	})?;

	Ok(())
}

fn build_runtime() -> Result<tokio::runtime::Runtime, RuntimeError> {
	tokio::runtime::Builder::new_multi_thread()
		.thread_name("aurora-editor")
		.enable_all()
		.build()
		.context("failed to initialize aurora_editor runtime")
		.map_err(RuntimeError::Anyhow)
}

fn init_tracing(level: &str, log_path: &Path) -> Result<(), RuntimeError> {
	if let Some(parent) = log_path.parent() {
		std::fs::create_dir_all(parent)?;
	}

	let filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(level))?;
	let log_path = log_path.to_path_buf();
	let writer = BoxMakeWriter::new(move || match open_log_file(log_path.as_path()) {
		Ok(file) => Box::new(file) as Box<dyn std::io::Write + Send>,
		Err(_) => Box::new(std::io::sink()) as Box<dyn std::io::Write + Send>,
	});
	let _ = tracing_subscriber::fmt()
		.with_env_filter(filter)
		.with_target(false)
		.with_ansi(false)
		.with_writer(writer)
		.try_init();
	Ok(())
}

fn default_log_path() -> Result<PathBuf, RuntimeError> {
	let mut base = dirs::data_local_dir()
		.or_else(dirs::home_dir)
		.ok_or_else(|| {
			RuntimeError::Anyhow(anyhow::anyhow!("could not determine data directory"))
		})?;
	base.push("aurora_editor");
	base.push("logs");
	base.push("aurora_editor.log");
	Ok(base)
}

fn open_log_file(path: &Path) -> Result<File, std::io::Error> {
	OpenOptions::new().create(true).append(true).open(path)
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
	#[error("An anyhow error has occurred: {0}")]
	Anyhow(#[from] anyhow::Error),
	#[error("An editor session error has occurred: {0}")]
	EditorSession(#[from] EditorSessionError),
	#[error("An IO error has occurred: {0}")]
	Io(#[from] std::io::Error),
	#[error("An error occurred parsing the environment: {0}")]
	EnvParseError(#[from] tracing_subscriber::filter::ParseError),
}
