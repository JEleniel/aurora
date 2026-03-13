//! Persistent editor configuration and OS-standard paths.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const APP_CONFIG_DIR: &str = "aurora";
const CONFIG_FILE_NAME: &str = "editor-config.json";
const LOG_DIR_NAME: &str = "logs";
const LOG_FILE_NAME: &str = "aurora_editor.log";
const CURRENT_EDITOR_CONFIG_VERSION: u32 = 1;

/// Resolved filesystem paths used by the editor runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPaths {
	pub config_path: PathBuf,
	pub log_path: PathBuf,
}

impl EditorPaths {
	/// Discover config and log locations from standard user directories.
	pub fn discover() -> Result<Self, EditorConfigError> {
		let config_root = dirs::config_dir()
			.or_else(dirs::home_dir)
			.ok_or(EditorConfigError::ConfigDirectoryUnavailable)?;
		let data_root = dirs::data_local_dir()
			.or_else(dirs::home_dir)
			.ok_or(EditorConfigError::DataDirectoryUnavailable)?;
		Ok(Self {
			config_path: config_root.join(APP_CONFIG_DIR).join(CONFIG_FILE_NAME),
			log_path: data_root
				.join("aurora_editor")
				.join(LOG_DIR_NAME)
				.join(LOG_FILE_NAME),
		})
	}

	#[cfg(test)]
	pub(crate) fn for_test(root: &Path) -> Self {
		Self {
			config_path: root.join(CONFIG_FILE_NAME),
			log_path: root.join(LOG_FILE_NAME),
		}
	}
}

/// Loaded config alongside whether it already existed on disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedEditorConfig {
	pub config: EditorConfig,
	pub exists: bool,
}

/// User-editable editor configuration.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EditorConfig {
	#[serde(default = "default_version")]
	pub version: u32,
	#[serde(default = "default_autosave")]
	pub autosave: bool,
	#[serde(default)]
	pub theme_preference: ThemePreference,
	#[serde(default = "default_base_font_size_px")]
	pub base_font_size_px: u16,
	#[serde(default)]
	pub editor_identity: EditorIdentity,
	#[serde(default)]
	pub providers: ProviderSettings,
}

impl EditorConfig {
	/// Load configuration if it exists; otherwise use defaults without writing a file.
	pub fn load(paths: &EditorPaths) -> Result<LoadedEditorConfig, EditorConfigError> {
		if !paths.config_path.exists() {
			return Ok(LoadedEditorConfig {
				config: Self::default(),
				exists: false,
			});
		}

		let contents = fs::read_to_string(&paths.config_path)?;
		let config = Self::from_json(&contents)?;
		Ok(LoadedEditorConfig {
			config,
			exists: true,
		})
	}

	/// Persist the current configuration as pretty JSON.
	pub fn save(&self, paths: &EditorPaths) -> Result<(), EditorConfigError> {
		self.validate()?;
		create_parent_directory(&paths.config_path)?;
		let json = serde_json::to_string_pretty(self)?;
		fs::write(&paths.config_path, format!("{json}\n"))?;
		Ok(())
	}

	fn from_json(contents: &str) -> Result<Self, EditorConfigError> {
		let config = serde_json::from_str::<Self>(contents)?;
		config.validate()?;
		Ok(config)
	}

	pub fn validate(&self) -> Result<(), EditorConfigError> {
		validate_version(self.version)?;
		validate_font_size(self.base_font_size_px)?;
		validate_identity(self.editor_identity.name.as_str())?;
		validate_identity_email(self.editor_identity.email.as_deref())?;
		validate_provider_endpoint("ollama", self.providers.ollama.endpoint.as_str())?;
		validate_provider_endpoint("openai", self.providers.openai.endpoint.as_str())?;
		validate_provider_endpoint(
			"github_models",
			self.providers.github_models.endpoint.as_str(),
		)?;
		Ok(())
	}
}

impl Default for EditorConfig {
	fn default() -> Self {
		Self {
			version: CURRENT_EDITOR_CONFIG_VERSION,
			autosave: true,
			theme_preference: ThemePreference::System,
			base_font_size_px: 16,
			editor_identity: EditorIdentity::default(),
			providers: ProviderSettings::default(),
		}
	}
}

/// Theme preference used by the editor chrome.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
	#[default]
	System,
	Light,
	Dark,
}

impl ThemePreference {
	/// Stable string form for logging and persistence diagnostics.
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::System => "system",
			Self::Light => "light",
			Self::Dark => "dark",
		}
	}
}

impl FromStr for ThemePreference {
	type Err = ParseThemePreferenceError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			"system" => Ok(Self::System),
			"light" => Ok(Self::Light),
			"dark" => Ok(Self::Dark),
			_ => Err(ParseThemePreferenceError),
		}
	}
}

/// Theme preference parse failure for UI/persistence inputs.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
#[error("invalid theme preference")]
pub struct ParseThemePreferenceError;

/// Audit-log attribution for local edits.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EditorIdentity {
	pub name: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub email: Option<String>,
}

