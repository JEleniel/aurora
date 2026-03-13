//! Focused graph workspace for the editor main area.

use std::collections::BTreeMap;
use std::sync::Arc;

use aurora_shared::{
	Card, CardRef, FocusedGraph, FocusedGraphDocument, FocusedGraphHotspot, FocusedGraphRole,
	render_focused_graph,
};
use dioxus::prelude::*;

use crate::EditorSession;
use crate::theme::ThemePalette;

const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 1.75;
const DEFAULT_ZOOM: f32 = 0.72;
const ZOOM_STEP: f32 = 0.1;

#[component]
pub(crate) fn GraphWorkspace(
	session: Option<Arc<EditorSession>>,
	selected_card_id: Signal<String>,
	palette: ThemePalette,
) -> Element {
	let mut zoom = use_signal(|| DEFAULT_ZOOM);
	let document = use_signal(|| Option::<FocusedGraphDocument>::None);
	let error = use_signal(|| Option::<String>::None);

	use_effect({
		let session = session.clone();
		let mut document = document;
		let mut error = error;
		move || {
			let current = selected_card_id();
			let Some(session) = session.as_ref() else {
				document.set(None);
				error.set(Some("Editor session is not available yet.".to_string()));
				return;
			};
			if current.trim().is_empty() {
				document.set(None);
				error.set(Some(
					"Select a card to render the focused graph.".to_string(),
				));
				return;
			}
			match build_document(session, current.as_str()) {
				Ok(next) => {
					document.set(Some(next));
					error.set(None);
				}
				Err(message) => {
					document.set(None);
					error.set(Some(message));
				}
			}
		}
	});

	let zoom_percent = (zoom() * 100.0).round() as i32;
	let grouped = group_hotspots(document().as_ref());

	rsx! {
		style { ".aurora-graph-canvas svg {{ width: 100%; height: 100%; display: block; }}" }
		div { style: workspace_style(),
			GraphToolbar {
				selected_card_id: selected_card_id(),
				zoom_percent,
				on_zoom_out: move |_| zoom.set(adjust_zoom(zoom(), -ZOOM_STEP)),
				on_zoom_reset: move |_| zoom.set(DEFAULT_ZOOM),
				on_zoom_in: move |_| zoom.set(adjust_zoom(zoom(), ZOOM_STEP)),
				palette,
			}
			if let Some(message) = error() {
				GraphError { message, palette }
			} else if let Some(document) = document() {
				GraphCanvas {
					document: document.clone(),
					selected_card_id: selected_card_id(),
					zoom: zoom(),
					on_zoom_out: move |_| zoom.set(adjust_zoom(zoom(), -ZOOM_STEP)),
					on_zoom_reset: move |_| zoom.set(DEFAULT_ZOOM),
					on_zoom_in: move |_| zoom.set(adjust_zoom(zoom(), ZOOM_STEP)),
					on_select: move |card_id| selected_card_id.set(card_id),
					palette,
				}
				GraphNavigation {
					groups: grouped,
					selected_card_id: selected_card_id(),
					on_select: move |card_id| selected_card_id.set(card_id),
					palette,
				}
			}
		}
	}
}

#[component]
fn GraphToolbar(
	selected_card_id: String,
	zoom_percent: i32,
	on_zoom_out: EventHandler<MouseEvent>,
	on_zoom_reset: EventHandler<MouseEvent>,
	on_zoom_in: EventHandler<MouseEvent>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		div { style: toolbar_style(&palette),
			div { style: "display: grid; gap: 4px;",
				strong { "Focused card" }
				span { style: muted_text_style(&palette), "{selected_card_id}" }
				span { style: muted_text_style(&palette),
					"Pan with scroll or trackpad. Focus the canvas and use +, -, or 0 to control zoom."
				}
			}
			div { style: "display: flex; gap: 10px; align-items: center; flex-wrap: wrap;",
				button { onclick: move |event| on_zoom_out.call(event), "Zoom out" }
				button { onclick: move |event| on_zoom_reset.call(event), "Reset" }
				button { onclick: move |event| on_zoom_in.call(event), "Zoom in" }
				span { style: muted_text_style(&palette), "{zoom_percent}%" }
			}
		}
	}
}

