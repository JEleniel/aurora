use crate::errors::{BackendError, BackendResult};
use crate::session::ModelSession;
use parking_lot::RwLock;

/// Global application state shared across Tauri commands.
#[derive(Default)]
pub struct AppState {
	session: RwLock<Option<ModelSession>>,
}

impl AppState {
	/// Replace the current session with a new one.
	pub fn set_session(&self, session: ModelSession) {
		let mut guard = self.session.write();
		*guard = Some(session);
	}

	/// Clear any active model session.
	pub fn clear(&self) {
		let mut guard = self.session.write();
		*guard = None;
	}

	/// Execute a closure with an immutable reference to the active session.
	pub fn with_session<T>(
		&self,
		f: impl FnOnce(&ModelSession) -> BackendResult<T>,
	) -> BackendResult<T> {
		let guard = self.session.read();
		let session = guard.as_ref().ok_or(BackendError::NoSession)?;
		f(session)
	}

	/// Execute a closure with a mutable reference to the active session.
	pub fn with_session_mut<T>(
		&self,
		f: impl FnOnce(&mut ModelSession) -> BackendResult<T>,
	) -> BackendResult<T> {
		let mut guard = self.session.write();
		let session = guard.as_mut().ok_or(BackendError::NoSession)?;
		f(session)
	}
}
