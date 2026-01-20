use chrono::{SecondsFormat, Utc};
use fern::colors::{Color, ColoredLevelConfig};
use log::{Level, LevelFilter, SetLoggerError};
use thiserror::Error;

pub struct Logging {}

impl Logging {
	pub fn init(log_level: log::Level) -> Result<(), LoggingError> {
		let colors = ColoredLevelConfig::new()
			.trace(Color::BrightBlack)
			.debug(Color::BrightBlue)
			.info(Color::BrightGreen)
			.warn(Color::BrightYellow)
			.error(Color::BrightRed);

		let stdout_builder = fern::Dispatch::new()
			.format(move |out, message, record| {
				out.finish(format_args!(
					"[{} {}] ({}) {} at {} {}",
					Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
					colors.color(record.level()),
					record.target(),
					message,
					record.line().unwrap_or(0),
					record.file().unwrap_or("unknown"),
				))
			})
			.filter(|f| f.level() != Level::Error)
			.chain(std::io::stdout());

		let stdoerr_builder = fern::Dispatch::new()
			.format(move |out, message, record| {
				out.finish(format_args!(
					"[{} {}] ({}) {} at {} {}, {}",
					Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
					colors.color(record.level()),
					record.target(),
					message,
					record.line().unwrap_or(0),
					record.file().unwrap_or("unknown"),
					record.module_path().unwrap_or(""),
				))
			})
			.level(LevelFilter::Error)
			.chain(std::io::stderr());

		fern::Dispatch::new()
			.level(log_level.to_level_filter())
			.chain(stdout_builder)
			.chain(stdoerr_builder)
			.apply()?;

		Ok(())
	}
}

#[derive(Debug, Error)]
pub enum LoggingError {
	#[error("Failed to initialize logging: {0}")]
	InitializationError(#[from] SetLoggerError),
}
