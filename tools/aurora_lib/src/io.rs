use std::{
	fs,
	io::{Read, Write},
	path::Path,
};

use crate::{card::AuroraCard, error::AuroraLibError, validation::validate_card};

/// Deserialize a card from any reader and validate its invariants.
pub fn read_card_from_reader<R: Read>(mut reader: R) -> Result<AuroraCard, AuroraLibError> {
	let mut buffer = String::new();
	reader
		.read_to_string(&mut buffer)
		.map_err(|err| AuroraLibError::io(Path::new("<reader>").to_path_buf(), err))?;
	let card: AuroraCard = serde_json::from_str(&buffer)?;
	validate_card(&card)?;
	Ok(card)
}

/// Load and validate a card from the filesystem path provided.
pub fn read_card_from_path(path: impl AsRef<Path>) -> Result<AuroraCard, AuroraLibError> {
	let path_ref = path.as_ref();
	let data = fs::read_to_string(path_ref)
		.map_err(|err| AuroraLibError::io(path_ref.to_path_buf(), err))?;
	let card: AuroraCard = serde_json::from_str(&data)?;
	validate_card(&card)?;
	Ok(card)
}

/// Persist a validated card to the provided filesystem path.
pub fn write_card_to_path(path: impl AsRef<Path>, card: &AuroraCard) -> Result<(), AuroraLibError> {
	let path_ref = path.as_ref();
	if let Some(parent) = path_ref.parent() {
		fs::create_dir_all(parent).map_err(|err| AuroraLibError::io(parent.to_path_buf(), err))?;
	}
	validate_card(card)?;
	let data = serde_json::to_vec_pretty(card)?;
	let mut file = fs::File::create(path_ref)
		.map_err(|err| AuroraLibError::io(path_ref.to_path_buf(), err))?;
	file.write_all(&data)
		.map_err(|err| AuroraLibError::io(path_ref.to_path_buf(), err))?;
	file.write_all(b"\n")
		.map_err(|err| AuroraLibError::io(path_ref.to_path_buf(), err))?;
	Ok(())
}
