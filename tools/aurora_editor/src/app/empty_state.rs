//! Empty-state rendering helpers.

use dioxus::prelude::*;

use super::assets::logo_asset;

/// Renders a standard empty state banner with the Aurora logo.
pub(crate) fn render_empty_state(message: &str) -> Element {
	let logo = logo_asset();
	rsx! {
		div { class: "empty-state",
			img {
				class: "empty-logo",
				src: "{logo.src}",
				srcset: "{logo.srcset}",
				alt: "Aurora logo",
			}
			div { class: "inline-muted", "{message}" }
		}
	}
}
