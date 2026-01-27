//! Shared domain library for Aurora tooling surfaces (CLI, editor, VS Code extension).
//! Implements model discovery, parsing, validation, and basic rendering/export helpers.

mod discovery;
mod errors;
mod model;
mod render;
mod validation;

pub use discovery::{ModelHome, discover_model_homes, load_model};
pub use errors::{AuroraError, Result};
pub use model::{AuditEvent, AuditTrail, AuroraModel, Card, Link};
pub use render::{
	RenderSummary, render_all, render_all_with_instructions, render_markdown, render_views,
	render_views_with_instructions, write_compact_model,
};
pub use validation::{
	DiagnosticSeverity, ValidationDiagnostic, ValidationReport, validate_model,
	validate_model_with_instructions,
};
