use std::{process::Command, time::Duration};

pub struct DCMI {
	pub power: Option<f64>,
	pub sampling_period_seconds: Option<u64>,
}

pub struct Energy {
	pub power_consumption: f64,
	pub refreshed: Duration,
	pub is_updating: bool,
}

impl Energy {

	pub fn new() -> Self{
		Energy {
			power_consumption: 0.0,
			refreshed: Duration::from_secs(0),
			is_updating: false,
		}
	}

	pub fn get_power_usage_w() -> Option<f64> {
    Self::get_dcmi_power_with_info()
			.and_then(|dcmi| dcmi.power)
			.or_else(Self::get_sensor_power)
	}

	pub fn get_dcmi_power_with_info() -> Option<DCMI> {
		let output = Command::new("ipmitool")
			.args(["dcmi", "power", "reading"])
			.output()
			.ok()?;

		if !output.status.success() {
			return None;
		}

		let stdout = String::from_utf8_lossy(&output.stdout);
		let mut power_value: Option<f64> = None;
		let mut sampling_period_seconds: Option<u64> = None;

		for line in stdout.lines() {
			// Look for instantaneous power reading
			if line.to_lowercase().contains("instantaneous power reading") {
				if let Some(value_part) = line.split(':').nth(1) {
					if let Some(watts_str) = value_part.trim().split_whitespace().next() {
						power_value = watts_str.parse::<f64>().ok();
					}
				}
			}

			// Look for sampling period
			if line.to_lowercase().contains("sampling period") {
				if let Some(value_part) = line.split(':').nth(1) {
					// Extract numeric value from "00000300 Seconds" format
					if let Some(period_str) = value_part.trim().split_whitespace().next() {
						if let Ok(period) = period_str.parse::<u64>() {
							sampling_period_seconds = Some(period);
						}
					}
				}
			}
		}

		Some(DCMI {
			power: power_value,
			sampling_period_seconds: sampling_period_seconds
		})
	}

	pub fn get_sensor_power() -> Option<f64> {
		let output = Command::new("ipmitool")
			.arg("sensor")
			.output()
			.ok()?;

		if !output.status.success() {
			return None;
		}

		let stdout = String::from_utf8_lossy(&output.stdout);
		let mut total_power = 0.0;
		let mut found_power = false;

		for line in stdout.lines() {
			if line.trim().is_empty() {
				continue;
			}

			// Format: "Sensor Name | Value | Units | Status"
			if line.to_lowercase().contains("watt") || line.to_lowercase().contains("power") {
				let parts: Vec<&str> = line.split('|').collect();

				// Ensure we have at least 3 parts (name, value, units)
				if parts.len() >= 3 {
					let value_str = parts[1].trim();
					let units = parts[2].trim().to_lowercase();

					// Check if units indicate watts
					if units.contains("watt") {
						if let Ok(value) = value_str.parse::<f64>() {
							// Some sensors might report 0 or negative values when inactive
							if value > 0.0 {
								// Check sensor name for common power sensor patterns
								let sensor_name = parts[0].trim().to_lowercase();

								// Priority for system-wide power sensors
								if sensor_name.contains("pwr consumption") ||
									sensor_name.contains("system power") ||
									sensor_name.contains("total power") ||
									sensor_name.contains("power meter") ||
									sensor_name.contains("power1") {
									return Some(value);
								}

								// Accumulate power readings from individual components
								if sensor_name.contains("cpu") ||
									sensor_name.contains("dimm") ||
									sensor_name.contains("psu") && !sensor_name.contains("out") {
									total_power += value;
									found_power = true;
								}
							}
						}
					}
				}
			}
		}

		// Return accumulated power if we found any readings
		if found_power && total_power > 0.0 {
			Some(total_power)
		} else {
			None
		}
	}

	pub fn refresh_dcmi(&mut self, now: Duration) {
		if let Some(power) = Self::get_dcmi_power_with_info().and_then(|dcmi| dcmi.power){
    	self.power_consumption = power;
    }

		self.refreshed = now;
	}

	pub fn refresh_sensor(&mut self, now: Duration) {
		if let Some(power) = Self::get_sensor_power(){
			self.power_consumption = power;
		}

		self.refreshed = now;
	}

}

impl Default for Energy {
	fn default() -> Self {
		Self::new()
	}
}