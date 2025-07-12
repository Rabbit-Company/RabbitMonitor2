use std::time::Duration;

use battery::{units::{electric_potential::volt, energy::watt_hour, power::watt, ratio::percent, ElectricPotential, Energy, Power, Ratio, ThermodynamicTemperature, Time}, State, Technology};

pub struct Battery {
	pub state_of_charge: Ratio,
	pub energy: Energy,
	pub energy_full: Energy,
	pub energy_full_design: Energy,
	pub energy_rate: Power,
	pub voltage: ElectricPotential,
	pub state_of_health: Ratio,
	pub state: State,
	pub technology: Technology,
	pub temperature: Option<ThermodynamicTemperature>,
	pub cycle_count: Option<u32>,
	pub time_to_full: Option<Time>,
	pub time_to_empty: Option<Time>,
	pub vendor: Option<String>,
	pub model: Option<String>,
	pub serial_number: Option<String>,
	pub refreshed: Duration,
}

impl Battery {

	pub fn new() -> Self{
		Battery {
			state_of_charge: Ratio::new::<percent>(0.0),
			energy: Energy::new::<watt_hour>(0.0),
			energy_full: Energy::new::<watt_hour>(0.0),
			energy_full_design: Energy::new::<watt_hour>(0.0),
			energy_rate: Power::new::<watt>(0.0),
			voltage: ElectricPotential::new::<volt>(0.0),
			state_of_health: Ratio::new::<percent>(0.0),
			state: State::Unknown,
			technology: Technology::Unknown,
			temperature: None,
			cycle_count: None,
			time_to_full: None,
			time_to_empty: None,
			vendor: None,
			model: None,
			serial_number: None,
			refreshed: Duration::from_secs(0),
    }
	}

}

impl Default for Battery {
	fn default() -> Self {
		Self::new()
	}
}