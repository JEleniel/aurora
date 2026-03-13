//! Settings and first-run UI for the editor shell.

use std::str::FromStr;
use std::sync::Arc;

use dioxus::prelude::*;
use tracing::{error, info};

use crate::EditorSession;
use crate::app::{EditorAppBootstrap, SessionSummary};
use crate::first_run::FirstRunWizardDraft;
use crate::redaction;
use crate::secrets::{AgentProvider, KeyringSecretStore, SecretSource, load_provider_secrets};
use crate::settings::SettingsDraft;
use crate::theme::ThemePalette;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeedbackTone {
	Success,
	Error,
}

/// Status feedback surfaced to the editor shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SaveFeedback {
	tone: FeedbackTone,
	message: String,
}

impl SaveFeedback {
	pub(crate) fn success(message: &str) -> Self {
		Self {
			tone: FeedbackTone::Success,
			message: message.to_string(),
		}
	}

	pub(crate) fn error(message: String) -> Self {
		Self {
			tone: FeedbackTone::Error,
			message,
		}
	}

	pub(crate) fn background(&self, palette: &ThemePalette) -> &'static str {
		match self.tone {
			FeedbackTone::Success => palette.success_background,
			FeedbackTone::Error => palette.error_background,
		}
	}

	pub(crate) fn foreground(&self, palette: &ThemePalette) -> &'static str {
		match self.tone {
			FeedbackTone::Success => palette.success_foreground,
			FeedbackTone::Error => palette.error_foreground,
		}
	}

	pub(crate) fn role(&self) -> &'static str {
		match self.tone {
			FeedbackTone::Success => "status",
			FeedbackTone::Error => "alert",
		}
	}

	pub(crate) fn message(&self) -> &str {
		self.message.as_str()
	}
}

#[derive(Clone)]
struct CompletedFirstRun {
	session: Arc<EditorSession>,
	session_summary: SessionSummary,
	settings_draft: SettingsDraft,
}

#[component]
pub(crate) fn FirstRunWizardScreen(
	bootstrap: EditorAppBootstrap,
	wizard_draft: Signal<FirstRunWizardDraft>,
	draft: Signal<SettingsDraft>,
	baseline: Signal<SettingsDraft>,
	first_run_wizard: Signal<bool>,
	session: Signal<Option<Arc<EditorSession>>>,
	session_summary: Signal<Option<SessionSummary>>,
	status: Signal<Option<SaveFeedback>>,
	palette: ThemePalette,
) -> Element {
	let model_home_display = bootstrap.model_home.display().to_string();

	rsx! {
		section { style: section_style(&palette),
			h2 { style: heading_style(), "First-run setup" }
			p { style: body_style(&palette),
				"Before Aurora opens a model home, it needs your core preferences and audit-log identity."
			}
			p { style: muted_text_style(&palette), "Model home: {model_home_display}" }
			WizardGeneralSection { wizard_draft }
			WizardIdentitySection { wizard_draft }
			FirstRunActions {
				bootstrap,
				wizard_draft,
				draft,
				baseline,
				first_run_wizard,
				session,
				session_summary,
				status,
			}
		}
	}
}

#[component]
fn FirstRunActions(
	bootstrap: EditorAppBootstrap,
	wizard_draft: Signal<FirstRunWizardDraft>,
	draft: Signal<SettingsDraft>,
	baseline: Signal<SettingsDraft>,
	first_run_wizard: Signal<bool>,
	session: Signal<Option<Arc<EditorSession>>>,
	session_summary: Signal<Option<SessionSummary>>,
	status: Signal<Option<SaveFeedback>>,
) -> Element {
	let reset_config = bootstrap.config.clone();

	rsx! {
		div { style: action_row_style(),
			button {
				onclick: {
					let bootstrap = bootstrap.clone();
					move |_| match complete_first_run(&bootstrap, &wizard_draft()) {
						Ok(completed) => {
							register_current_secrets(&completed.settings_draft);
							baseline.set(completed.settings_draft.clone());
							draft.set(completed.settings_draft);
							session.set(Some(completed.session));
							session_summary.set(Some(completed.session_summary));
							first_run_wizard.set(false);
							status
								.set(
									Some(SaveFeedback::success("Setup complete. Aurora is ready.")),
								);
						}
						Err(message) => {
							error!(reason = % message, "aurora_editor first-run setup failed");
							status.set(Some(SaveFeedback::error(message)));
						}
					}
				},
				"Save and continue"
			}
			button {
				onclick: move |_| {
					wizard_draft.set(FirstRunWizardDraft::from_config(&reset_config));
					status.set(Some(SaveFeedback::success("Wizard reset to defaults.")));
				},
				"Reset"
			}
		}
	}
}

