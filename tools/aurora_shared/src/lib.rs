//! Shared domain library for Aurora tooling surfaces (CLI, editor, VS Code extension).
//! Implements model discovery, parsing, validation, and basic rendering/export helpers.

mod aurora;
mod registry;
pub mod render;

pub use aurora::*;
pub use render::render_error;
pub use render::{Layout, LayoutEdge, LayoutNode, layout_model};
