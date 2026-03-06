use serde::{Deserialize, Serialize};

use crate::{CardError, registry::CardRegistry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
	pub target: String,
	pub relationship: String,
}

impl Link {
	pub fn markdown_with_href(&self, href: &str) -> String {
		format!("- {} [{}]({})\n", self.relationship, self.target, href)
	}

	pub fn try_get_markdown(&self, registry: &CardRegistry) -> Result<String, CardError> {
		let target_acronym = self
			.target
			.split('-')
			.next()
			.map(str::trim)
			.unwrap_or_default();
		if target_acronym.len() != 3 {
			return Err(CardError::InvalidCard(format!(
				"Invalid link target id format: {}",
				self.target
			)));
		}

		Ok(format!(
			"- {} [{}](../{}/{}.md)\n",
			self.relationship,
			self.target,
			registry.try_get_by_acronym(target_acronym)?.card_type,
			self.target,
		))
	}
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod link_tests;