#[component]
pub(crate) fn GeneralSection(draft: Signal<SettingsDraft>, palette: ThemePalette) -> Element {
	rsx! {
		section { style: section_style(&palette),
			h2 { style: heading_style(), "General" }
			label { style: field_label_style(),
				"Autosave"
				select {
					value: if draft().autosave { "enabled" } else { "disabled" },
					oninput: move |evt| update_draft(draft, |state| state.autosave = evt.value() == "enabled"),
					option { value: "enabled", "Enabled" }
					option { value: "disabled", "Disabled" }
				}
			}
			label { style: field_label_style(),
				"Theme"
				select {
					value: draft().theme_preference.as_str(),
					oninput: move |evt| update_theme(draft, evt.value().as_str()),
					option { value: "system", "System" }
					option { value: "light", "Light" }
					option { value: "dark", "Dark" }
				}
			}
			label { style: field_label_style(),
				"Base font size (px)"
				input {
					value: draft().base_font_size_px.clone(),
					oninput: move |evt| update_draft(draft, |state| state.base_font_size_px = evt.value()),
					placeholder: "16",
				}
			}
		}
	}
}

#[component]
pub(crate) fn IdentitySection(draft: Signal<SettingsDraft>, palette: ThemePalette) -> Element {
	rsx! {
		section { style: section_style(&palette),
			h2 { style: heading_style(), "Identity" }
			label { style: field_label_style(),
				"Editor name"
				input {
					value: draft().editor_name.clone(),
					oninput: move |evt| update_draft(draft, |state| state.editor_name = evt.value()),
					placeholder: "Aurora Editor",
				}
			}
			label { style: field_label_style(),
				"Editor email"
				input {
					value: draft().editor_email.clone(),
					oninput: move |evt| update_draft(draft, |state| state.editor_email = evt.value()),
					placeholder: "editor@example.com",
				}
			}
		}
	}
}

#[component]
fn WizardGeneralSection(wizard_draft: Signal<FirstRunWizardDraft>) -> Element {
	rsx! {
		label { style: field_label_style(),
			"Autosave"
			select {
				value: if wizard_draft().autosave { "enabled" } else { "disabled" },
				oninput: move |evt| update_wizard(
					wizard_draft,
					|state| state.autosave = evt.value() == "enabled",
				),
				option { value: "enabled", "Enabled" }
				option { value: "disabled", "Disabled" }
			}
		}
		label { style: field_label_style(),
			"Theme"
			select {
				value: wizard_draft().theme_preference.as_str(),
				oninput: move |evt| update_wizard_theme(wizard_draft, evt.value().as_str()),
				option { value: "system", "System" }
				option { value: "light", "Light" }
				option { value: "dark", "Dark" }
			}
		}
	}
}

#[component]
fn WizardIdentitySection(wizard_draft: Signal<FirstRunWizardDraft>) -> Element {
	rsx! {
		label { style: field_label_style(),
			"Editor name"
			input {
				value: wizard_draft().editor_name.clone(),
				oninput: move |evt| update_wizard(wizard_draft, |state| state.editor_name = evt.value()),
				placeholder: "Aurora Editor",
			}
		}
		label { style: field_label_style(),
			"Editor email"
			input {
				value: wizard_draft().editor_email.clone(),
				oninput: move |evt| update_wizard(wizard_draft, |state| state.editor_email = evt.value()),
				placeholder: "editor@example.com",
			}
		}
	}
}

#[component]
pub(crate) fn SettingsActions(
	bootstrap: EditorAppBootstrap,
	draft: Signal<SettingsDraft>,
	baseline: Signal<SettingsDraft>,
	status: Signal<Option<SaveFeedback>>,
) -> Element {
	rsx! {
		div { style: action_row_style(),
			button {
				onclick: {
					let bootstrap = bootstrap.clone();
					move |_| match save_settings(
						&bootstrap.paths,
						&bootstrap.secret_service_name,
						&draft(),
					) {
						Ok(saved) => {
							register_current_secrets(&saved);
							baseline.set(saved.clone());
							draft.set(saved);
							status.set(Some(SaveFeedback::success("Settings saved.")));
							info!("aurora_editor settings saved");
						}
						Err(message) => {
							error!(reason = % message, "aurora_editor settings save failed");
							status.set(Some(SaveFeedback::error(message)));
						}
					}
				},
				"Save settings"
			}
			button {
				onclick: move |_| {
					draft.set(baseline());
					status
						.set(Some(SaveFeedback::success("Changes reset to the last saved state.")));
				},
				"Reset"
			}
		}
	}
}

