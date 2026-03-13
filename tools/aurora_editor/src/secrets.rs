//! Secret resolution with keychain-first storage and environment fallback.

use crate::config::{EditorConfig, ProviderConfig};

/// Supported agent providers that may require API keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentProvider {
	Ollama,
	OpenAi,
	GitHubModels,
}

impl AgentProvider {
	/// Stable identifier used in logs and keychain account names.
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Ollama => "ollama",
			Self::OpenAi => "openai",
			Self::GitHubModels => "github_models",
		}
	}

	pub fn all() -> [Self; 3] {
		[Self::Ollama, Self::OpenAi, Self::GitHubModels]
	}

	fn config<'a>(&self, config: &'a EditorConfig) -> &'a ProviderConfig {
		match self {
			Self::Ollama => &config.providers.ollama,
			Self::OpenAi => &config.providers.openai,
			Self::GitHubModels => &config.providers.github_models,
		}
	}

	fn default_env_var(&self) -> &'static str {
		match self {
			Self::Ollama => "OLLAMA_API_KEY",
			Self::OpenAi => "OPENAI_API_KEY",
			Self::GitHubModels => "GITHUB_TOKEN",
		}
	}
}

/// Where a provider secret was sourced from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecretSource {
	Keychain,
	Environment,
	Unavailable,
}

/// Resolved API key state for a single provider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedProviderSecret {
	pub provider: AgentProvider,
	pub api_key: Option<String>,
	pub source: SecretSource,
	pub warnings: Vec<String>,
}

/// Abstract secret store to keep the runtime testable.
pub trait SecretStore {
	fn get_api_key(&self, provider: AgentProvider) -> Result<Option<String>, SecretStoreError>;
	fn set_api_key(&self, provider: AgentProvider, api_key: &str) -> Result<(), SecretStoreError>;
	fn delete_api_key(&self, provider: AgentProvider) -> Result<(), SecretStoreError>;
}

/// OS-keychain secret store backed by the `keyring` crate.
#[derive(Clone, Debug)]
pub struct KeyringSecretStore {
	service_name: String,
}

impl KeyringSecretStore {
	/// Create a new keychain-backed store for the given service name.
	pub fn new(service_name: &str) -> Self {
		Self {
			service_name: service_name.to_string(),
		}
	}

	fn entry_for(&self, provider: AgentProvider) -> Result<keyring::Entry, SecretStoreError> {
		keyring::Entry::new(self.service_name.as_str(), provider.as_str())
			.map_err(SecretStoreError::Keyring)
	}
}

impl SecretStore for KeyringSecretStore {
	fn get_api_key(&self, provider: AgentProvider) -> Result<Option<String>, SecretStoreError> {
		let entry = self.entry_for(provider)?;
		match entry.get_password() {
			Ok(secret) => Ok(Some(secret)),
			Err(keyring::Error::NoEntry) => Ok(None),
			Err(error) => Err(SecretStoreError::Keyring(error)),
		}
	}

	fn set_api_key(&self, provider: AgentProvider, api_key: &str) -> Result<(), SecretStoreError> {
		let entry = self.entry_for(provider)?;
		entry
			.set_password(api_key)
			.map_err(SecretStoreError::Keyring)
	}

	fn delete_api_key(&self, provider: AgentProvider) -> Result<(), SecretStoreError> {
		let entry = self.entry_for(provider)?;
		match entry.delete_credential() {
			Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
			Err(error) => Err(SecretStoreError::Keyring(error)),
		}
	}
}

