#[cfg(not(test))]
fn main() -> Result<(), aurora_editor::RuntimeError> {
	aurora_editor::run()
}

#[cfg(test)]
fn main() {}
