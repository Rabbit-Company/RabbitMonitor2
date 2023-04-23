pub struct Storage {
	pub total: u64,
	pub used: u64,
	pub free: u64,
	pub percent: f64
}

impl Storage {

	pub fn new() -> Self{
		Storage { total: 0, used: 0, free: 0, percent: 0.0 }
	}

}

impl Default for Storage {
	fn default() -> Self {
		Self::new()
	}
}