impl Default for EditorIdentity {
	fn default() -> Self {
		Self {
			name: default_editor_name(),
			email: None,
		}
	}
}

/// Endpoint and environment fallback settings for agent providers.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProviderConfig {
	pub endpoint: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub api_key_env_var: Option<String>,
}

impl ProviderConfig {
	fn new(endpoint: &str, api_key_env_var: Option<&str>) -> Self {
		Self {
			endpoint: endpoint.to_string(),
			api_key_env_var: api_key_env_var.map(ToOwned::to_owned),
		}
	}
}

/// Provider settings for all supported agent backends.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProviderSettings {
	pub ollama: ProviderConfig,
	pub openai: ProviderConfig,
	pub github_models: ProviderConfig,
}

impl Default for ProviderSettings {
	fn default() -> Self {
		Self {
			ollama: ProviderConfig::new("http://127.0.0.1:11434", Some("OLLAMA_API_KEY")),
			openai: ProviderConfig::new("https://api.openai.com/v1", Some("OPENAI_API_KEY")),
			github_models: ProviderConfig::new(
				"https://models.inference.ai.azure.com",
				Some("GITHUB_TOKEN"),
			),
		}
	}
}

/// Errors raised while loading or saving editor configuration.
#[derive(Debug, Error)]
pub enum EditorConfigError {
	#[error("could not determine the user config directory")]
	ConfigDirectoryUnavailable,
	#[error("could not determine the user data directory")]
	DataDirectoryUnavailable,
	#[error("invalid editor configuration: {0}")]
	Invalid(String),
	#[error("editor configuration IO failed: {0}")]
	Io(#[from] std::io::Error),
	#[error("editor configuration JSON failed: {0}")]
	Json(#[from] serde_json::Error),
}

fn create_parent_directory(path: &Path) -> Result<(), EditorConfigError> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	Ok(())
}

fn default_version() -> u32 {
	CURRENT_EDITOR_CONFIG_VERSION
}

fn default_autosave() -> bool {
	true
}

fn default_base_font_size_px() -> u16 {
	16
}

fn default_editor_name() -> String {
	std::env::var("USER")
		.or_else(|_| std::env::var("USERNAME"))
		.unwrap_or_else(|_| "Aurora Editor".to_string())
}

fn validate_version(version: u32) -> Result<(), EditorConfigError> {
	if version == 0 {
		return Err(EditorConfigError::Invalid(
			"config version must be a positive integer".to_string(),
		));
	}
	Ok(())
}

fn validate_font_size(font_size: u16) -> Result<(), EditorConfigError> {
	if font_size == 0 {
		return Err(EditorConfigError::Invalid(
			"base_font_size_px must be greater than zero".to_string(),
		));
	}
	Ok(())
}

fn validate_identity(name: &str) -> Result<(), EditorConfigError> {
	if name.trim().is_empty() {
		return Err(EditorConfigError::Invalid(
			"editor_identity.name must not be blank".to_string(),
		));
	}
	Ok(())
}

fn validate_identity_email(email: Option<&str>) -> Result<(), EditorConfigError> {
	let Some(email) = email else {
		return Ok(());
	};
	if email.trim().is_empty() {
		return Err(EditorConfigError::Invalid(
			"editor_identity.email must not be blank when provided".to_string(),
		));
	}
	Ok(())
}

fn validate_provider_endpoint(provider: &str, endpoint: &str) -> Result<(), EditorConfigError> {
	if endpoint.trim().is_empty() {
		return Err(EditorConfigError::Invalid(format!(
			"providers.{provider}.endpoint must not be blank"
		)));
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::{EditorConfig, EditorConfigError, EditorPaths, ThemePreference};

	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn load_uses_defaults_when_config_is_missing() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let paths = EditorPaths::for_test(temp.path());

		let loaded = EditorConfig::load(&paths)?;

		assert!(!loaded.exists);
		assert_eq!(loaded.config.version, 1);
		assert!(loaded.config.autosave);
		assert_eq!(loaded.config.theme_preference, ThemePreference::System);
		Ok(())
	}

	#[test]
	fn save_then_load_round_trips() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let paths = EditorPaths::for_test(temp.path());
		let config = EditorConfig {
			autosave: false,
			base_font_size_px: 20,
			editor_identity: super::EditorIdentity {
				name: "Test Editor".to_string(),
				email: None,
			},
			..EditorConfig::default()
		};

		config.save(&paths)?;
		let loaded = EditorConfig::load(&paths)?;

		assert!(loaded.exists);
		assert_eq!(loaded.config, config);
		Ok(())
	}

	#[test]
	fn blank_provider_endpoint_is_rejected() {
		let config = EditorConfig {
			providers: super::ProviderSettings {
				openai: super::ProviderConfig::new("", Some("OPENAI_API_KEY")),
				..super::ProviderSettings::default()
			},
			..EditorConfig::default()
		};

		let error = config.save(&EditorPaths::for_test(std::path::Path::new(".")));
		assert!(matches!(error, Err(EditorConfigError::Invalid(_))));
	}
}
