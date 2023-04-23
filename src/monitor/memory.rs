#[derive(Clone)]
pub struct Memory {
	pub total: u64,
	pub available: u64,
	pub used: u64,
	pub free: u64,
	pub percent: f64
}

impl Memory {

	pub fn new() -> Self{
		Memory { 	total: 0, available: 0, used: 0, free: 0, percent: 0.0 }
	}

}

impl Default for Memory {
	fn default() -> Self {
		Self::new()
	}
}