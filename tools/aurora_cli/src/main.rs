use aurora_cli::run;

fn main() {
	match run() {
		Ok(_) => {}
		Err(e) => {
			eprintln!("{}", e);
			std::process::exit(1);
		}
	}
}
