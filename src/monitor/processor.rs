use std::fs;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct CpuStats {
	user: u64,
	nice: u64,
	system: u64,
	idle: u64,
	iowait: u64,
	irq: u64,
	softirq: u64,
	steal: u64,
}

impl CpuStats {
	fn total(&self) -> u64 {
		self.user + self.nice + self.system + self.idle +
		self.iowait + self.irq + self.softirq + self.steal
	}

	fn idle_time(&self) -> u64 {
		self.idle + self.iowait
	}
}

pub struct Thread {
	pub name: String,
	pub brand: String,
	pub cpu_usage: f32,
	pub frequency: u64
}

pub struct Processor {
	pub min1: f64,
	pub min5: f64,
	pub min15: f64,
	pub percent: f32,
	pub thread_count: u64,
	pub arch: String,
	pub threads: Vec<Thread>,
	pub refreshed: Duration,
	last_stats: Option<CpuStats>,
	last_measurement: Option<Instant>,
}

impl Processor {

	pub fn new() -> Self{
		Processor {
			min1: 0.0,
			min5: 0.0,
			min15: 0.0,
			percent: 0.0,
			thread_count: 0,
			arch: String::new(),
			threads: Vec::new(),
			refreshed: Duration::from_secs(0),
			last_stats: None,
      last_measurement: None,
		}
	}

	fn read_cpu_stats() -> Result<CpuStats, Box<dyn std::error::Error>> {
		let stat_content = fs::read_to_string("/proc/stat")?;
		let cpu_line = stat_content
			.lines()
			.find(|line| line.starts_with("cpu "))
			.ok_or("CPU line not found")?;

		let values: Vec<u64> = cpu_line
			.split_whitespace()
			.skip(1) // Skip "cpu" prefix
			.take(8) // Take the 8 CPU time values
			.map(|s| s.parse::<u64>())
			.collect::<Result<Vec<_>, _>>()?;

		if values.len() < 8 {
			return Err("Not enough CPU values".into());
		}

		Ok(CpuStats {
			user: values[0],
			nice: values[1],
			system: values[2],
			idle: values[3],
			iowait: values[4],
			irq: values[5],
			softirq: values[6],
			steal: values[7],
		})
	}

	pub fn calculate_cpu_usage(&mut self) -> f32 {
		match Self::read_cpu_stats() {
			Ok(current_stats) => {
				let now = Instant::now();

				// If we have previous stats, calculate usage
				if let (Some(prev_stats), Some(prev_time)) = (&self.last_stats, &self.last_measurement) {
					let time_diff = now.duration_since(*prev_time);

					// Only calculate if enough time has passed (at least 100ms)
					if time_diff >= Duration::from_millis(100) {
						let prev_total = prev_stats.total();
						let curr_total = current_stats.total();
						let prev_idle = prev_stats.idle_time();
						let curr_idle = current_stats.idle_time();

						let total_diff = curr_total.saturating_sub(prev_total);
						let idle_diff = curr_idle.saturating_sub(prev_idle);

						if total_diff > 0 {
							let usage = 100.0 * (1.0 - (idle_diff as f64 / total_diff as f64));
							self.percent = usage as f32;
						}
					}
				}

				self.last_stats = Some(current_stats);
				self.last_measurement = Some(now);

				self.percent.max(0.0).min(100.0)
			}
			Err(_) => self.percent, // Return last known value on error
		}
	}

}

impl Default for Processor {
	fn default() -> Self {
		Self::new()
	}
}