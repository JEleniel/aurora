//! First-run configuration wizard state and persistence helpers.

use crate::config::{
	EditorConfig, EditorConfigError, EditorIdentity, EditorPaths, ThemePreference,
};

/// Minimal configuration collected before the editor opens a model home.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FirstRunWizardDraft {
	pub autosave: bool,
	pub theme_preference: ThemePreference,
	pub editor_name: String,
	pub editor_email: String,
}

impl FirstRunWizardDraft {
	/// Seed the wizard from the current configuration defaults.
	pub fn from_config(config: &EditorConfig) -> Self {
		Self {
			autosave: config.autosave,
			theme_preference: config.theme_preference.clone(),
			editor_name: config.editor_identity.name.clone(),
			editor_email: config.editor_identity.email.clone().unwrap_or_default(),
		}
	}

	/// Persist the completed wizard and return the saved editor configuration.
	pub fn complete(
		&self,
		base_config: &EditorConfig,
		paths: &EditorPaths,
	) -> Result<EditorConfig, EditorConfigError> {
		let config = self.to_config(base_config)?;
		config.save(paths)?;
		Ok(config)
	}

	fn to_config(&self, base_config: &EditorConfig) -> Result<EditorConfig, EditorConfigError> {
		let mut config = base_config.clone();
		config.autosave = self.autosave;
		config.theme_preference = self.theme_preference.clone();
		config.editor_identity = EditorIdentity {
			name: self.editor_name.trim().to_string(),
			email: normalize_optional(self.editor_email.as_str()),
		};
		config.validate()?;
		Ok(config)
	}
}

fn normalize_optional(value: &str) -> Option<String> {
	let trimmed = value.trim();
	if trimmed.is_empty() {
		None
	} else {
		Some(trimmed.to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::FirstRunWizardDraft;
	use crate::config::{EditorConfig, EditorConfigError, EditorPaths, ThemePreference};

	type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn completion_persists_initial_config() -> TestResult<()> {
		let temp = tempfile::tempdir()?;
		let paths = EditorPaths::for_test(temp.path());
		let mut draft = FirstRunWizardDraft::from_config(&EditorConfig::default());
		draft.autosave = false;
		draft.theme_preference = ThemePreference::Light;
		draft.editor_name = "First Run User".to_string();
		draft.editor_email = "first.run@example.com".to_string();

		let saved = draft.complete(&EditorConfig::default(), &paths)?;
		let loaded = EditorConfig::load(&paths)?;

		assert!(!saved.autosave);
		assert_eq!(saved.theme_preference, ThemePreference::Light);
		assert_eq!(saved.editor_identity.name, "First Run User");
		assert_eq!(
			saved.editor_identity.email.as_deref(),
			Some("first.run@example.com")
		);
		assert!(loaded.exists);
		assert_eq!(loaded.config, saved);
		Ok(())
	}

	#[test]
	fn blank_identity_is_rejected() {
		let paths = EditorPaths::for_test(std::path::Path::new("."));
		let draft = FirstRunWizardDraft {
			autosave: true,
			theme_preference: ThemePreference::System,
			editor_name: "   ".to_string(),
			editor_email: String::new(),
		};

		let result = draft.complete(&EditorConfig::default(), &paths);

		assert!(matches!(result, Err(EditorConfigError::Invalid(_))));
	}
}
