pub struct Processor {
	pub min1: f64,
	pub min5: f64,
	pub min15: f64,
	pub percent: f32
}

impl Processor {

	pub fn new() -> Self{
		Processor { min1: 0.0, min5: 0.0, min15: 0.0, percent: 0.0 }
	}

}

impl Default for Processor {
	fn default() -> Self {
		Self::new()
	}
}