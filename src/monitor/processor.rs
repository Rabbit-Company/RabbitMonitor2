use std::time::Duration;

pub struct Processor {
	pub min1: f64,
	pub min5: f64,
	pub min15: f64,
	pub percent: f32,
	pub threads: u64,
	pub arch: String,
	pub refreshed: Duration,
}

impl Processor {

	pub fn new() -> Self{
		Processor {
			min1: 0.0,
			min5: 0.0,
			min15: 0.0,
			percent: 0.0,
			threads: 0,
			arch: String::new(),
			refreshed: Duration::from_secs(0),
		}
	}

}

impl Default for Processor {
	fn default() -> Self {
		Self::new()
	}
}