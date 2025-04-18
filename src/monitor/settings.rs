pub struct Settings {
	pub cache: u64,
	pub interfaces: Vec<String>,
	pub disks: Vec<String>,
	pub all_metrics: bool,
	pub cpu_details: bool,
	pub memory_details: bool,
	pub swap_details: bool,
	pub storage_details: bool,
	pub network_details: bool,
}

impl Settings {

	pub fn new() -> Self{
		Settings {
			cache: 3,
			interfaces: Vec::new(),
			disks: Vec::new(),
			all_metrics: false,
			cpu_details: false,
			memory_details: false,
			swap_details: false,
			storage_details: false,
			network_details: false,
		}
	}

}

impl Default for Settings {
	fn default() -> Self {
		Self::new()
	}
}