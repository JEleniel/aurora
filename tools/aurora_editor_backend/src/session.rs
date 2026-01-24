use aurora_shared::{AuroraModel, ModelHome, load_model};

use crate::errors::BackendResult;

/// Mutable session representing the currently opened model.
pub struct ModelSession {
	home: ModelHome,
	model: AuroraModel,
}

impl ModelSession {
	pub fn new(model: AuroraModel) -> Self {
		let home = model.home().clone();
		ModelSession { home, model }
	}

	pub fn home(&self) -> &ModelHome {
		&self.home
	}

	pub fn model(&self) -> &AuroraModel {
		&self.model
	}

	pub fn reload(&mut self) -> BackendResult<()> {
		self.model = load_model(&self.home)?;
		Ok(())
	}

	pub fn replace(&mut self, model: AuroraModel) {
		self.home = model.home().clone();
		self.model = model;
	}
}
