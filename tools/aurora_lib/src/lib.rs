//! Core data structures and helpers shared between Aurora tools.
//!
//! The library exposes a lightweight representation of an Aurora card,
//! along with deterministic read, write, and validation helpers. The
//! intent is to keep business logic in one place so that both the CLI
//! tooling and the GUI editor can remain consistent.

/// Data structures that model Aurora cards and links.
pub mod card;
/// Common error enumeration shared by helpers.
pub mod error;
/// Deterministic IO helpers.
pub mod io;
/// Lightweight validation routines.
pub mod validation;

pub use card::{AuditEntry, AuditTrail, AuroraCard, AuroraLink};
pub use error::AuroraLibError;
pub use io::{read_card_from_path, write_card_to_path};
pub use validation::validate_card;
