use serde::{Deserialize, Serialize};

use crate::registry::CardDefinition;

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
	pub target: String,
	pub relationship: String,
}

impl Link {
	pub fn get_markdown(&self) -> String {
		format!(
			"- {} [{}](../{}/{}.md)\n",
			self.relationship,
			self.target,
			CardDefinition::get_by_acronym(&self.target[0..3]).card_type,
			self.target,
		)
	}
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod link_tests;
