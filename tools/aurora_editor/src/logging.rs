//! Logging setup for the Aurora editor.

use std::io::{self, Write};

use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::MakeWriter;

use crate::error::EditorError;

/// Initialize tracing to emit INFO and below to stdout, ERROR to stderr.
pub fn init_logging() -> Result<(), EditorError> {
	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
	let writer = LevelWriter::default();
	if let Err(error) = tracing_subscriber::fmt()
		.with_env_filter(filter)
		.with_writer(writer)
		.with_target(false)
		.try_init()
	{
		return Err(EditorError::Logging(error.to_string()));
	}
	Ok(())
}

#[derive(Debug, Default)]
struct LevelWriter;

#[derive(Debug)]
struct LevelBasedWriter {
	level: Level,
}

impl<'a> MakeWriter<'a> for LevelWriter {
	type Writer = LevelBasedWriter;

	fn make_writer(&'a self) -> Self::Writer {
		LevelBasedWriter::new(Level::INFO)
	}

	fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
		LevelBasedWriter::new(*meta.level())
	}
}

impl LevelBasedWriter {
	fn new(level: Level) -> Self {
		Self { level }
	}

	fn is_error(&self) -> bool {
		self.level >= Level::ERROR
	}
}

impl Write for LevelBasedWriter {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		if self.is_error() {
			io::stderr().write(buf)
		} else {
			io::stdout().write(buf)
		}
	}

	fn flush(&mut self) -> io::Result<()> {
		if self.is_error() {
			io::stderr().flush()
		} else {
			io::stdout().flush()
		}
	}
}
