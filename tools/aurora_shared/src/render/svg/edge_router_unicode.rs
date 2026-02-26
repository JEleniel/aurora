//! Deterministic Unicode-aware ordering helpers for edge routing.
//!
//! The routing spec calls for NFC normalization plus Unicode case folding. We implement a
//! deterministic, best-effort approximation using NFC normalization and lowercase.

use std::cmp::Ordering;

use unicode_normalization::UnicodeNormalization;

fn unicode_key(value: &str) -> (String, Vec<u32>) {
	// Spec calls for NFC + Unicode case-fold. For now we do NFC + lowercase as a
	// deterministic, best-effort stand-in.
	let nfc: String = value.nfc().collect();
	let folded = nfc.to_lowercase();
	let cps: Vec<u32> = value.chars().map(|c| c as u32).collect();
	(folded, cps)
}

pub(super) fn unicode_cmp(left: &str, right: &str) -> Ordering {
	let (lk, lcps) = unicode_key(left);
	let (rk, rcps) = unicode_key(right);
	lk.cmp(&rk).then_with(|| lcps.cmp(&rcps))
}
