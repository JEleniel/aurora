//! Header rendering for the editor UI.

use dioxus::prelude::*;

use crate::state::AppState;

use super::assets::logo_asset;

/// Renders the editor header bar.
pub(crate) fn render_header(state: &AppState) -> Element {
	let logo = logo_asset();
	let mission_label = state
		.current_mission()
		.map(|mission| format!("{} — {}", mission.mission_id, mission.mission_name));
	let model_home = state
		.model_home
		.as_ref()
		.map(|path| path.display().to_string())
		.unwrap_or_else(|| "No model loaded".to_string());

	rsx! {
		header {
			img {
				class: "header-logo",
				src: "{logo.src}",
				srcset: "{logo.srcset}",
				alt: "Aurora logo",
			}
			div { class: "header-title",
				h1 { "Aurora Editor" }
				div { class: "header-meta",
					div { class: "small", "Model:" }
					div { class: "model-home", "{model_home}" }
					if let Some(label) = mission_label {
						div { "Mission: {label}" }
					}
				}
			}
		}
	}
}
