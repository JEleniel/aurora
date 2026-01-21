//! Lightweight validation logic shared across Aurora tooling stacks.

use crate::{card::AuroraCard, error::AuroraLibError};

/// Perform lightweight validation that complements full schema checks.
pub fn validate_card(card: &AuroraCard) -> Result<(), AuroraLibError> {
	if card.id.trim().is_empty() {
		return Err(AuroraLibError::validation("card id must not be empty"));
	}

	if card.card_type.trim().is_empty() {
		return Err(AuroraLibError::validation("card_type must not be empty"));
	}

	if card.name.trim().is_empty() {
		return Err(AuroraLibError::validation("name must not be empty"));
	}

	if card.description.trim().is_empty() {
		return Err(AuroraLibError::validation("description must not be empty"));
	}

	if card.audit_trail.history.is_empty() {
		return Err(AuroraLibError::validation(
			"audit trail must contain at least one history entry",
		));
	}

	for (index, link) in card.links.iter().enumerate() {
		if link.target.trim().is_empty() {
			return Err(AuroraLibError::validation(format!(
				"link at index {index} is missing a target"
			)));
		}

		if link.relationship.trim().is_empty() {
			return Err(AuroraLibError::validation(format!(
				"link at index {index} is missing a relationship"
			)));
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		AuroraCard,
		card::{AuditEntry, AuditTrail, AuroraLink},
	};

	fn sample_card() -> AuroraCard {
		AuroraCard {
			schema: Some("./Aurora.schema.json".into()),
			id: "MIS-001".into(),
			card_type: "Mission".into(),
			card_subtype: None,
			name: "Enable Deterministic Aurora CLI Tooling".into(),
			description: "Demonstrate deterministic modeling behavior".into(),
			status: None,
			links: vec![AuroraLink {
				target: "DRI-001".into(),
				relationship: "establishes".into(),
			}],
			audit_trail: AuditTrail {
				version: "1.0.0".into(),
				hash: None,
				history: vec![AuditEntry {
					editor: "Test".into(),
					timestamp: "2024-01-01T00:00:00Z".into(),
					event: "created".into(),
					hash: None,
				}],
			},
			attributes: Default::default(),
		}
	}

	#[test]
	fn valid_card_passes_validation() {
		assert!(validate_card(&sample_card()).is_ok());
	}

	#[test]
	fn missing_name_is_rejected() {
		let mut card = sample_card();
		card.name.clear();
		assert!(validate_card(&card).is_err());
	}
}
