//! Editor session orchestration for bounded-load model-home startup.

use std::collections::{BTreeSet, VecDeque};
use std::path::Path;

use aurora_shared::{
	Card, CardRef, CardRegistry, ModelHomeSession, ModelHomeSessionError, ModelIndex,
	ModelIndexError, ModelRootCard, RegistryError, SvgTemplateDefsError, load_svg_template,
};
use thiserror::Error;
use tracing::warn;

/// Bounded-working-set editor session.
pub struct EditorSession {
	model_home_session: ModelHomeSession,
	index: ModelIndex,
	card_registry: CardRegistry,
	svg_template: String,
}

impl PartialEq for EditorSession {
	fn eq(&self, other: &Self) -> bool {
		self.model_home() == other.model_home() && self.roots() == other.roots()
	}
}

impl Eq for EditorSession {}

impl EditorSession {
	/// Open the editor session without fully materializing every card.
	pub fn open(path: &Path) -> Result<Self, EditorSessionError> {
		let model_home_session = ModelHomeSession::try_open_for_update(path)?;
		let index = ModelIndex::open(&model_home_session.model_home)?;
		let card_registry = load_card_registry(&model_home_session.model_home)?;
		let svg_template = match load_svg_template(&model_home_session.model_home) {
			Ok(svg_template) => svg_template,
			Err(SvgTemplateDefsError::TemplateMissing(path)) => {
				warn!(template = %path, "aurora_editor graph view template is unavailable");
				String::new()
			}
			Err(error) => return Err(EditorSessionError::SvgTemplate(error)),
		};
		Ok(Self {
			model_home_session,
			index,
			card_registry,
			svg_template,
		})
	}

	/// Root mission cards available immediately after startup.
	pub fn roots(&self) -> &[ModelRootCard] {
		self.model_home_session.roots()
	}

	/// Resolved model-home directory for this session.
	pub fn model_home(&self) -> &Path {
		&self.model_home_session.model_home
	}

	/// Search the model home using the live in-memory index.
	pub fn search(&self, query: &str) -> Result<Vec<CardRef>, EditorSessionError> {
		Ok(self.index.search(query)?)
	}

	/// Root mission cards adapted for sidebar navigation.
	pub fn root_cards(&self) -> Result<Vec<CardRef>, EditorSessionError> {
		self.roots()
			.iter()
			.map(|root| {
				Ok(self
					.card_ref(root.id.as_str())?
					.unwrap_or_else(|| root_summary_ref(root)))
			})
			.collect()
	}

	/// Load a full card document on demand by its card ID.
	pub fn load_card(&self, card_id: &str) -> Result<Option<Card>, EditorSessionError> {
		let Some(relative_path) = self.index.resolve_card_path(card_id)? else {
			return Ok(None);
		};
		Ok(Some(
			self.model_home_session
				.load_card_by_relative_path(&relative_path)?,
		))
	}

	/// Resolve cards that link directly to the provided card ID.
	pub fn cards_linking_to(&self, card_id: &str) -> Result<Vec<CardRef>, EditorSessionError> {
		Ok(self.index.find_cards_linking_to(card_id)?)
	}

	/// Shortest discovered breadcrumb path from a mission root to the selected card.
	pub fn breadcrumb(&self, selected_card_id: &str) -> Result<Vec<CardRef>, EditorSessionError> {
		let Some(selected) = self.card_ref(selected_card_id)? else {
			return Ok(Vec::new());
		};

		let root_ids = self
			.roots()
			.iter()
			.map(|root| root.id.clone())
			.collect::<BTreeSet<_>>();
		if root_ids.contains(selected_card_id) {
			return Ok(vec![selected]);
		}

		let mut queue = VecDeque::from([(
			selected_card_id.to_string(),
			vec![selected_card_id.to_string()],
		)]);
		let mut visited = BTreeSet::from([selected_card_id.to_string()]);

		while let Some((current_id, path_from_selected)) = queue.pop_front() {
			for parent in self.cards_linking_to(current_id.as_str())? {
				if !visited.insert(parent.id.clone()) {
					continue;
				}

				let mut next_path = path_from_selected.clone();
				next_path.push(parent.id.clone());
				if root_ids.contains(parent.id.as_str()) {
					next_path.reverse();
					return self.card_refs_for_ids(next_path);
				}

				queue.push_back((parent.id, next_path));
			}
		}

		Ok(vec![selected])
	}

	/// Shared card registry loaded from the model home reference files.
	pub fn card_registry(&self) -> &CardRegistry {
		&self.card_registry
	}

	/// Shared SVG template used by the focused graph view.
	pub fn svg_template(&self) -> &str {
		self.svg_template.as_str()
	}

	fn card_ref(&self, card_id: &str) -> Result<Option<CardRef>, EditorSessionError> {
		Ok(self.load_card(card_id)?.map(card_to_ref))
	}

	fn card_refs_for_ids(&self, card_ids: Vec<String>) -> Result<Vec<CardRef>, EditorSessionError> {
		let mut cards = Vec::new();
		for card_id in card_ids {
			if let Some(card) = self.card_ref(card_id.as_str())? {
				cards.push(card);
			}
		}
		Ok(cards)
	}
}

fn card_to_ref(card: Card) -> CardRef {
	CardRef {
		id: card.id,
		card_type: card.card_type,
		card_subtype: card.card_subtype,
		name: card.name,
	}
}

fn root_summary_ref(root: &ModelRootCard) -> CardRef {
	CardRef {
		id: root.id.clone(),
		card_type: "Mission".to_string(),
		card_subtype: None,
		name: root.name.clone(),
	}
}

fn load_card_registry(model_home: &Path) -> Result<CardRegistry, EditorSessionError> {
	let reference_dir = model_home.join("reference");
	let model_configuration =
		std::fs::read_to_string(reference_dir.join("Aurora.modelconfiguration.json"))?;
	let view_configuration =
		std::fs::read_to_string(reference_dir.join("Aurora.viewconfiguration.json"))?;
	Ok(CardRegistry::try_new_from_configurations(
		&model_configuration,
		&view_configuration,
	)?)
}

/// Errors raised while opening or using an editor session.
#[derive(Debug, Error)]
pub enum EditorSessionError {
	#[error("Model-home session failed: {0}")]
	ModelHomeSession(#[from] ModelHomeSessionError),
	#[error("Reference file IO failed: {0}")]
	Io(#[from] std::io::Error),
	#[error("Search index failed: {0}")]
	ModelIndex(#[from] ModelIndexError),
	#[error("Card registry failed: {0}")]
	Registry(#[from] RegistryError),
	#[error("SVG template failed: {0}")]
	SvgTemplate(#[from] SvgTemplateDefsError),
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod session_tests;
