use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::errors::{BackendError, BackendResult};

pub fn write_atomic(path: &Path, contents: &str) -> BackendResult<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).map_err(|err| BackendError::io(parent, err))?;
	}
	let tmp_path = tmp_path(path);
	{
		let mut file = File::create(&tmp_path).map_err(|err| BackendError::io(&tmp_path, err))?;
		file.write_all(contents.as_bytes())
			.map_err(|err| BackendError::io(&tmp_path, err))?;
		file.sync_all()
			.map_err(|err| BackendError::io(&tmp_path, err))?;
	}
	fs::rename(&tmp_path, path).map_err(|err| BackendError::io(path, err))?;
	Ok(())
}

fn tmp_path(path: &Path) -> PathBuf {
	let mut name = path
		.file_name()
		.and_then(|n| n.to_str())
		.map(|n| format!(".{n}.tmp"))
		.unwrap_or_else(|| ".aurora.tmp".to_string());
	if name == path.file_name().and_then(|n| n.to_str()).unwrap_or("") {
		name.push_str(".tmp");
	}
	path.with_file_name(name)
}
