//! Best-effort secret redaction for file-based logs.

use std::io::{self, Write};
use std::sync::{LazyLock, RwLock};

static REGISTERED_SECRETS: LazyLock<RwLock<Vec<String>>> =
	LazyLock::new(|| RwLock::new(Vec::new()));

/// Register a secret value for best-effort log redaction.
pub fn register_secret(secret: &str) {
	if secret.trim().is_empty() {
		return;
	}

	let mut secrets = REGISTERED_SECRETS
		.write()
		.expect("registered secrets lock poisoned");
	if !secrets.iter().any(|candidate| candidate == secret) {
		secrets.push(secret.to_string());
	}
}

/// Writer wrapper that redacts previously registered secrets from log output.
pub struct RedactingWriter<W> {
	inner: W,
}

impl<W> RedactingWriter<W> {
	/// Wrap an existing writer with redaction support.
	pub fn new(inner: W) -> Self {
		Self { inner }
	}
}

impl<W> Write for RedactingWriter<W>
where
	W: Write,
{
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let rendered = String::from_utf8_lossy(buf);
		let redacted = redact(rendered.as_ref());
		self.inner.write_all(redacted.as_bytes())?;
		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner.flush()
	}
}

fn redact(input: &str) -> String {
	let secrets = REGISTERED_SECRETS
		.read()
		.expect("registered secrets lock poisoned");
	let mut output = input.to_string();
	for secret in secrets.iter() {
		output = output.replace(secret, "[REDACTED]");
	}
	output
}

#[cfg(test)]
mod tests {
	use std::io::Write;

	use super::{RedactingWriter, register_secret};

	#[test]
	fn registered_secret_is_redacted_from_output() {
		register_secret("super-secret-token");
		let mut sink = Vec::new();
		let mut writer = RedactingWriter::new(&mut sink);

		writer
			.write_all(b"Authorization: Bearer super-secret-token")
			.expect("expected write to succeed");

		let rendered = String::from_utf8(sink).expect("expected UTF-8 output");
		assert_eq!(rendered, "Authorization: Bearer [REDACTED]");
	}
}
