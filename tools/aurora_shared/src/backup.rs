//! Background backup support for Aurora model homes.

use std::fs::File;
use std::path::{Path, PathBuf};

use chrono::Utc;
use thiserror::Error;
use tokio::task::JoinHandle;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::{BackgroundRuntimeError, spawn_blocking_background};

/// Identifies which write-session surface is requesting a model-home backup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupSessionKind {
	/// Backup requested by the desktop editor.
	Editor,
	/// Backup requested by the MCP server.
	Mcp,
}

/// Parameters for creating a single timestamped backup ZIP for a model home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupRequest {
	pub model_home: PathBuf,
	pub mission_id: String,
	pub session_kind: BackupSessionKind,
}

impl BackupRequest {
	/// Create a backup request for a specific model home and mission identifier.
	pub fn new(
		model_home: PathBuf,
		mission_id: impl Into<String>,
		session_kind: BackupSessionKind,
	) -> Self {
		Self {
			model_home,
			mission_id: mission_id.into(),
			session_kind,
		}
	}

	fn archive_name(&self) -> String {
		let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
		match self.session_kind {
			BackupSessionKind::Editor => format!("{}-{timestamp}.zip", self.mission_id),
			BackupSessionKind::Mcp => format!("MCP-{}-{timestamp}.zip", self.mission_id),
		}
	}

	fn archive_path(&self) -> PathBuf {
		self.model_home.join("backups").join(self.archive_name())
	}
}

/// Creates timestamped ZIP archives of model homes for write sessions.
#[derive(Debug, Default)]
pub struct BackupManager;

impl BackupManager {
	/// Start creating a model-home backup on the shared background runtime.
	pub fn spawn(
		request: BackupRequest,
	) -> Result<JoinHandle<Result<PathBuf, BackupError>>, BackgroundRuntimeError> {
		spawn_blocking_background("model-home-backup", move || Self::create(&request))
	}

	/// Create a model-home backup synchronously.
	pub fn create(request: &BackupRequest) -> Result<PathBuf, BackupError> {
		let archive_path = request.archive_path();
		let backup_dir = archive_path
			.parent()
			.ok_or_else(|| BackupError::InvalidBackupPath(archive_path.display().to_string()))?;

		std::fs::create_dir_all(backup_dir)?;
		write_archive(&request.model_home, backup_dir, &archive_path)?;
		Ok(archive_path)
	}
}

fn write_archive(
	model_home: &Path,
	backup_dir: &Path,
	archive_path: &Path,
) -> Result<(), BackupError> {
	let archive_file = File::create(archive_path)?;
	let mut zip = ZipWriter::new(archive_file);
	let options = SimpleFileOptions::default()
		.compression_method(CompressionMethod::Deflated)
		.unix_permissions(0o644);

	for path in backup_files(model_home, backup_dir)? {
		let archive_name = archive_name_for(model_home, &path)?;
		zip.start_file(archive_name, options)?;
		let mut source = File::open(&path)?;
		std::io::copy(&mut source, &mut zip)?;
	}

	zip.finish()?;
	Ok(())
}

fn backup_files(model_home: &Path, backup_dir: &Path) -> Result<Vec<PathBuf>, BackupError> {
	let mut files = Vec::new();
	for entry in WalkDir::new(model_home)
		.into_iter()
		.filter_entry(|entry| !entry.path().starts_with(backup_dir))
	{
		let entry = entry.map_err(|error| BackupError::Io(error.into()))?;
		if entry.file_type().is_file() {
			files.push(entry.into_path());
		}
	}
	files.sort();
	Ok(files)
}

fn archive_name_for(model_home: &Path, path: &Path) -> Result<String, BackupError> {
	let relative = path
		.strip_prefix(model_home)
		.map_err(|_| BackupError::InvalidBackupPath(path.display().to_string()))?;
	Ok(relative.to_string_lossy().replace('\\', "/"))
}

/// Errors raised while preparing or writing a model-home backup archive.
#[derive(Debug, Error)]
pub enum BackupError {
	#[error("Backup I/O error: {0}")]
	Io(#[from] std::io::Error),
	#[error("Backup ZIP error: {0}")]
	Zip(#[from] zip::result::ZipError),
	#[error("Invalid backup path: {0}")]
	InvalidBackupPath(String),
}

#[cfg(test)]
mod tests {
	use std::io::Read;
	use std::path::Path;

	use zip::ZipArchive;

	use super::{BackupManager, BackupRequest, BackupSessionKind};
	use crate::block_on_background;

	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn create_writes_editor_named_archive_with_model_contents() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = temp.path().join("aurora");
		seed_model_home(&model_home)?;
		std::fs::write(model_home.join("backups").join("old.zip"), b"ignore me")?;

		let archive_path = BackupManager::create(&BackupRequest::new(
			model_home.clone(),
			"MIS-001",
			BackupSessionKind::Editor,
		))?;

		assert!(archive_path.exists());
		assert!(
			archive_path
				.file_name()
				.and_then(|name| name.to_str())
				.is_some_and(|name| {
					name.starts_with("MIS-001-")
						&& name.ends_with(".zip")
						&& !name.starts_with("MCP-")
				})
		);

		let entries = archive_entries(&archive_path)?;
		assert!(entries.contains(&"MIS-001-Alpha.json".to_string()));
		assert!(entries.contains(&"MIS-001/AuditLog.ndjson".to_string()));
		assert!(entries.contains(&"reference/Aurora.modelconfiguration.json".to_string()));
		assert!(!entries.iter().any(|entry| entry.starts_with("backups/")));
		Ok(())
	}

	#[test]
	fn spawn_writes_mcp_named_archive() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let model_home = temp.path().join("aurora");
		seed_model_home(&model_home)?;

		let handle = BackupManager::spawn(BackupRequest::new(
			model_home,
			"MIS-001",
			BackupSessionKind::Mcp,
		))?;
		let backup_result = block_on_background(handle)?;
		let archive_path = backup_result??;

		assert!(
			archive_path
				.file_name()
				.and_then(|name| name.to_str())
				.is_some_and(|name| name.starts_with("MCP-MIS-001-") && name.ends_with(".zip"))
		);
		Ok(())
	}

	fn seed_model_home(model_home: &Path) -> Result<()> {
		std::fs::create_dir_all(model_home.join("MIS-001"))?;
		std::fs::create_dir_all(model_home.join("reference"))?;
		std::fs::create_dir_all(model_home.join("schemas"))?;
		std::fs::create_dir_all(model_home.join("backups"))?;
		std::fs::write(model_home.join("MIS-001-Alpha.json"), b"{}")?;
		std::fs::write(model_home.join("MIS-001").join("AuditLog.ndjson"), b"{}\n")?;
		std::fs::write(
			model_home
				.join("reference")
				.join("Aurora.modelconfiguration.json"),
			b"{}",
		)?;
		std::fs::write(
			model_home.join("schemas").join("Aurora.card.schema.json"),
			b"{}",
		)?;
		Ok(())
	}

	fn archive_entries(path: &Path) -> Result<Vec<String>> {
		let file = std::fs::File::open(path)?;
		let mut archive = ZipArchive::new(file)?;
		let mut entries = Vec::new();
		for index in 0..archive.len() {
			let mut entry = archive.by_index(index)?;
			let mut contents = String::new();
			let _ = entry.read_to_string(&mut contents);
			entries.push(entry.name().to_string());
		}
		entries.sort();
		Ok(entries)
	}
}
