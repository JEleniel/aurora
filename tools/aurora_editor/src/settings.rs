//! Editable settings state and persistence helpers for the editor settings UI.

use crate::config::{
	EditorConfig, EditorConfigError, EditorIdentity, EditorPaths, ProviderConfig, ProviderSettings,
	ThemePreference,
};
use crate::secrets::{
	AgentProvider, ResolvedProviderSecret, SecretSource, SecretStore, SecretStoreError,
};

/// Editable draft state for a single provider section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderDraft {
	pub endpoint: String,
	pub api_key: String,
	pub api_key_env_var: Option<String>,
	pub source: SecretSource,
	pub warnings: Vec<String>,
}

impl ProviderDraft {
	fn from_parts(config: &ProviderConfig, secret: Option<&ResolvedProviderSecret>) -> Self {
		let source = secret
			.map(|secret| secret.source)
			.unwrap_or(SecretSource::Unavailable);
		let warnings = secret
			.map(|secret| secret.warnings.clone())
			.unwrap_or_default();
		let api_key = secret
			.and_then(|secret| secret.api_key.clone())
			.unwrap_or_default();
		Self {
			endpoint: config.endpoint.clone(),
			api_key,
			api_key_env_var: config.api_key_env_var.clone(),
			source,
			warnings,
		}
	}

	fn mark_saved(&mut self) {
		if self.api_key.trim().is_empty() {
			self.source = SecretSource::Unavailable;
		} else {
			self.source = SecretSource::Keychain;
		}
		self.warnings.clear();
	}

	fn to_provider_config(&self) -> ProviderConfig {
		ProviderConfig {
			endpoint: self.endpoint.trim().to_string(),
			api_key_env_var: self.api_key_env_var.clone(),
		}
	}
}

/// Editable settings draft shown in the UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsDraft {
	pub version: u32,
	pub autosave: bool,
	pub theme_preference: ThemePreference,
	pub base_font_size_px: String,
	pub editor_name: String,
	pub editor_email: String,
	pub ollama: ProviderDraft,
	pub openai: ProviderDraft,
	pub github_models: ProviderDraft,
}

impl SettingsDraft {
	/// Build an editable draft from the persisted config plus resolved secrets.
	pub fn from_loaded(config: &EditorConfig, secrets: &[ResolvedProviderSecret]) -> Self {
		Self {
			version: config.version,
			autosave: config.autosave,
			theme_preference: config.theme_preference.clone(),
			base_font_size_px: config.base_font_size_px.to_string(),
			editor_name: config.editor_identity.name.clone(),
			editor_email: config.editor_identity.email.clone().unwrap_or_default(),
			ollama: ProviderDraft::from_parts(
				&config.providers.ollama,
				find_secret(secrets, AgentProvider::Ollama),
			),
			openai: ProviderDraft::from_parts(
				&config.providers.openai,
				find_secret(secrets, AgentProvider::OpenAi),
			),
			github_models: ProviderDraft::from_parts(
				&config.providers.github_models,
				find_secret(secrets, AgentProvider::GitHubModels),
			),
		}
	}

	/// Save the current draft to the config file and the secure key store.
	pub fn save(
		&self,
		paths: &EditorPaths,
		secret_store: &dyn SecretStore,
	) -> Result<(), SettingsSaveError> {
		let config = self.to_config()?;
		config.save(paths)?;
		for provider in AgentProvider::all() {
			let api_key = self.provider(provider).api_key.trim();
			if api_key.is_empty() {
				secret_store.delete_api_key(provider)?;
			} else {
				secret_store.set_api_key(provider, api_key)?;
			}
		}
		Ok(())
	}

	/// Mark the current draft as successfully persisted.
	pub fn mark_saved(&mut self) {
		self.provider_mut(AgentProvider::Ollama).mark_saved();
		self.provider_mut(AgentProvider::OpenAi).mark_saved();
		self.provider_mut(AgentProvider::GitHubModels).mark_saved();
	}

	/// Get a provider draft by provider identifier.
	pub fn provider(&self, provider: AgentProvider) -> &ProviderDraft {
		match provider {
			AgentProvider::Ollama => &self.ollama,
			AgentProvider::OpenAi => &self.openai,
			AgentProvider::GitHubModels => &self.github_models,
		}
	}

	/// Get a mutable provider draft by provider identifier.
	pub fn provider_mut(&mut self, provider: AgentProvider) -> &mut ProviderDraft {
		match provider {
			AgentProvider::Ollama => &mut self.ollama,
			AgentProvider::OpenAi => &mut self.openai,
			AgentProvider::GitHubModels => &mut self.github_models,
		}
	}

	fn to_config(&self) -> Result<EditorConfig, SettingsSaveError> {
		let font_size = self
			.base_font_size_px
			.trim()
			.parse::<u16>()
			.map_err(|_| SettingsSaveError::InvalidFontSize(self.base_font_size_px.clone()))?;
		let config = EditorConfig {
			version: self.version,
			autosave: self.autosave,
			theme_preference: self.theme_preference.clone(),
			base_font_size_px: font_size,
			editor_identity: EditorIdentity {
				name: self.editor_name.trim().to_string(),
				email: normalize_optional_text(self.editor_email.as_str()),
			},
			providers: ProviderSettings {
				ollama: self.ollama.to_provider_config(),
				openai: self.openai.to_provider_config(),
				github_models: self.github_models.to_provider_config(),
			},
		};
		config.validate()?;
		Ok(config)
	}
}

