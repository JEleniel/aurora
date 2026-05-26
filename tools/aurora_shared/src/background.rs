//! Shared background runtime support for Aurora tooling.

use std::future::Future;
use std::sync::OnceLock;

use thiserror::Error;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;
use tracing::{debug, trace};

/// Spawn an async task onto the shared Aurora background runtime.
pub fn spawn_background<F>(
	task_name: &'static str,
	future: F,
) -> Result<JoinHandle<F::Output>, BackgroundRuntimeError>
where
	F: Future + Send + 'static,
	F::Output: Send + 'static,
{
	trace!(task = task_name, "Scheduling async background task");
	Ok(shared_runtime()?.spawn(future))
}

/// Spawn blocking work onto the shared Aurora background runtime.
pub fn spawn_blocking_background<F, R>(
	task_name: &'static str,
	operation: F,
) -> Result<JoinHandle<R>, BackgroundRuntimeError>
where
	F: FnOnce() -> R + Send + 'static,
	R: Send + 'static,
{
	debug!(task = task_name, "Scheduling blocking background task");
	Ok(shared_runtime()?.spawn_blocking(operation))
}

/// Run a future to completion on the shared Aurora background runtime.
pub fn block_on_background<F>(future: F) -> Result<F::Output, BackgroundRuntimeError>
where
	F: Future,
{
	Ok(shared_runtime()?.block_on(future))
}

fn shared_runtime() -> Result<&'static Runtime, BackgroundRuntimeError> {
	static RUNTIME: OnceLock<Result<&'static Runtime, String>> = OnceLock::new();
	match RUNTIME.get_or_init(|| {
		build_runtime()
			.map(|runtime| {
				let runtime: &'static Runtime = Box::leak(Box::new(runtime));
				runtime
			})
			.map_err(|error| error.to_string())
	}) {
		Ok(runtime) => Ok(runtime),
		Err(message) => Err(BackgroundRuntimeError::RuntimeInitialization(
			message.clone(),
		)),
	}
}

fn build_runtime() -> Result<Runtime, BackgroundRuntimeError> {
	let worker_threads = background_worker_threads();
	Ok(Builder::new_multi_thread()
		.thread_name("aurora-bg")
		.worker_threads(worker_threads)
		.enable_all()
		.build()
		.map_err(|error| BackgroundRuntimeError::RuntimeInitialization(error.to_string()))?)
}

fn background_worker_threads() -> usize {
	std::thread::available_parallelism()
		.map(|parallelism| parallelism.get().clamp(2, 4))
		.unwrap_or(2)
}

/// Errors raised while initializing or using the shared Aurora background runtime.
#[derive(Debug, Error)]
pub enum BackgroundRuntimeError {
	#[error("Failed to initialize Aurora background runtime: {0}")]
	RuntimeInitialization(String),
}

#[cfg(test)]
mod tests {
	use super::{block_on_background, spawn_background, spawn_blocking_background};

	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn block_on_background_runs_future() -> Result<()> {
		let value = block_on_background(async { 21 * 2 })?;
		assert_eq!(value, 42);
		Ok(())
	}

	#[test]
	fn spawn_background_runs_async_task() -> Result<()> {
		let handle = spawn_background("async-test", async { 20 + 22 })?;
		let value = block_on_background(handle)?;
		assert_eq!(value?, 42);
		Ok(())
	}

	#[test]
	fn spawn_blocking_background_runs_blocking_task() -> Result<()> {
		let handle = spawn_blocking_background("blocking-test", || 40 + 2)?;
		let value = block_on_background(handle)?;
		assert_eq!(value?, 42);
		Ok(())
	}
}
