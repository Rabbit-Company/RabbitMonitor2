pub struct Network {
	pub download: f64,
	pub upload: f64
}

impl Network {

	pub fn new() -> Self{
		Network { download: 0.0, upload: 0.0 }
	}

}

impl Default for Network {
	fn default() -> Self {
		Self::new()
	}
}