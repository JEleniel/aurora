//! Four-region workspace shell for the editor UI.

use std::sync::Arc;

use dioxus::prelude::*;

use crate::EditorSession;
use crate::app::{EditorAppBootstrap, SessionSummary};
use crate::app_settings::SaveFeedback;
use crate::bottom_panel::BottomPanelTabs;
use crate::graph_view::GraphWorkspace;
use crate::inspector_sidebar::InspectorSidebar;
use crate::navigation_sidebar::NavigationSidebar;
use crate::settings::SettingsDraft;
use crate::shell::{ResizeCommand, ShellLayout, ShellRegion, resize_command_for_key};
use crate::theme::ThemePalette;

const LEFT_SIDEBAR_REGION_ID: &str = "left-sidebar-region";
const MAIN_WORKSPACE_REGION_ID: &str = "main-workspace-region";
const RIGHT_SIDEBAR_REGION_ID: &str = "right-sidebar-region";
const BOTTOM_PANEL_REGION_ID: &str = "bottom-panel-region";

#[component]
pub(crate) fn ConfiguredEditorScreen(
	bootstrap: EditorAppBootstrap,
	session: Option<Arc<EditorSession>>,
	draft: Signal<SettingsDraft>,
	baseline: Signal<SettingsDraft>,
	shell_layout: Signal<ShellLayout>,
	selected_card_id: Signal<String>,
	status: Signal<Option<SaveFeedback>>,
	summary: Option<SessionSummary>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		EditorWorkspaceShell {
			bootstrap,
			session,
			draft,
			baseline,
			shell_layout,
			selected_card_id,
			status,
			summary,
			palette,
		}
	}
}

#[component]
fn EditorWorkspaceShell(
	bootstrap: EditorAppBootstrap,
	session: Option<Arc<EditorSession>>,
	draft: Signal<SettingsDraft>,
	baseline: Signal<SettingsDraft>,
	shell_layout: Signal<ShellLayout>,
	selected_card_id: Signal<String>,
	status: Signal<Option<SaveFeedback>>,
	summary: Option<SessionSummary>,
	palette: ThemePalette,
) -> Element {
	let layout = shell_layout();

	rsx! {
		SummaryHeader { summary: summary.clone(), palette }
		WorkspaceGrid {
			bootstrap,
			session,
			draft,
			baseline,
			shell_layout,
			selected_card_id,
			status,
			summary,
			layout,
			palette,
		}
	}
}

#[component]
fn WorkspaceGrid(
	bootstrap: EditorAppBootstrap,
	session: Option<Arc<EditorSession>>,
	draft: Signal<SettingsDraft>,
	baseline: Signal<SettingsDraft>,
	shell_layout: Signal<ShellLayout>,
	selected_card_id: Signal<String>,
	status: Signal<Option<SaveFeedback>>,
	summary: Option<SessionSummary>,
	layout: ShellLayout,
	palette: ThemePalette,
) -> Element {
	rsx! {
		div { style: workspace_shell_style(layout),
			LeftSidebarRegion {
				session: session.clone(),
				selected_card_id,
				summary: summary.clone(),
				layout,
				draft,
				palette,
			}
			LeftSidebarRail { shell_layout, layout, palette }
			MainWorkspaceRegion { session: session.clone(), selected_card_id, palette }
			RightSidebarRail { shell_layout, layout, palette }
			RightSidebarRegion {
				bootstrap,
				session: session.clone(),
				selected_card_id,
				draft,
				baseline,
				status,
				palette,
			}
			BottomPanelRail { shell_layout, layout, palette }
			BottomPanelRegion { session, selected_card_id, palette }
		}
	}
}

#[component]
fn SummaryHeader(summary: Option<SessionSummary>, palette: ThemePalette) -> Element {
	let Some(summary) = summary else {
		return rsx! {
			p { style: header_text_style(&palette), "Complete setup to open the selected model home." }
		};
	};

	rsx! {
		div { style: "display: grid; gap: 4px; margin-bottom: 24px;",
			p { style: header_text_style(&palette), "Model home: {summary.model_home_display}" }
			p { style: header_text_style(&palette), "Root missions loaded: {summary.root_count}" }
		}
	}
}

#[component]
fn LeftSidebarRegion(
	session: Option<Arc<EditorSession>>,
	selected_card_id: Signal<String>,
	summary: Option<SessionSummary>,
	layout: ShellLayout,
	draft: Signal<SettingsDraft>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		aside {
			id: LEFT_SIDEBAR_REGION_ID,
			aria_label: "Navigation sidebar",
			style: workspace_region_style(
				&palette,
				"grid-column: 1; grid-row: 1 / span 3; overflow: auto;",
			),
			NavigationSidebar {
				session,
				selected_card_id,
				summary,
				layout,
				draft,
				palette,
			}
		}
	}
}

