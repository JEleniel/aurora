use aurora_cli::run;

fn main() {
	match run() {
		Ok(_) => {}
		Err(e) => {
			eprintln!("An unhandled error occurred: {:?}", e);
			std::process::exit(1);
		}
	}
}
