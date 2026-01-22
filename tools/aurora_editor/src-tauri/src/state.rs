use std::{
	path::PathBuf,
	sync::{Mutex, MutexGuard},
	thread,
};

use aurora_lib::AuroraLibError;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::model_home::{self, LoadedModel};

pub(crate) const MODEL_CHANGED_EVENT: &str = "model-home-changed";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelChangedPayload {
	pub root: String,
	pub summary: model_home::ModelHomeSummary,
}

pub(crate) struct AppState {
	loaded_model: Mutex<Option<LoadedModel>>,
	watcher: Mutex<Option<ModelWatcher>>,
}

impl Default for AppState {
	fn default() -> Self {
		Self {
			loaded_model: Mutex::new(None),
			watcher: Mutex::new(None),
		}
	}
}

impl AppState {
	pub(crate) fn store_model(&self, model: LoadedModel) -> Result<(), String> {
		let mut guard = self.lock_model()?;
		*guard = Some(model);
		Ok(())
	}

	pub(crate) fn watch_model_home(
		&self,
		app_handle: &AppHandle,
		root: PathBuf,
	) -> Result<(), String> {
		let mut guard = self.lock_watcher()?;
		*guard = None;
		let watcher = ModelWatcher::start(app_handle.clone(), root)?;
		*guard = Some(watcher);
		Ok(())
	}

	pub(crate) fn with_model_view<R, F>(&self, f: F) -> Result<R, String>
	where
		F: FnOnce(&LoadedModel) -> R,
	{
		let guard = self.lock_model()?;
		let Some(model) = guard.as_ref() else {
			return Err(Self::missing_model_error());
		};
		Ok(f(model))
	}

	pub(crate) fn with_model<R, F>(&self, f: F) -> Result<R, String>
	where
		F: FnOnce(&LoadedModel) -> Result<R, AuroraLibError>,
	{
		let guard = self.lock_model()?;
		let Some(model) = guard.as_ref() else {
			return Err(Self::missing_model_error());
		};
		f(model).map_err(|err| err.to_string())
	}

	fn lock_model(&self) -> Result<MutexGuard<'_, Option<LoadedModel>>, String> {
		self.loaded_model.lock().map_err(|_| Self::lock_error())
	}

	fn lock_watcher(&self) -> Result<MutexGuard<'_, Option<ModelWatcher>>, String> {
		self.watcher.lock().map_err(|_| Self::lock_error())
	}

	fn lock_error() -> String {
		"Application state lock is unavailable; restart Aurora Editor.".into()
	}

	fn missing_model_error() -> String {
		"Load a model home before using this command.".into()
	}
}

struct ModelWatcher {
	_watcher: RecommendedWatcher,
}

impl ModelWatcher {
	fn start(app_handle: AppHandle, root: PathBuf) -> Result<Self, String> {
		let watch_path = root.clone();
		let mut watcher = RecommendedWatcher::new(
			move |res| handle_fs_event(app_handle.clone(), root.clone(), res),
			Config::default(),
		)
		.map_err(|err| format!("failed to initialize filesystem watcher: {err}"))?;
		watcher
			.watch(&watch_path, RecursiveMode::Recursive)
			.map_err(|err| format!("failed to watch model home {}: {err}", watch_path.display()))?;
		Ok(Self { _watcher: watcher })
	}
}

fn handle_fs_event(app_handle: AppHandle, root: PathBuf, result: notify::Result<Event>) {
	match result {
		Ok(event) if should_reload(&event) => {
			thread::spawn(move || {
				if let Err(err) = reload_model_home(app_handle, root) {
					eprintln!("[aurora_editor] watcher reload failed: {err}");
				}
			});
		}
		Ok(_) => {}
		Err(err) => {
			eprintln!("[aurora_editor] watcher error: {err}");
		}
	}
}

fn reload_model_home(app_handle: AppHandle, root: PathBuf) -> Result<(), String> {
	let model =
		model_home::load_model_home(&root).map_err(|err: AuroraLibError| err.to_string())?;
	let summary = model.summary().clone();
	let state = app_handle.state::<AppState>();
	state.store_model(model)?;
	let payload = ModelChangedPayload {
		root: summary.root.clone(),
		summary,
	};
	app_handle
		.emit(MODEL_CHANGED_EVENT, payload)
		.map_err(|err| format!("failed to emit model change event: {err}"))?;
	Ok(())
}

fn should_reload(event: &Event) -> bool {
	matches!(
		event.kind,
		EventKind::Any | EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
	)
}
