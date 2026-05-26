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
pub use registry::{
	CardRegistry, MODEL_CONFIGURATION_VERSION, ModelConfiguration,
	ModelConfigurationCardDefinition, ModelConfigurationCommonPropertyDefinition,
	ModelConfigurationRelationshipDefinition, RegistryError, VIEW_CONFIGURATION_VERSION,
	ViewConfiguration, ViewConfigurationCardDefinition, ViewDefinition, ViewDomainDefinition,
	ViewSubdomainDefinition,
};
pub use render::render_error;
pub use render::{
	FocusedGraph, FocusedGraphDocument, FocusedGraphHotspot, FocusedGraphRole, Layout, LayoutEdge,
	LayoutNode, layout_model, render_focused_graph,
};
pub use svg_template_defs::*;