/// Settings save failures surfaced to the UI.
#[derive(Debug, thiserror::Error)]
pub enum SettingsSaveError {
	#[error("base font size must be a whole number, got `{0}`")]
	InvalidFontSize(String),
	#[error("{0}")]
	EditorConfig(#[from] EditorConfigError),
	#[error("{0}")]
	SecretStore(#[from] SecretStoreError),
}

fn find_secret(
	secrets: &[ResolvedProviderSecret],
	provider: AgentProvider,
) -> Option<&ResolvedProviderSecret> {
	secrets.iter().find(|secret| secret.provider == provider)
}

fn normalize_optional_text(value: &str) -> Option<String> {
	let trimmed = value.trim();
	if trimmed.is_empty() {
		None
	} else {
		Some(trimmed.to_string())
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use std::sync::Mutex;

	use super::{SettingsDraft, SettingsSaveError};
	use crate::config::{EditorConfig, EditorPaths};
	use crate::secrets::{
		AgentProvider, ResolvedProviderSecret, SecretSource, SecretStore, SecretStoreError,
	};

	type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	#[derive(Default)]
	struct FakeSecretStore {
		entries: Mutex<BTreeMap<&'static str, Option<String>>>,
	}

	impl SecretStore for FakeSecretStore {
		fn get_api_key(
			&self,
			provider: AgentProvider,
		) -> std::result::Result<Option<String>, SecretStoreError> {
			Ok(self
				.entries
				.lock()
				.expect("fake secret store mutex poisoned")
				.get(provider.as_str())
				.cloned()
				.flatten())
		}

		fn set_api_key(
			&self,
			provider: AgentProvider,
			api_key: &str,
		) -> std::result::Result<(), SecretStoreError> {
			self.entries
				.lock()
				.expect("fake secret store mutex poisoned")
				.insert(provider.as_str(), Some(api_key.to_string()));
			Ok(())
		}

		fn delete_api_key(
			&self,
			provider: AgentProvider,
		) -> std::result::Result<(), SecretStoreError> {
			self.entries
				.lock()
				.expect("fake secret store mutex poisoned")
				.insert(provider.as_str(), None);
			Ok(())
		}
	}

	#[test]
	fn invalid_font_size_is_rejected() {
		let config = EditorConfig::default();
		let mut draft = SettingsDraft::from_loaded(&config, &[]);
		draft.base_font_size_px = "huge".to_string();

		let result = draft.save(
			&EditorPaths::for_test(std::path::Path::new(".")),
			&FakeSecretStore::default(),
		);

		assert!(matches!(result, Err(SettingsSaveError::InvalidFontSize(_))));
	}

	#[test]
	fn save_persists_config_and_key_store() -> TestResult<()> {
		let temp = tempfile::tempdir()?;
		let paths = EditorPaths::for_test(temp.path());
		let config = EditorConfig::default();
		let secrets = vec![ResolvedProviderSecret {
			provider: AgentProvider::OpenAi,
			api_key: Some("existing-openai-key".to_string()),
			source: SecretSource::Keychain,
			warnings: Vec::new(),
		}];
		let mut draft = SettingsDraft::from_loaded(&config, &secrets);
		draft.autosave = false;
		draft.base_font_size_px = "18".to_string();
		draft.editor_name = "Editor Test".to_string();
		draft.editor_email = "editor@example.com".to_string();
		draft.provider_mut(AgentProvider::Ollama).api_key = "ollama-key".to_string();
		draft.provider_mut(AgentProvider::GitHubModels).api_key = String::new();
		let store = FakeSecretStore::default();

		draft.save(&paths, &store)?;
		draft.mark_saved();
		let loaded = EditorConfig::load(&paths)?;

		assert!(loaded.exists);
		assert!(!loaded.config.autosave);
		assert_eq!(loaded.config.base_font_size_px, 18);
		assert_eq!(loaded.config.editor_identity.name, "Editor Test");
		assert_eq!(
			loaded.config.editor_identity.email.as_deref(),
			Some("editor@example.com")
		);
		assert_eq!(
			store.get_api_key(AgentProvider::Ollama)?,
			Some("ollama-key".to_string())
		);
		assert_eq!(
			store.get_api_key(AgentProvider::OpenAi)?,
			Some("existing-openai-key".to_string())
		);
		assert_eq!(store.get_api_key(AgentProvider::GitHubModels)?, None);
		assert_eq!(
			draft.provider(AgentProvider::Ollama).source,
			SecretSource::Keychain
		);
		assert_eq!(
			draft.provider(AgentProvider::GitHubModels).source,
			SecretSource::Unavailable
		);
		Ok(())
	}
}