#[component]
fn LeftSidebarRail(
	shell_layout: Signal<ShellLayout>,
	layout: ShellLayout,
	palette: ThemePalette,
) -> Element {
	rsx! {
		div {
			class: "aurora-resize-rail",
			role: "group",
			tabindex: "0",
			aria_label: "Resize left sidebar",
			title: "Use Left and Right arrow keys to resize. Home and End jump to the bounds.",
			aria_controls: "left-sidebar-region main-workspace-region",
			onkeydown: move |event| {
				let key = event.key().to_string();
				if let Some(command) = resize_command_for_key(
					ShellRegion::LeftSidebar,
					key.as_str(),
				) {
					event.prevent_default();
					apply_resize_command(shell_layout, ShellRegion::LeftSidebar, command);
				}
			},
			style: resize_rail_style(&palette, "grid-column: 2; grid-row: 1 / span 3;", "col-resize"),
			SidebarResizeRail {
				title: "Left sidebar",
				shrink_label: "Shrink left sidebar",
				grow_label: "Grow left sidebar",
				current_percent: layout.percent_for(ShellRegion::LeftSidebar),
				on_shrink: move |_| shell_layout.set(shell_layout().shrink_left()),
				on_grow: move |_| shell_layout.set(shell_layout().grow_left()),
			}
		}
	}
}

#[component]
fn MainWorkspaceRegion(
	session: Option<Arc<EditorSession>>,
	selected_card_id: Signal<String>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		main {
			id: MAIN_WORKSPACE_REGION_ID,
			aria_label: "Graph workspace",
			style: workspace_region_style(&palette, "grid-column: 3; grid-row: 1; overflow: auto;"),
			GraphWorkspace { session, selected_card_id, palette }
		}
	}
}

#[component]
fn RightSidebarRail(
	shell_layout: Signal<ShellLayout>,
	layout: ShellLayout,
	palette: ThemePalette,
) -> Element {
	rsx! {
		div {
			class: "aurora-resize-rail",
			role: "group",
			tabindex: "0",
			aria_label: "Resize right sidebar",
			title: "Use Left and Right arrow keys to resize. Home and End jump to the bounds.",
			aria_controls: "main-workspace-region right-sidebar-region",
			onkeydown: move |event| {
				let key = event.key().to_string();
				if let Some(command) = resize_command_for_key(
					ShellRegion::RightSidebar,
					key.as_str(),
				) {
					event.prevent_default();
					apply_resize_command(shell_layout, ShellRegion::RightSidebar, command);
				}
			},
			style: resize_rail_style(&palette, "grid-column: 4; grid-row: 1 / span 3;", "col-resize"),
			SidebarResizeRail {
				title: "Right sidebar",
				shrink_label: "Shrink right sidebar",
				grow_label: "Grow right sidebar",
				current_percent: layout.percent_for(ShellRegion::RightSidebar),
				on_shrink: move |_| shell_layout.set(shell_layout().shrink_right()),
				on_grow: move |_| shell_layout.set(shell_layout().grow_right()),
			}
		}
	}
}

#[component]
fn RightSidebarRegion(
	bootstrap: EditorAppBootstrap,
	session: Option<Arc<EditorSession>>,
	selected_card_id: Signal<String>,
	draft: Signal<SettingsDraft>,
	baseline: Signal<SettingsDraft>,
	status: Signal<Option<SaveFeedback>>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		aside {
			id: RIGHT_SIDEBAR_REGION_ID,
			aria_label: "Inspector sidebar",
			style: workspace_region_style(
				&palette,
				"grid-column: 5; grid-row: 1 / span 3; overflow: auto;",
			),
			InspectorSidebar {
				bootstrap,
				session,
				selected_card_id,
				draft,
				baseline,
				status,
				palette,
			}
		}
	}
}

