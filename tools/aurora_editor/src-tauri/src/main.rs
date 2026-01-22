#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

fn main() {
	if let Err(err) = aurora_editor::run() {
		eprintln!("Aurora Editor failed: {err}");
	}
}
