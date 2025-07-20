use std::time::Duration;

pub struct Thread {
	pub name: String,
	pub brand: String,
	pub cpu_usage: f32,
	pub frequency: u64
}

pub struct Processor {
	pub min1: f64,
	pub min5: f64,
	pub min15: f64,
	pub percent: f32,
	pub thread_count: u64,
	pub arch: String,
	pub threads: Vec<Thread>,
	pub refreshed: Duration,
}

impl Processor {

	pub fn new() -> Self{
		Processor {
			min1: 0.0,
			min5: 0.0,
			min15: 0.0,
			percent: 0.0,
			thread_count: 0,
			arch: String::new(),
			threads: Vec::new(),
			refreshed: Duration::from_secs(0),
		}
	}

}

impl Default for Processor {
	fn default() -> Self {
		Self::new()
	}
}