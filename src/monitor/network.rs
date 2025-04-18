use std::time::Duration;

pub struct Network {
	pub download: f64,
	pub upload: f64,
	pub total_errors_on_received: u64,
	pub total_errors_on_transmitted: u64,
	pub total_packets_received: u64,
	pub total_packets_transmitted: u64,
	pub refreshed: Duration,
}

impl Network {

	pub fn new() -> Self{
		Network {
			download: 0.0,
			upload: 0.0,
			total_errors_on_received: 0,
			total_errors_on_transmitted: 0,
			total_packets_received: 0,
			total_packets_transmitted: 0,
			refreshed: Duration::from_secs(0),
		}
	}

}

impl Default for Network {
	fn default() -> Self {
		Self::new()
	}
}