#[component]
fn GraphCanvas(
	document: FocusedGraphDocument,
	selected_card_id: String,
	zoom: f32,
	on_zoom_out: EventHandler<KeyboardEvent>,
	on_zoom_reset: EventHandler<KeyboardEvent>,
	on_zoom_in: EventHandler<KeyboardEvent>,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	let scaled_width = scale_pixels(document.width_px, zoom);
	let scaled_height = scale_pixels(document.height_px, zoom);

	rsx! {
		div {
			aria_label: "Focused graph canvas",
			tabindex: "0",
			onkeydown: move |event| {
				match zoom_key_action(event.key().to_string().as_str()) {
					Some(ZoomKeyAction::ZoomOut) => {
						event.prevent_default();
						on_zoom_out.call(event);
					}
					Some(ZoomKeyAction::Reset) => {
						event.prevent_default();
						on_zoom_reset.call(event);
					}
					Some(ZoomKeyAction::ZoomIn) => {
						event.prevent_default();
						on_zoom_in.call(event);
					}
					None => {}
				}
			},
			style: viewport_style(&palette),
			div {
				class: "aurora-graph-canvas",
				style: canvas_style(scaled_width, scaled_height, &palette),
				div {
					style: "position: absolute; inset: 0;",
					dangerous_inner_html: "{document.svg}",
				}
				for hotspot in document.hotspots {
					button {
						key: "{hotspot.card_id}-{hotspot.role.label()}",
						aria_label: hotspot.card_label.clone(),
						title: hotspot.card_label.clone(),
						style: hotspot_style(&hotspot, zoom, selected_card_id.as_str(), &palette),
						onclick: {
							let next_id = hotspot.card_id.clone();
							move |_| on_select.call(next_id.clone())
						},
						span { style: sr_only_style(), "{hotspot.card_label}" }
					}
				}
			}
		}
	}
}

#[component]
fn GraphNavigation(
	groups: BTreeMap<FocusedGraphRole, Vec<FocusedGraphHotspot>>,
	selected_card_id: String,
	on_select: EventHandler<String>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		div { style: navigation_panel_style(&palette),
			for role in [FocusedGraphRole::Parent, FocusedGraphRole::Sibling, FocusedGraphRole::Child] {
				if let Some(cards) = groups.get(&role) {
					if !cards.is_empty() {
						section {
							key: "{role.label()}",
							style: "display: grid; gap: 10px;",
							h3 { style: "margin: 0;", "{role.label()}" }
							div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
								for card in cards {
									button {
										key: "{card.card_id}",
										style: navigation_button_style(selected_card_id.as_str(), card.card_id.as_str(), &palette),
										onclick: {
											let next_id = card.card_id.clone();
											move |_| on_select.call(next_id.clone())
										},
										"{card.card_label}"
									}
								}
							}
						}
					}
				}
			}
		}
	}
}

#[component]
fn GraphError(message: String, palette: ThemePalette) -> Element {
	rsx! {
		div {
			role: "alert",
			style: format!(
				"padding: 16px; border-radius: 12px; background: {}; color: {};",
				palette.error_background,
				palette.error_foreground,
			),
			"{message}"
		}
	}
}

fn build_document(
	session: &EditorSession,
	selected_card_id: &str,
) -> Result<FocusedGraphDocument, String> {
	if session.svg_template().trim().is_empty() {
		return Err(
			"The model home is missing reference/SVGTemplate.svgz (or SVGTemplate.svg), so the graph view cannot render yet."
				.to_string(),
		);
	}
	let graph = load_focused_graph(session, selected_card_id)?;
	render_focused_graph(&graph, session.card_registry(), session.svg_template())
		.map_err(|error| error.to_string())
}

fn load_focused_graph(
	session: &EditorSession,
	selected_card_id: &str,
) -> Result<FocusedGraph, String> {
	let center = session
		.load_card(selected_card_id)
		.map_err(|error| error.to_string())?
		.ok_or_else(|| {
			format!("Card {selected_card_id} is not available in the current model home.")
		})?;
	let parents = load_cards_from_refs(
		session,
		session
			.cards_linking_to(selected_card_id)
			.map_err(|error| error.to_string())?,
	)?;
	let children = load_cards_from_ids(
		session,
		center
			.links
			.iter()
			.map(|link| link.target.clone())
			.collect(),
	)?;
	let siblings = collect_siblings(session, &parents, center.id.as_str())?;
	Ok(FocusedGraph::new(
		center,
		parents,
		siblings,
		children,
		session.model_home().to_path_buf(),
		session.model_home().join("focused"),
	))
}

fn load_cards_from_refs(session: &EditorSession, refs: Vec<CardRef>) -> Result<Vec<Card>, String> {
	load_cards_from_ids(session, refs.into_iter().map(|card| card.id).collect())
}

fn load_cards_from_ids(session: &EditorSession, ids: Vec<String>) -> Result<Vec<Card>, String> {
	let mut by_id = BTreeMap::new();
	for card_id in ids {
		let Some(card) = session
			.load_card(card_id.as_str())
			.map_err(|error| error.to_string())?
		else {
			continue;
		};
		by_id.insert(card.id.clone(), card);
	}
	Ok(by_id.into_values().collect())
}

fn collect_siblings(
	session: &EditorSession,
	parents: &[Card],
	selected_card_id: &str,
) -> Result<Vec<Card>, String> {
	let ids = parents
		.iter()
		.flat_map(|parent| parent.links.iter().map(|link| link.target.clone()))
		.filter(|target| target != selected_card_id)
		.collect::<Vec<_>>();
	load_cards_from_ids(session, ids)
}