/// Errors surfaced while accessing secure secret storage.
#[derive(Debug, thiserror::Error)]
pub enum SecretStoreError {
	#[error("keychain access failed: {0}")]
	Keyring(#[from] keyring::Error),
}

/// Resolve all provider secrets using keychain-first semantics with env fallback.
pub fn load_provider_secrets(
	config: &EditorConfig,
	store: &dyn SecretStore,
) -> Vec<ResolvedProviderSecret> {
	AgentProvider::all()
		.into_iter()
		.map(|provider| resolve_provider_secret(provider, config, store, default_env_lookup))
		.collect()
}

fn resolve_provider_secret(
	provider: AgentProvider,
	config: &EditorConfig,
	store: &dyn SecretStore,
	env_lookup: fn(&str) -> Option<String>,
) -> ResolvedProviderSecret {
	match store.get_api_key(provider) {
		Ok(Some(api_key)) => ResolvedProviderSecret {
			provider,
			api_key: Some(api_key),
			source: SecretSource::Keychain,
			warnings: Vec::new(),
		},
		Ok(None) => resolve_from_environment(
			provider,
			config,
			Some(keychain_missing_warning(provider)),
			env_lookup,
		),
		Err(error) => resolve_from_environment(
			provider,
			config,
			Some(keychain_error_warning(provider, &error)),
			env_lookup,
		),
	}
}

fn resolve_from_environment(
	provider: AgentProvider,
	config: &EditorConfig,
	keychain_warning: Option<String>,
	env_lookup: fn(&str) -> Option<String>,
) -> ResolvedProviderSecret {
	let mut warnings = Vec::new();
	if let Some(warning) = keychain_warning {
		warnings.push(warning);
	}

	let env_var = provider
		.config(config)
		.api_key_env_var
		.as_deref()
		.unwrap_or(provider.default_env_var());
	match env_lookup(env_var) {
		Some(api_key) if !api_key.is_empty() => {
			warnings.push(environment_fallback_warning(provider, env_var));
			ResolvedProviderSecret {
				provider,
				api_key: Some(api_key),
				source: SecretSource::Environment,
				warnings,
			}
		}
		_ => ResolvedProviderSecret {
			provider,
			api_key: None,
			source: SecretSource::Unavailable,
			warnings,
		},
	}
}

fn default_env_lookup(name: &str) -> Option<String> {
	std::env::var(name).ok()
}

fn keychain_missing_warning(provider: AgentProvider) -> String {
	format!(
		"no API key stored in the OS keychain for {}; checking environment fallback",
		provider.as_str()
	)
}

fn keychain_error_warning(provider: AgentProvider, error: &SecretStoreError) -> String {
	format!(
		"OS keychain lookup failed for {} ({error}); checking environment fallback",
		provider.as_str()
	)
}

fn environment_fallback_warning(provider: AgentProvider, env_var: &str) -> String {
	format!(
		"using environment fallback {env_var} for {}; store this key in the OS keychain when possible",
		provider.as_str()
	)
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use std::sync::Mutex;

	use super::{
		AgentProvider, EditorConfig, SecretSource, SecretStore, SecretStoreError,
		resolve_provider_secret,
	};

	#[derive(Clone, Debug, PartialEq, Eq)]
	enum FakeResponse {
		Missing,
		Secret(String),
		Error,
	}

	struct FakeSecretStore {
		secrets: Mutex<BTreeMap<&'static str, FakeResponse>>,
	}

	impl FakeSecretStore {
		fn with_secret(provider: AgentProvider, value: &str) -> Self {
			let mut secrets = BTreeMap::new();
			secrets.insert(provider.as_str(), FakeResponse::Secret(value.to_string()));
			Self {
				secrets: Mutex::new(secrets),
			}
		}

		fn with_error(provider: AgentProvider) -> Self {
			let mut secrets = BTreeMap::new();
			secrets.insert(provider.as_str(), FakeResponse::Error);
			Self {
				secrets: Mutex::new(secrets),
			}
		}
	}

	impl Default for FakeSecretStore {
		fn default() -> Self {
			Self {
				secrets: Mutex::new(BTreeMap::new()),
			}
		}
	}

	impl SecretStore for FakeSecretStore {
		fn get_api_key(&self, provider: AgentProvider) -> Result<Option<String>, SecretStoreError> {
			match self
				.secrets
				.lock()
				.expect("fake secret store mutex poisoned")
				.get(provider.as_str())
				.cloned()
				.unwrap_or(FakeResponse::Missing)
			{
				FakeResponse::Missing => Ok(None),
				FakeResponse::Secret(secret) => Ok(Some(secret)),
				FakeResponse::Error => {
					Err(SecretStoreError::Keyring(keyring::Error::NoStorageAccess(
						Box::new(std::io::Error::other("dbus unavailable")),
					)))
				}
			}
		}

		fn set_api_key(
			&self,
			_provider: AgentProvider,
			_api_key: &str,
		) -> Result<(), SecretStoreError> {
			Ok(())
		}

		fn delete_api_key(&self, _provider: AgentProvider) -> Result<(), SecretStoreError> {
			Ok(())
		}
	}

	#[test]
	fn keychain_secret_wins_over_environment_fallback() {
		let config = EditorConfig::default();
		let store = FakeSecretStore::with_secret(AgentProvider::OpenAi, "from-keychain");
		let openai = resolve_provider_secret(AgentProvider::OpenAi, &config, &store, |_| {
			Some("from-env".to_string())
		});

		assert_eq!(openai.source, SecretSource::Keychain);
		assert_eq!(openai.api_key.as_deref(), Some("from-keychain"));
		assert!(openai.warnings.is_empty());
	}

	#[test]
	fn environment_fallback_is_used_when_keychain_fails() {
		let config = EditorConfig::default();
		let store = FakeSecretStore::with_error(AgentProvider::GitHubModels);
		let github_models =
			resolve_provider_secret(AgentProvider::GitHubModels, &config, &store, |name| {
				(name == "GITHUB_TOKEN").then(|| "from-env".to_string())
			});

		assert_eq!(github_models.source, SecretSource::Environment);
		assert_eq!(github_models.api_key.as_deref(), Some("from-env"));
		assert_eq!(github_models.warnings.len(), 2);
	}
}
