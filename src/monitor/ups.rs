use std::{process::Command, time::Duration};

pub struct UPS {
	pub manufacturer: String,
	pub model: String,
	pub status: String,
	pub charge_percent: f32,
	pub load_percent: f32,
	pub runtime_seconds: u64,
	pub input_voltage: f32,
	pub output_voltage: f32,
	pub real_power_nominal: f32,
	pub power_usage: f32,
	pub refreshed: Duration,
}

impl UPS {

	pub fn new() -> Self{
		UPS {
			manufacturer: String::new(),
			model: String::new(),
			status: String::new(),
			charge_percent: 0.0,
			load_percent: 0.0,
			runtime_seconds: 0,
			input_voltage: 0.0,
			output_voltage: 0.0,
			real_power_nominal: 0.0,
			power_usage: 0.0,
			refreshed: Duration::from_secs(0),
		}
	}

	pub fn detect_ups() -> Option<Vec<String>> {
		// List all UPS devices managed by NUT
		let output = Command::new("upsc")
			.arg("-l")
			.output()
			.ok()?;

		if !output.status.success() {
			return None;
		}

		let stdout = String::from_utf8_lossy(&output.stdout);
		let devices: Vec<String> = stdout
			.lines()
			.filter(|line| !line.starts_with("Init SSL")) // Skip the warning
			.map(|s| s.trim().to_string())
			.filter(|s| !s.is_empty())
			.collect();

		if devices.is_empty() {
			None
		} else {
			Some(devices)
		}
	}

	pub fn get_ups_data(ups_name: &str, refreshed: Duration) -> Option<UPS> {
		let output = Command::new("upsc")
			.arg(ups_name)
			.output()
			.ok()?;

		if !output.status.success() {
			return None;
		}

		let stdout = String::from_utf8_lossy(&output.stdout);
		let mut info = UPS {
			manufacturer: String::new(),
			model: String::new(),
			status: String::new(),
			charge_percent: 0.0,
			load_percent: 0.0,
			runtime_seconds: 0,
			input_voltage: 0.0,
			output_voltage: 0.0,
			real_power_nominal: 0.0,
			power_usage: 0.0,
			refreshed: refreshed
		};

		for line in stdout.lines() {
			if line.starts_with("Init SSL") {
        continue; // Skip SSL warning
    	}

			let parts: Vec<&str> = line.split(':').collect();
			if parts.len() != 2 {
				continue;
			}

			let key = parts[0].trim();
			let value = parts[1].trim();

			match key {
				"device.mfr" => info.manufacturer = value.to_string(),
				"device.model" => info.model = value.to_string(),
				"ups.status" => info.status = value.to_string(),
				"battery.charge" => info.charge_percent = value.parse().unwrap_or(0.0),
				"ups.load" => info.load_percent = value.parse().unwrap_or(0.0),
				"battery.runtime" => info.runtime_seconds = value.parse().unwrap_or(0),
				"input.voltage" => info.input_voltage = value.parse().unwrap_or(0.0),
				"output.voltage" => info.output_voltage = value.parse().unwrap_or(0.0),
				"ups.realpower.nominal" => info.real_power_nominal = value.parse().unwrap_or(0.0),
				_ => {}
			}
		}

		if info.real_power_nominal > 0.0 {
			info.power_usage = (info.load_percent / 100.0) * info.real_power_nominal;
		}

		Some(info)
	}
}

impl Default for UPS {
	fn default() -> Self {
		Self::new()
	}
}