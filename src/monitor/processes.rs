use std::time::Duration;

pub struct Process {
	pub pid: u32,
	pub name: String,
	pub cpu: f32,
	pub memory: u64,
	pub virtual_memory: u64,
	pub refreshed: Duration,
}

impl Process {

	pub fn new() -> Self{
		Process {
			pid: 0,
			name: String::new(),
			cpu: 0.0,
			memory: 0,
			virtual_memory: 0,
			refreshed: Duration::from_secs(0),
		}
	}

}

impl Default for Process {
	fn default() -> Self {
		Self::new()
	}
}