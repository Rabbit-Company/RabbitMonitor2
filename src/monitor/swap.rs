use std::time::Duration;

pub struct Swap {
	pub total: u64,
	pub used: u64,
	pub free: u64,
	pub percent: f64,
	pub refreshed: Duration,
}

impl Swap {

	pub fn new() -> Self{
		Swap {
			total: 0,
			used: 0,
			free: 0,
			percent: 0.0,
			refreshed: Duration::from_secs(0),
		}
	}

}

impl Default for Swap {
	fn default() -> Self {
		Self::new()
	}
}