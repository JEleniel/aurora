//! Desktop app bootstrap and top-level composition for the editor shell.

use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_desktop::{
	Config as DesktopConfig, LogicalSize, WindowBuilder, WindowEvent, tao::event::Event,
	use_window, use_wry_event_handler,
};
use tokio::runtime::Runtime;

use crate::EditorSession;
use crate::app_settings::{FirstRunWizardScreen, SaveFeedback};
use crate::app_shell::ConfiguredEditorScreen;
use crate::config::{EditorConfig, EditorPaths, ThemePreference};
use crate::first_run::FirstRunWizardDraft;
use crate::secrets::ResolvedProviderSecret;
use crate::settings::SettingsDraft;
use crate::shell::ShellLayout;
use crate::theme::{SystemTheme, ThemePalette, global_theme_css, preferred_window_theme};

/// Service identifier used for keychain storage.
pub const SECRET_SERVICE_NAME: &str = "org.crystultima.aurora";

/// Startup state handed to the desktop app.
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

/// Cheap summary rendered in the header while the full editor shell is still under construction.
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

/// Launch the current desktop editor window.
pub fn launch_editor_app(bootstrap: EditorAppBootstrap) -> ! {
	let cfg = DesktopConfig::new().with_window(
		WindowBuilder::new()
			.with_title("Aurora Editor")
			.with_inner_size(LogicalSize::new(1200.0, 860.0))
			.with_resizable(true)
			.with_theme(preferred_window_theme(
				bootstrap.config.theme_preference.clone(),
			)),
	);
	dioxus_desktop::launch::launch(
		editor_app_root,
		vec![Box::new(move || {
			Box::new(bootstrap.clone()) as Box<dyn Any>
		})],
		vec![Box::new(cfg) as Box<dyn Any>],
	)
}

fn editor_app_root() -> Element {
	let bootstrap = consume_context::<EditorAppBootstrap>();
	rsx! {
		EditorApp { bootstrap }
	}
}

#[component]
fn EditorApp(bootstrap: EditorAppBootstrap) -> Element {
	let draft =
		use_signal(|| SettingsDraft::from_loaded(&bootstrap.config, &bootstrap.provider_secrets));
	let baseline =
		use_signal(|| SettingsDraft::from_loaded(&bootstrap.config, &bootstrap.provider_secrets));
	let wizard_draft = use_signal(|| FirstRunWizardDraft::from_config(&bootstrap.config));
	let first_run_wizard = use_signal(|| bootstrap.require_first_run_wizard);
	let session = use_signal(|| bootstrap.session.clone());
	let session_summary = use_signal(|| bootstrap.session_summary.clone());
	let status = use_signal(|| Option::<SaveFeedback>::None);
	let shell_layout = use_signal(ShellLayout::default);
	let selected_card_id = use_signal(|| initial_selected_card_id(bootstrap.session.as_deref()));
	let window = use_window();
	let system_theme = use_signal(|| SystemTheme::from_window(&window));
	let theme_preference = active_theme_preference(&draft(), &wizard_draft(), first_run_wizard());
	let palette = ThemePalette::resolve(&theme_preference, system_theme());

	use_effect({
		let mut selected_card_id = selected_card_id;
		move || {
			if !selected_card_id().trim().is_empty() {
				return;
			}
			if let Some(current_session) = session()
				&& let Some(root) = current_session.roots().first()
			{
				selected_card_id.set(root.id.clone());
			}
		}
	});

	use_wry_event_handler({
		let mut system_theme = system_theme;
		move |event, _| {
			if let Event::WindowEvent {
				event: WindowEvent::ThemeChanged(theme),
				..
			} = event
			{
				system_theme.set(SystemTheme::from_tao(*theme));
			}
		}
	});

	{
		let window = window.clone();
		let theme_preference = theme_preference.clone();
		use_effect(move || {
			window.set_theme(preferred_window_theme(theme_preference.clone()));
		});
	}

	rsx! {
		style { "{global_theme_css(&palette)}" }
		div { style: app_style(&palette, resolved_font_size(&draft())),
			h1 { style: "margin: 0 0 8px 0;", "Aurora Editor" }
			StatusBanner { status: status(), palette }
			if first_run_wizard() {
				FirstRunWizardScreen {
					bootstrap: bootstrap.clone(),
					wizard_draft,
					draft,
					baseline,
					first_run_wizard,
					session,
					session_summary,
					status,
					palette,
				}
			} else {
				ConfiguredEditorScreen {
					bootstrap,
					session: session(),
					draft,
					baseline,
					shell_layout,
					selected_card_id,
					status,
					summary: session_summary(),
					palette,
				}
			}
		}
	}
}

#[component]
fn StatusBanner(status: Option<SaveFeedback>, palette: ThemePalette) -> Element {
	let Some(status) = status else {
		return rsx! {};
	};

	rsx! {
		div {
			role: status.role(),
			style: format!(
				"margin-bottom: 20px; padding: 12px 14px; border-radius: 8px; background: {}; color: {};",
				status.background(&palette),
				status.foreground(&palette),
			),
			"{status.message()}"
		}
	}
}

fn active_theme_preference(
	draft: &SettingsDraft,
	wizard_draft: &FirstRunWizardDraft,
	first_run_wizard: bool,
) -> ThemePreference {
	if first_run_wizard {
		wizard_draft.theme_preference.clone()
	} else {
		draft.theme_preference.clone()
	}
}

fn app_style(palette: &ThemePalette, font_size: u16) -> String {
	format!(
		"min-height: 100vh; padding: 24px; background: {}; color: {}; font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; font-size: {}px;",
		palette.background, palette.foreground, font_size,
	)
}

fn resolved_font_size(draft: &SettingsDraft) -> u16 {
	draft.base_font_size_px.trim().parse::<u16>().unwrap_or(16)
}

fn initial_selected_card_id(session: Option<&EditorSession>) -> String {
	session
		.and_then(|session| session.roots().first().map(|root| root.id.clone()))
		.unwrap_or_default()
}