#[component]
pub(crate) fn ProviderSection(
	draft: Signal<SettingsDraft>,
	provider: AgentProvider,
	palette: ThemePalette,
) -> Element {
	let provider_state = draft().provider(provider).clone();

	rsx! {
		section { style: section_style(&palette),
			h2 { style: heading_style(), "{display_name(provider)}" }
			label { style: field_label_style(),
				"Endpoint"
				input {
					value: provider_state.endpoint.clone(),
					oninput: move |evt| update_draft(
						draft,
						|state| state.provider_mut(provider).endpoint = evt.value(),
					),
					placeholder: "https://...",
				}
			}
			label { style: field_label_style(),
				"API key"
				input {
					r#type: "password",
					value: provider_state.api_key.clone(),
					oninput: move |evt| update_draft(
						draft,
						|state| state.provider_mut(provider).api_key = evt.value(),
					),
					placeholder: provider_state
						.api_key_env_var
						.clone()
						.unwrap_or_else(|| "Stored in OS keychain".to_string()),
				}
			}
			p { style: muted_text_style(&palette),
				"Current source: {secret_source_label(provider_state.source)}"
			}
			if let Some(env_var) = provider_state.api_key_env_var.clone() {
				p { style: muted_text_style(&palette), "Environment fallback: {env_var}" }
			}
			if !provider_state.warnings.is_empty() {
				ul { style: warning_list_style(),
					for warning in provider_state.warnings {
						li { "{warning}" }
					}
				}
			}
		}
	}
}

fn complete_first_run(
	bootstrap: &EditorAppBootstrap,
	wizard_draft: &FirstRunWizardDraft,
) -> Result<CompletedFirstRun, String> {
	let config = wizard_draft
		.complete(&bootstrap.config, &bootstrap.paths)
		.map_err(|error| error.to_string())?;
	let session = bootstrap
		.runtime
		.block_on(async { EditorSession::open(&bootstrap.model_home) })
		.map_err(|error| error.to_string())?;
	let session_summary = SessionSummary::from_session(&session);
	let provider_secrets = load_provider_secrets(
		&config,
		&KeyringSecretStore::new(bootstrap.secret_service_name.as_str()),
	);
	info!(model_home = %bootstrap.model_home.display(), "aurora_editor first-run setup completed");
	Ok(CompletedFirstRun {
		session: Arc::new(session),
		session_summary,
		settings_draft: SettingsDraft::from_loaded(&config, &provider_secrets),
	})
}

fn save_settings(
	paths: &crate::config::EditorPaths,
	secret_service_name: &str,
	settings: &SettingsDraft,
) -> Result<SettingsDraft, String> {
	let store = KeyringSecretStore::new(secret_service_name);
	settings
		.save(paths, &store)
		.map_err(|error| error.to_string())?;
	let mut saved = settings.clone();
	saved.mark_saved();
	Ok(saved)
}

fn register_current_secrets(settings: &SettingsDraft) {
	for provider in AgentProvider::all() {
		let secret = settings.provider(provider).api_key.trim();
		if !secret.is_empty() {
			redaction::register_secret(secret);
		}
	}
}

fn section_style(palette: &ThemePalette) -> String {
	format!(
		"background: {}; border: 1px solid {}; padding: 18px; border-radius: 12px; display: grid; gap: 12px;",
		palette.panel_background, palette.border,
	)
}

fn heading_style() -> &'static str {
	"margin: 0;"
}

fn body_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; line-height: 1.5; color: {};",
		palette.foreground
	)
}

fn muted_text_style(palette: &ThemePalette) -> String {
	format!(
		"margin: 0; line-height: 1.5; color: {};",
		palette.muted_foreground
	)
}

fn action_row_style() -> &'static str {
	"display: flex; gap: 12px; margin-top: 8px; flex-wrap: wrap;"
}

fn field_label_style() -> &'static str {
	"display: grid; gap: 6px; font-weight: 600;"
}

fn warning_list_style() -> &'static str {
	"margin: 10px 0 0 18px; padding: 0; line-height: 1.5;"
}

fn update_draft(mut signal: Signal<SettingsDraft>, update: impl FnOnce(&mut SettingsDraft)) {
	let mut next = signal();
	update(&mut next);
	signal.set(next);
}

fn update_theme(mut signal: Signal<SettingsDraft>, value: &str) {
	let Ok(theme) = crate::config::ThemePreference::from_str(value) else {
		return;
	};
	let mut next = signal();
	next.theme_preference = theme;
	signal.set(next);
}

fn update_wizard(
	mut signal: Signal<FirstRunWizardDraft>,
	update: impl FnOnce(&mut FirstRunWizardDraft),
) {
	let mut next = signal();
	update(&mut next);
	signal.set(next);
}

fn update_wizard_theme(mut signal: Signal<FirstRunWizardDraft>, value: &str) {
	let Ok(theme) = crate::config::ThemePreference::from_str(value) else {
		return;
	};
	let mut next = signal();
	next.theme_preference = theme;
	signal.set(next);
}

fn display_name(provider: AgentProvider) -> &'static str {
	match provider {
		AgentProvider::Ollama => "Ollama",
		AgentProvider::OpenAi => "OpenAI",
		AgentProvider::GitHubModels => "GitHub Models",
	}
}

fn secret_source_label(source: SecretSource) -> &'static str {
	match source {
		SecretSource::Keychain => "OS keychain",
		SecretSource::Environment => "environment fallback",
		SecretSource::Unavailable => "not configured",
	}
}