#[component]
fn BottomPanelRail(
	shell_layout: Signal<ShellLayout>,
	layout: ShellLayout,
	palette: ThemePalette,
) -> Element {
	rsx! {
		div {
			class: "aurora-resize-rail",
			role: "group",
			tabindex: "0",
			aria_label: "Resize bottom panel",
			title: "Use Up and Down arrow keys to resize. Home and End jump to the bounds.",
			aria_controls: "main-workspace-region bottom-panel-region",
			onkeydown: move |event| {
				let key = event.key().to_string();
				if let Some(command) = resize_command_for_key(
					ShellRegion::BottomPanel,
					key.as_str(),
				) {
					event.prevent_default();
					apply_resize_command(shell_layout, ShellRegion::BottomPanel, command);
				}
			},
			style: resize_rail_style(
				&palette,
				"grid-column: 3; grid-row: 2; flex-direction: row;",
				"row-resize",
			),
			BottomResizeRail {
				current_percent: layout.percent_for(ShellRegion::BottomPanel),
				on_shrink: move |_| shell_layout.set(shell_layout().shrink_bottom()),
				on_grow: move |_| shell_layout.set(shell_layout().grow_bottom()),
			}
		}
	}
}

#[component]
fn BottomPanelRegion(
	session: Option<Arc<EditorSession>>,
	selected_card_id: Signal<String>,
	palette: ThemePalette,
) -> Element {
	rsx! {
		section {
			id: BOTTOM_PANEL_REGION_ID,
			aria_label: "Diagnostics and providers panel",
			style: workspace_region_style(&palette, "grid-column: 3; grid-row: 3; overflow: auto;"),
			BottomPanelTabs { session, selected_card_id, palette }
		}
	}
}

#[component]
fn SidebarResizeRail(
	title: &'static str,
	shrink_label: &'static str,
	grow_label: &'static str,
	current_percent: u8,
	on_shrink: EventHandler<MouseEvent>,
	on_grow: EventHandler<MouseEvent>,
) -> Element {
	rsx! {
		div { style: rail_content_style(),
			p { style: "margin: 0; font-size: 12px; line-height: 1.4;", "{title}" }
			button {
				aria_label: shrink_label,
				title: shrink_label,
				onclick: move |event| on_shrink.call(event),
				style: resize_button_style(),
				"−"
			}
			span { style: "font-weight: 700;", "{current_percent}%" }
			button {
				aria_label: grow_label,
				title: grow_label,
				onclick: move |event| on_grow.call(event),
				style: resize_button_style(),
				"+"
			}
		}
	}
}

#[component]
fn BottomResizeRail(
	current_percent: u8,
	on_shrink: EventHandler<MouseEvent>,
	on_grow: EventHandler<MouseEvent>,
) -> Element {
	rsx! {
		div { style: "display: flex; align-items: center; gap: 12px; justify-content: center; width: 100%;",
			span { style: "font-size: 12px;", "Bottom panel" }
			button {
				aria_label: "Shrink bottom panel",
				title: "Shrink bottom panel",
				onclick: move |event| on_shrink.call(event),
				style: resize_button_style(),
				"−"
			}
			span { style: "font-weight: 700; min-width: 44px; text-align: center;", "{current_percent}%" }
			button {
				aria_label: "Grow bottom panel",
				title: "Grow bottom panel",
				onclick: move |event| on_grow.call(event),
				style: resize_button_style(),
				"+"
			}
		}
	}
}

fn workspace_shell_style(layout: ShellLayout) -> String {
	format!(
		"display: grid; gap: 0; grid-template-columns: {}; grid-template-rows: {}; min-height: calc(100vh - 180px);",
		layout.column_template(),
		layout.row_template(),
	)
}

fn workspace_region_style(palette: &ThemePalette, placement: &str) -> String {
	format!(
		"{placement} background: {}; border: 1px solid {}; border-radius: 12px; padding: 18px; display: grid; gap: 14px;",
		palette.panel_background, palette.border,
	)
}

fn resize_rail_style(palette: &ThemePalette, placement: &str, cursor: &str) -> String {
	format!(
		"{placement} display: flex; align-items: center; justify-content: center; padding: 4px; background: {}; border-inline: 1px solid {}; cursor: {}; user-select: none;",
		palette.rail_background, palette.border, cursor,
	)
}

fn apply_resize_command(
	mut shell_layout: Signal<ShellLayout>,
	region: ShellRegion,
	command: ResizeCommand,
) {
	shell_layout.set(shell_layout().adjust_region(region, command));
}

fn header_text_style(palette: &ThemePalette) -> String {
	format!("margin: 0; color: {};", palette.muted_foreground)
}

fn rail_content_style() -> &'static str {
	"display: grid; gap: 10px; justify-items: center; text-align: center;"
}

fn resize_button_style() -> &'static str {
	"min-width: 36px; min-height: 36px; border-radius: 999px; display: grid; place-items: center; padding: 0; font-size: 20px; font-weight: 700;"
}
