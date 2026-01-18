fn main() {
	match aurora_cli::run() {
		Ok(0) => {}
		Ok(code) => std::process::exit(code),
		Err(err) => {
			eprintln!("error: {err}");
			std::process::exit(1);
		}
	}
}
