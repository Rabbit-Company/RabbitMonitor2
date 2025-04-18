
pub struct SystemInfo {
	pub name: String,
	pub kernel_version: String,
	pub os_version: String,
	pub long_os_version: String,
	pub distribution_id: String,
	pub host_name: String,
	pub boot_time: u64,
}

impl SystemInfo {

	pub fn new() -> Self{
		SystemInfo {
			name: String::new(),
			kernel_version: String::new(),
			os_version: String::new(),
			long_os_version: String::new(),
			distribution_id: String::new(),
			host_name: String::new(),
			boot_time: 0,
		}
	}

}

impl Default for SystemInfo {
	fn default() -> Self {
		Self::new()
	}
}