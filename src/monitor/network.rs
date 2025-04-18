use std::time::Duration;

pub struct Network {
	pub download: f64,
	pub upload: f64,
	pub refreshed: Duration,
}

impl Network {

	pub fn new() -> Self{
		Network {
			download: 0.0,
			upload: 0.0,
			refreshed: Duration::from_secs(0),
		}
	}

}

impl Default for Network {
	fn default() -> Self {
		Self::new()
	}
}