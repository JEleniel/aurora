use log::debug;

const EXIT_FAILED: u8 = 1;

fn main() {
	match aurora_cli::run() {
		Ok(result) => print!("{}", result),
		Err(err) => {
			debug!("{err}");
			match err {
				aurora_cli::AuroraError::ModelError(_) => std::process::exit(EXIT_FAILED.into()),
				_ => std::process::exit(99),
			}
		}
	}
}
