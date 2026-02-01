//! Asset resolution helpers for the editor UI.

use base64::Engine;

use super::types::LogoAsset;

/// Builds the logo assets for the header and empty-state UI.
pub(crate) fn logo_asset() -> LogoAsset {
	let entries = [
		(
			16,
			data_url(include_bytes!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/../../assets/AURORA.16.png"
			))),
		),
		(
			32,
			data_url(include_bytes!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/../../assets/AURORA.32.png"
			))),
		),
		(
			64,
			data_url(include_bytes!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/../../assets/AURORA.64.png"
			))),
		),
		(
			180,
			data_url(include_bytes!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/../../assets/AURORA.180.png"
			))),
		),
		(
			192,
			data_url(include_bytes!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/../../assets/AURORA.192.png"
			))),
		),
		(
			512,
			data_url(include_bytes!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/../../assets/AURORA.512.png"
			))),
		),
		(
			1024,
			data_url(include_bytes!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/../../assets/AURORA.1024.png"
			))),
		),
	];
	let src = entries
		.first()
		.map(|(_, url)| url.clone())
		.unwrap_or_default();
	let srcset = entries
		.iter()
		.map(|(width, url)| format!("{url} {width}w"))
		.collect::<Vec<String>>()
		.join(", ");
	LogoAsset { src, srcset }
}

fn data_url(bytes: &[u8]) -> String {
	let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
	format!("data:image/png;base64,{encoded}")
}
