//! Constants shared by the Aurora editor.

/// Default path used to locate Aurora model homes when no input is provided.
pub const DEFAULT_MODEL_HOME: &str = "docs/design/aurora";

/// SVG graph width for the context view.
pub const GRAPH_WIDTH: f32 = 800.0;

/// SVG graph height for the context view.
pub const GRAPH_HEIGHT: f32 = 520.0;

/// Default node width for the context view.
pub const GRAPH_NODE_WIDTH: f32 = 180.0;

/// Default node height for the context view.
pub const GRAPH_NODE_HEIGHT: f32 = 96.0;

/// Horizontal/vertical offset between the focused node and adjacent columns.
pub const GRAPH_COLUMN_OFFSET: f32 = 220.0;

/// Vertical spacing between rows in the side columns.
pub const GRAPH_ROW_SPACING: f32 = 120.0;

/// Center X coordinate for the graph view.
pub const GRAPH_CENTER_X: f32 = GRAPH_WIDTH / 2.0;

/// Center Y coordinate for the graph view.
pub const GRAPH_CENTER_Y: f32 = GRAPH_HEIGHT / 2.0;

/// CSS Reset and base styling for the editor UI.
pub const RESET_CSS: &str = include_str!("styles/reset.css");

/// Global CSS styling for the editor UI.
pub const APP_CSS: &str = include_str!("styles/app.css");
