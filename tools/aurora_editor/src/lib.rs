#[cfg(not(test))]
mod app;
#[cfg(not(test))]
mod app_settings;
#[cfg(not(test))]
mod app_shell;
#[cfg(not(test))]
mod graph_view;
mod inspector_model;
#[cfg(not(test))]
mod inspector_sidebar;
mod navigation_model;
#[cfg(not(test))]
mod navigation_sidebar;
#[cfg(test)]
mod app {
	use std::path::PathBuf;
	use std::sync::Arc;

	use tokio::runtime::Runtime;

	use crate::EditorSession;
	use crate::config::{EditorConfig, EditorPaths};
	use crate::secrets::ResolvedProviderSecret;

	pub const SECRET_SERVICE_NAME: &str = "org.crystultima.aurora";

	#[derive(Clone, Debug, PartialEq, Eq)]
	pub struct SessionSummary {
		pub model_home_display: String,
		pub root_count: usize,
	}

	impl SessionSummary {
		pub fn from_session(session: &EditorSession) -> Self {
			Self {
				model_home_display: session.model_home().display().to_string(),
				root_count: session.roots().len(),
			}
		}
	}

	#[derive(Clone)]
	pub struct EditorAppBootstrap {
		pub model_home: PathBuf,
		pub paths: EditorPaths,
		pub config: EditorConfig,
		pub provider_secrets: Vec<ResolvedProviderSecret>,
		pub session: Option<Arc<EditorSession>>,
		pub runtime: Arc<Runtime>,
		pub session_summary: Option<SessionSummary>,
		pub secret_service_name: String,
		pub require_first_run_wizard: bool,
	}

	impl PartialEq for EditorAppBootstrap {
		fn eq(&self, other: &Self) -> bool {
			self.model_home == other.model_home
				&& self.paths == other.paths
				&& self.config == other.config
				&& self.provider_secrets == other.provider_secrets
				&& self.session_summary == other.session_summary
				&& self.secret_service_name == other.secret_service_name
				&& self.require_first_run_wizard == other.require_first_run_wizard
		}
	}

	pub fn launch_editor_app(_bootstrap: EditorAppBootstrap) -> ! {
		let _ = (&_bootstrap.session, &_bootstrap.runtime);
		panic!("desktop app launch is not available in unit tests")
	}
}
mod cli;
pub mod config;
mod first_run;
mod redaction;
pub mod secrets;
mod session;
mod settings;
mod shell;
mod theme;

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use app::{EditorAppBootstrap, SECRET_SERVICE_NAME, SessionSummary, launch_editor_app};
use config::{EditorConfig, EditorPaths};
use secrets::{KeyringSecretStore, load_provider_secrets};
pub use session::*;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

/// Run the Aurora editor scaffold.
pub fn run() -> Result<(), RuntimeError> {
	let cli = cli::parse_args();
	let paths = EditorPaths::discover()?;
	init_tracing(&cli.log, &paths.log_path)?;
	let loaded_config = EditorConfig::load(&paths)?;
	let runtime = Arc::new(build_runtime()?);
	log_startup(&cli.model_home, &paths, loaded_config.exists);
	if loaded_config.exists {
		return launch_configured_editor(cli.model_home, paths, loaded_config.config, runtime);
	}
	launch_first_run_wizard(cli.model_home, paths, loaded_config.config, runtime)
}

fn log_startup(model_home: &Path, paths: &EditorPaths, config_exists: bool) {
	info!(
		model_home = %model_home.display(),
		log_path = %paths.log_path.display(),
		config_path = %paths.config_path.display(),
		config_exists,
		"aurora_editor starting"
	);
}

fn launch_configured_editor(
	model_home: PathBuf,
	paths: EditorPaths,
	config: EditorConfig,
	runtime: Arc<tokio::runtime::Runtime>,
) -> Result<(), RuntimeError> {
	let startup = Instant::now();
	let provider_secrets =
		load_provider_secrets(&config, &KeyringSecretStore::new(SECRET_SERVICE_NAME));
	register_provider_secrets(&provider_secrets);
	log_provider_warnings(&provider_secrets);
	let session = runtime.block_on(async { EditorSession::open(&model_home) })?;
	let session_summary = SessionSummary::from_session(&session);
	info!(
		model_home = %session.model_home().display(),
		root_count = session.roots().len(),
		autosave = config.autosave,
		theme = config.theme_preference.as_str(),
		base_font_size_px = config.base_font_size_px,
		elapsed_ms = startup.elapsed().as_millis(),
		"aurora_editor interactive session ready"
	);
	launch_editor_app(EditorAppBootstrap {
		model_home,
		paths,
		config,
		provider_secrets,
		session: Some(Arc::new(session)),
		runtime,
		session_summary: Some(session_summary),
		secret_service_name: SECRET_SERVICE_NAME.to_string(),
		require_first_run_wizard: false,
	});
}

fn launch_first_run_wizard(
	model_home: PathBuf,
	paths: EditorPaths,
	config: EditorConfig,
	runtime: Arc<tokio::runtime::Runtime>,
) -> ! {
	info!(
		model_home = %model_home.display(),
		"aurora_editor first-run configuration required before opening the model home"
	);
	launch_editor_app(EditorAppBootstrap {
		model_home,
		paths,
		config,
		provider_secrets: Vec::new(),
		session: None,
		runtime,
		session_summary: None,
		secret_service_name: SECRET_SERVICE_NAME.to_string(),
		require_first_run_wizard: true,
	});
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
		Ok(file) => {
			Box::new(redaction::RedactingWriter::new(file)) as Box<dyn std::io::Write + Send>
		}
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
fn open_log_file(path: &Path) -> Result<File, std::io::Error> {
	OpenOptions::new().create(true).append(true).open(path)
}

fn register_provider_secrets(provider_secrets: &[secrets::ResolvedProviderSecret]) {
	for provider_secret in provider_secrets {
		if let Some(secret) = provider_secret.api_key.as_deref() {
			redaction::register_secret(secret);
		}
	}
}

fn log_provider_warnings(provider_secrets: &[secrets::ResolvedProviderSecret]) {
	for provider_secret in provider_secrets {
		for warning in &provider_secret.warnings {
			tracing::warn!(provider = provider_secret.provider.as_str(), "{warning}");
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
	#[error("An anyhow error has occurred: {0}")]
	Anyhow(#[from] anyhow::Error),
	#[error("An editor config error has occurred: {0}")]
	EditorConfig(#[from] config::EditorConfigError),
	#[error("An editor session error has occurred: {0}")]
	EditorSession(#[from] EditorSessionError),
	#[error("An IO error has occurred: {0}")]
	Io(#[from] std::io::Error),
	#[error("An error occurred parsing the environment: {0}")]
	EnvParseError(#[from] tracing_subscriber::filter::ParseError),
}
