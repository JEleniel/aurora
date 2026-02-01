//! CLI constants for Aurora model operations.

/// Default path used when no input is provided.
pub const DEFAULT_INPUT: &str = "docs/design/aurora";

/// Default output directory for rendered artifacts.
pub const DEFAULT_OUTPUT: &str = "docs/design/";

/// Maximum depth to walk when searching for model homes.
pub const MAX_DISCOVERY_DEPTH: usize = 6;

/// Candidate schema filenames for full card schemas.
pub const CARD_SCHEMA_FILES: [&str; 2] = ["Aurora.schema.jsjson", "Aurora.schema.json"];

/// Candidate schema filenames for compact card schemas.
pub const COMPACT_SCHEMA_FILES: [&str; 2] =
	["Aurora.compact.schema.jsjson", "Aurora.compact.schema.json"];
