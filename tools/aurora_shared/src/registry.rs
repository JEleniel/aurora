//! Embedded canonical registries derived from the Aurora instruction set.

pub(crate) const CARD_DEFINITIONS: &str = include_str!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/../../.github/instructions/details/1a-Card_Definitions.md"
));

pub(crate) const RELATIONSHIP_DEFINITIONS: &str = include_str!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/../../.github/instructions/details/1b-Relationship_Definitions.md"
));

pub(crate) const VIEW_DEFINITIONS: &str = include_str!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/../../.github/instructions/details/2-View_Definitions.md"
));

pub(crate) const VIEW_STYLING: &str = include_str!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/../../.github/instructions/details/2a-View_Styling_Guide.md"
));
