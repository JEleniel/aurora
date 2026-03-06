use std::collections::VecDeque;

use thiserror::Error;

use super::{Model, ModelError};

const DEFAULT_HISTORY_CAPACITY: usize = 50;

/// A reversible edit captured as complete before/after model snapshots.
#[derive(Debug, Clone)]
pub struct EditCommand {
	before: Model,
	after: Model,
}

impl EditCommand {
	/// Capture a reversible edit from the model state before and after a mutation.
	pub fn new(before: &Model, after: &Model) -> Self {
		Self {
			before: before.clone(),
			after: after.clone(),
		}
	}

	fn before(&self) -> &Model {
		&self.before
	}

	fn after(&self) -> &Model {
		&self.after
	}
}

/// In-memory bounded undo/redo history for model edits.
#[derive(Debug, Clone)]
pub struct EditHistory {
	undo_stack: VecDeque<EditCommand>,
	redo_stack: Vec<EditCommand>,
	capacity: usize,
}

impl Default for EditHistory {
	fn default() -> Self {
		Self::new()
	}
}

impl EditHistory {
	pub const DEFAULT_CAPACITY: usize = DEFAULT_HISTORY_CAPACITY;

	/// Create a history stack with the default 50-entry depth.
	pub fn new() -> Self {
		Self {
			undo_stack: VecDeque::with_capacity(DEFAULT_HISTORY_CAPACITY),
			redo_stack: Vec::new(),
			capacity: DEFAULT_HISTORY_CAPACITY,
		}
	}

	pub fn push(&mut self, command: EditCommand) {
		if self.undo_stack.len() == self.capacity {
			self.undo_stack.pop_front();
		}
		self.undo_stack.push_back(command);
		self.redo_stack.clear();
	}

	pub fn undo(&mut self, model: &mut Model) -> Result<(), EditHistoryError> {
		let Some(command) = self.undo_stack.back().cloned() else {
			return Err(EditHistoryError::NothingToUndo);
		};

		apply_snapshot(model, command.before())?;
		if let Some(command) = self.undo_stack.pop_back() {
			self.redo_stack.push(command);
		}
		Ok(())
	}

	pub fn redo(&mut self, model: &mut Model) -> Result<(), EditHistoryError> {
		let Some(command) = self.redo_stack.last().cloned() else {
			return Err(EditHistoryError::NothingToRedo);
		};

		apply_snapshot(model, command.after())?;
		if let Some(command) = self.redo_stack.pop() {
			self.undo_stack.push_back(command);
		}
		Ok(())
	}

	pub fn can_undo(&self) -> bool {
		!self.undo_stack.is_empty()
	}

	pub fn can_redo(&self) -> bool {
		!self.redo_stack.is_empty()
	}

	#[cfg(test)]
	fn with_capacity(capacity: usize) -> Self {
		Self {
			undo_stack: VecDeque::with_capacity(capacity),
			redo_stack: Vec::new(),
			capacity,
		}
	}
}

fn apply_snapshot(model: &mut Model, snapshot: &Model) -> Result<(), ModelError> {
	let previous = model.clone();
	*model = snapshot.clone();

	if let Err(error) = model.write() {
		*model = previous;
		return Err(error);
	}

	Ok(())
}

#[derive(Debug, Error)]
pub enum EditHistoryError {
	#[error("No edits available to undo.")]
	NothingToUndo,
	#[error("No edits available to redo.")]
	NothingToRedo,
	#[error("Failed to apply model edit: {0}")]
	ModelError(#[from] ModelError),
}

#[cfg(test)]
#[path = "edit_history_tests.rs"]
mod edit_history_tests;
