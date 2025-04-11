pub struct Storage {
	pub total: u64,
	pub used: u64,
	pub free: u64,
	pub percent: f64,
	pub total_read_bytes: u64,
	pub total_written_bytes: u64,
	pub read_speed: f64,
	pub write_speed: f64,
}

impl Storage {

	pub fn new() -> Self{
		Storage {
			total: 0,
			used: 0,
			free: 0,
			percent: 0.0,
			total_read_bytes: 0,
			total_written_bytes: 0,
			read_speed: 0.0,
			write_speed: 0.0,
		}
	}

}

impl Default for Storage {
	fn default() -> Self {
		Self::new()
	}
}