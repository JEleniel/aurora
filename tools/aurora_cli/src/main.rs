use log::error;

const EXIT_VALIDATION_FAILED: i32 = 2;
const EXIT_RENDER_FAILED: i32 = 3;
const EXIT_BUMP_FAILED: i32 = 4;
const EXIT_COMPACT_FAILED: i32 = 5;

fn main() {
	match aurora_cli::run() {
		Ok(result) => print!("{}", result),
		Err(err) => {
			error!("{err:?}");
			match err {
				aurora_cli::AuroraError::ModelError(_) => std::process::exit(EXIT_BUMP_FAILED),
				_ => std::process::exit(99),
			}
		}
	}
}
