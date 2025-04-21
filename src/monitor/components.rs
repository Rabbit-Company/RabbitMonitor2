use std::time::Duration;

pub struct Component {
	pub label: String,
	pub temperature: Option<f32>,
	pub critical: Option<f32>,
	pub max: Option<f32>,
	pub refreshed: Duration,
}

impl Component {

	pub fn new() -> Self{
		Component {
			label: String::new(),
			temperature: Some(0.0),
			critical: Some(0.0),
			max: Some(0.0),
			refreshed: Duration::from_secs(0),
		}
	}

}

impl Default for Component {
	fn default() -> Self {
		Self::new()
	}
}