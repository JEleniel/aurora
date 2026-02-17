fn main() {
	match svg_prep::run() {
		Ok(_) => {}
		Err(e) => {
			eprintln!("{}", e);
			std::process::exit(1);
		}
	}
}
