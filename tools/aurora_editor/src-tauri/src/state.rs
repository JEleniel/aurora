//! Application state for the Aurora Editor backend.

use std::path::PathBuf;
use std::sync::RwLock;

/// Workspace configuration for the editor backend.
#[derive(Debug, Clone)]
pub struct WorkspaceState {
	pub root: PathBuf,
	pub trusted: bool,
}

/// Shared editor backend state managed by Tauri.
#[derive(Debug)]
pub struct EditorState {
	workspace: RwLock<Option<WorkspaceState>>,
}

impl EditorState {
	/// Creates a new editor state with no workspace configured.
	pub fn new() -> Self {
		EditorState {
			workspace: RwLock::new(None),
		}
	}

	/// Replace the current workspace state.
	pub fn set_workspace(&self, workspace: WorkspaceState) -> Result<WorkspaceState, String> {
		let mut guard = self.workspace.write().map_err(|_| {
			"Workspace state lock was poisoned; restart the editor backend".to_string()
		})?;
		*guard = Some(workspace.clone());
		Ok(workspace)
	}

	/// Returns the configured workspace, if any.
	pub fn workspace(&self) -> Result<Option<WorkspaceState>, String> {
		let guard = self.workspace.read().map_err(|_| {
			"Workspace state lock was poisoned; restart the editor backend".to_string()
		})?;
		Ok(guard.clone())
	}
}