fn group_hotspots(
	document: Option<&FocusedGraphDocument>,
) -> BTreeMap<FocusedGraphRole, Vec<FocusedGraphHotspot>> {
	let mut groups = BTreeMap::new();
	if let Some(document) = document {
		for hotspot in &document.hotspots {
			groups
				.entry(hotspot.role)
				.or_insert_with(Vec::new)
				.push(hotspot.clone());
		}
	}
	groups
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ZoomKeyAction {
	ZoomOut,
	Reset,
	ZoomIn,
}

fn zoom_key_action(key: &str) -> Option<ZoomKeyAction> {
	match key {
		"-" | "_" => Some(ZoomKeyAction::ZoomOut),
		"0" => Some(ZoomKeyAction::Reset),
		"+" | "=" => Some(ZoomKeyAction::ZoomIn),
		_ => None,
	}
}

fn adjust_zoom(current: f32, delta: f32) -> f32 {
	(current + delta).clamp(MIN_ZOOM, MAX_ZOOM)
}

fn scale_pixels(value: i32, zoom: f32) -> i32 {
	((value as f32) * zoom).round().max(1.0) as i32
}

fn workspace_style() -> &'static str {
	"display: grid; gap: 14px; min-height: 100%; align-content: start;"
}

fn toolbar_style(palette: &ThemePalette) -> String {
	format!(
		"display: flex; justify-content: space-between; gap: 12px; flex-wrap: wrap; padding: 14px; border: 1px solid {}; border-radius: 12px; background: {};",
		palette.border, palette.panel_background,
	)
}

fn viewport_style(palette: &ThemePalette) -> String {
	format!(
		"min-height: 420px; max-height: 620px; overflow: auto; border-radius: 14px; border: 1px solid {}; background: {}; padding: 12px;",
		palette.border, palette.background,
	)
}

fn canvas_style(width_px: i32, height_px: i32, palette: &ThemePalette) -> String {
	format!(
		"position: relative; width: {}px; height: {}px; border-radius: 12px; background: {};",
		width_px, height_px, palette.panel_background,
	)
}

fn hotspot_style(
	hotspot: &FocusedGraphHotspot,
	zoom: f32,
	selected_card_id: &str,
	palette: &ThemePalette,
) -> String {
	let border = if hotspot.card_id == selected_card_id {
		palette.accent
	} else {
		"transparent"
	};
	format!(
		"position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; padding: 0; background: transparent; border: 3px solid {}; border-radius: 14px; opacity: 0.75;",
		scale_pixels(hotspot.x_px, zoom),
		scale_pixels(hotspot.y_px, zoom),
		scale_pixels(hotspot.width_px, zoom),
		scale_pixels(hotspot.height_px, zoom),
		border,
	)
}

fn navigation_panel_style(palette: &ThemePalette) -> String {
	format!(
		"display: grid; gap: 14px; padding: 14px; border-radius: 12px; background: {}; border: 1px solid {};",
		palette.panel_background, palette.border,
	)
}

fn navigation_button_style(
	selected_card_id: &str,
	card_id: &str,
	palette: &ThemePalette,
) -> String {
	if selected_card_id == card_id {
		return format!(
			"background: {}; color: {}; border-color: {};",
			palette.accent, palette.button_foreground, palette.accent,
		);
	}
	format!(
		"background: {}; color: {}; border-color: {};",
		palette.button_background, palette.button_foreground, palette.border,
	)
}

fn muted_text_style(palette: &ThemePalette) -> String {
	format!("margin: 0; color: {};", palette.muted_foreground)
}

fn sr_only_style() -> &'static str {
	"position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;"
}

#[cfg(test)]
mod tests {
	use super::{ZoomKeyAction, adjust_zoom, zoom_key_action};

	#[test]
	fn keyboard_shortcuts_map_to_zoom_actions() {
		assert_eq!(zoom_key_action("-"), Some(ZoomKeyAction::ZoomOut));
		assert_eq!(zoom_key_action("_"), Some(ZoomKeyAction::ZoomOut));
		assert_eq!(zoom_key_action("0"), Some(ZoomKeyAction::Reset));
		assert_eq!(zoom_key_action("+"), Some(ZoomKeyAction::ZoomIn));
		assert_eq!(zoom_key_action("="), Some(ZoomKeyAction::ZoomIn));
		assert_eq!(zoom_key_action("x"), None);
	}

	#[test]
	fn zoom_is_clamped_to_supported_bounds() {
		assert_eq!(adjust_zoom(0.5, -0.2), 0.5);
		assert_eq!(adjust_zoom(1.75, 0.5), 1.75);
	}
}
