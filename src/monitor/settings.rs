
pub struct EnergySettings {
	pub enabled: bool,
	pub interval: Option<u64>,
}

pub struct Settings {
	pub cache: u64,
	pub energy: EnergySettings,
	pub interfaces: Vec<String>,
	pub mounts: Vec<String>,
	pub components: Vec<String>,
	pub processes: Vec<String>,
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
			energy: EnergySettings { enabled: false, interval: None },
			interfaces: Vec::new(),
			mounts: Vec::new(),
			components: Vec::new(),
			processes: Vec::new(),
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