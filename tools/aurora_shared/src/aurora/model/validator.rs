use super::Model;

pub(crate) fn id_prefix(id: &str) -> Option<&str> {
	id.split('-').next()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
	pub errors: Vec<String>,
}

impl Model {
	pub fn validate(&self) -> ValidationReport {
		let mut errors: Vec<String> = Vec::new();

		// Check for schema errors
		errors.extend(self.get_schema_validation_errors());

		// Check for invariant violations
		errors.extend(self.validate_no_mission_incoming_links());
		errors.extend(self.validate_reachability_and_orphans());
		errors.extend(self.validate_broken_links());

		ValidationReport { errors }
	}
}
