use serde::{Deserialize, Serialize};

use crate::{CardError, registry::CardRegistry};

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
	pub target: String,
	pub relationship: String,
}

impl Link {
	pub fn markdown_with_href(&self, href: &str) -> String {
		format!("- {} [{}]({})\n", self.relationship, self.target, href)
	}

	pub fn try_get_markdown(&self) -> Result<String, CardError> {
		let registry = CardRegistry::try_new()?;
		Ok(format!(
			"- {} [{}](../{}/{}.md)\n",
			self.relationship,
			self.target,
			registry.try_get_by_acronym(&self.target[0..3])?.card_type,
			self.target,
		))
	}
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod link_tests;
