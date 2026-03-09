//! Shared domain library for Aurora tooling surfaces (CLI, editor, VS Code extension).
//! Implements model discovery, parsing, validation, and basic rendering/export helpers.

mod aurora;
mod background;
mod backup;
mod model_home_session;
mod model_index;
mod registry;
pub mod render;
mod svg_template_defs;

pub use aurora::*;
pub use background::*;
pub use backup::*;
pub use model_home_session::*;
pub use model_index::*;
pub use render::render_error;
pub use render::{Layout, LayoutEdge, LayoutNode, layout_model};
pub use svg_template_defs::*;
