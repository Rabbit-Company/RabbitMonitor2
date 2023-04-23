#[derive(Clone)]
pub struct Settings {
	pub cache: u64,
	pub logger: u8,
	pub interface: String
}

impl Settings {

	pub fn new() -> Self{
		Settings { cache: 3, logger: 1, interface: "eth0".to_string() }
	}

}

impl Default for Settings {
	fn default() -> Self {
		Self::new()
	}
}