mod layout;
pub mod svg;

pub mod render_error;

pub use layout::*;

use std::path::PathBuf;

use crate::Aurora;
use render_error::RenderError;

/// Render all views for the provided Aurora models.
pub fn render(_aurora: &Aurora, _output_dir: &PathBuf) -> Result<(), RenderError> {
	Err(RenderError::RenderUnavailable)
}
