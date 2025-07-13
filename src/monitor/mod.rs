use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use battery::Battery;
use chrono::Utc;
use components::Component;
use processes::Process;
use sysinfo::{Components, CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, Networks, Pid, ProcessRefreshKind, System};
use system_info::SystemInfo;
use crate::monitor::energy::Energy;
use crate::monitor::processor::Thread;
use crate::monitor::ups::UPS;
use crate::utils::mega_bits;
use self::{processor::Processor, memory::Memory, swap::Swap, storage::Storage, network::Network, settings::Settings};

pub mod settings;
pub mod system_info;
pub mod processor;
pub mod memory;
pub mod swap;
pub mod storage;
pub mod network;
pub mod components;
pub mod processes;
pub mod battery;
pub mod ups;
pub mod energy;

pub struct Monitor{
	pub system: System,
	pub disks: Disks,
	pub networks: Networks,
	pub components: Components,
	pub settings: Settings,
	pub system_info: SystemInfo,
	pub processor: Processor,
	pub memory: Memory,
	pub swap: Swap,
	pub energy: Arc<Mutex<Energy>>,
	pub upses: HashMap<String, UPS>,
	pub batteries: HashMap<String, Battery>,
	pub storage_devices: HashMap<String, Storage>,
	pub network_interfaces: HashMap<String, Network>,
	pub component_list: HashMap<String, Component>,
	pub process_list: HashMap<String, Process>,
	pub refreshed: Instant
}

impl Monitor{

	pub fn new() -> Self{
		let mut sys = System::new_all();
		sys.refresh_all();

		let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::nothing().with_storage().with_io_usage());
		let networks = Networks::new_with_refreshed_list();
		let components = Components::new_with_refreshed_list();

		let system_info = SystemInfo {
			name: System::name().unwrap_or("unknown".to_string()),
			kernel_version: System::kernel_version().unwrap_or("unknown".to_string()),
			os_version: System::os_version().unwrap_or("unknown".to_string()),
			long_os_version: System::long_os_version().unwrap_or("unknown".to_string()),
			distribution_id: System::distribution_id(),
			host_name: System::host_name().unwrap_or("unknown".to_string()),
			boot_time: System::boot_time(),
		};

		let mut processor = Processor::new();
		processor.arch = System::cpu_arch();
		processor.thread_count = sys.cpus().len() as u64;
		processor.calculate_cpu_usage();

		Monitor {system: sys, disks, networks, components, settings: Settings::new(), system_info, processor, memory: Memory::new(), swap: Swap::new(), energy: Arc::new(Mutex::new(Energy::new())), storage_devices: HashMap::new(), network_interfaces: HashMap::new(), component_list: HashMap::new(), process_list: HashMap::new(), upses: HashMap::new(), batteries: HashMap::new(), refreshed: Instant::now() }
	}

	pub fn refresh(&mut self){
		let now = Duration::from_millis(Utc::now().timestamp_millis() as u64);

		self.cpu(now);
		self.memory(now);
		self.swap(now);
		self.storage(now);
		self.network(now);
		self.componenet(now);
		self.processes(now);

		if self.settings.energy.enabled {
			self.energy_async(now);
		}

		self.refreshed = Instant::now();
	}

	pub fn cpu(&mut self, now: Duration){
		let load_average = System::load_average();
		self.processor.min1 = load_average.one;
		self.processor.min5 = load_average.five;
		self.processor.min15 = load_average.fifteen;

		self.system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage().with_frequency());

		/*
		let cpus = self.system.cpus();
    let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
    let average_usage = if cpus.is_empty() {
      0.0
    } else {
      total_usage / cpus.len() as f32
    };
    self.processor.percent = average_usage;

		let vmstat_cpu = get_vmstat_cpu_usage().unwrap_or(-1.0);

		println!(
      "CPU sysinfo (manual): {:.2}% | CPU sysinfo (global_cpu_usage): {:.2}% | CPU VMSTAT: {:.2}% | CPU /proc/stat: {:.2}%",
      self.system.global_cpu_usage(),
      average_usage,
      vmstat_cpu,
			self.processor.calculate_cpu_usage()
    );
		*/

		self.processor.calculate_cpu_usage();

		self.processor.threads = self.system.cpus().iter().map(|cpu| Thread {
    	name: cpu.name().into(),
    	brand: cpu.brand().into(),
    	cpu_usage: cpu.cpu_usage(),
    	frequency: cpu.frequency(),
		}).collect();

		self.processor.refreshed = now;
	}

	pub fn memory(&mut self, now: Duration){
		self.system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
		self.memory.total = self.system.total_memory();
		self.memory.available = self.system.available_memory();
		self.memory.used = self.system.used_memory();
		self.memory.free = self.system.free_memory();

		let memory_percent: f64 = (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0;
		self.memory.percent = if !f64::is_nan(memory_percent) { memory_percent } else { 0.0 };

		self.memory.refreshed = now;
	}

	pub fn swap(&mut self, now: Duration){
		self.system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
		self.swap.total = self.system.total_swap();
		self.swap.used = self.system.used_swap();
		self.swap.free = self.system.free_swap();

		let swap_percent: f64 = (self.system.used_swap() as f64 / self.system.total_swap() as f64) * 100.0;
		self.swap.percent = if !f64::is_nan(swap_percent) { swap_percent } else { 0.0 };

		self.swap.refreshed = now;
	}

	pub fn energy_async(&self, now: Duration) {
		let energy_clone = Arc::clone(&self.energy);
		let use_dcmi = self
			.settings
			.energy
			.interval
			.map_or(false, |interval| interval <= self.settings.cache);

		// Check if we're already updating
		{
			let energy_guard = energy_clone.lock().unwrap();
			if energy_guard.is_updating {
				return;
			}
		}

		if use_dcmi {
			// DCMI is fast, do it synchronously
			let mut energy_guard = energy_clone.lock().unwrap();
			energy_guard.refresh_dcmi(now);
		} else {
			// Sensor reading is slow, spawn background thread
			thread::spawn(move || {
				let mut energy_guard = energy_clone.lock().unwrap();
				energy_guard.is_updating = true;
				drop(energy_guard); // Release lock during slow operation

				if let Some(power) = Energy::get_sensor_power() {
					let mut energy_guard = energy_clone.lock().unwrap();
					energy_guard.power_consumption = power;
					energy_guard.refreshed = now;
					energy_guard.is_updating = false;
				} else {
					let mut energy_guard = energy_clone.lock().unwrap();
					energy_guard.is_updating = false;
				}
			});
		}
  }

	pub fn storage(&mut self, now: Duration){
		let monitoring_time: u64 = self.refreshed.elapsed().as_millis() as u64;
		self.disks.refresh_specifics(true, DiskRefreshKind::nothing().with_storage().with_io_usage());

		let mut millis: f64 = monitoring_time as f64 / 1000.0;
		if millis == 0.0 {
			millis = 1.0;
		}

		for disk in self.disks.list() {
			let mount = disk.mount_point().to_string_lossy().to_string();

			if !self.settings.mounts.is_empty() && !self.settings.mounts.contains(&mount) {
				continue; // Skip if not in the user-defined mount list
			}

			let name = disk.name().to_string_lossy().to_string();
			let total = disk.total_space();
			let free = disk.available_space();
			let used = total - free;
			let usage = disk.usage();

			let mut percent: f64 = (used as f64 / total as f64) * 100.0;
			percent = if !f64::is_nan(percent) { percent }else{ 0.0 };

			self.storage_devices.insert(name.clone(), Storage {
				name,
				mount_point: mount,
				total,
				used,
				free,
				percent,
				read_speed: usage.read_bytes as f64 / millis,
				write_speed: usage.written_bytes as f64 / millis,
				total_read_bytes: usage.total_read_bytes,
				total_written_bytes: usage.total_written_bytes,
				refreshed: now,
			});
		}
	}

	pub fn network(&mut self, now: Duration){
		let monitoring_time: u64 = self.refreshed.elapsed().as_millis() as u64;
		self.networks.refresh(true);

		let mut millis: f64 = monitoring_time as f64 / 1000.0;
		if millis == 0.0 {
			millis = 1.0;
		}

		for (iface, network) in self.networks.list() {

			if !self.settings.interfaces.is_empty() && !self.settings.interfaces.contains(iface) {
				continue; // Skip if not in the user-defined interface list
			}

			let download = mega_bits(network.received() as f64 / millis);
			let upload = mega_bits(network.transmitted() as f64 / millis);
			self.network_interfaces.insert(iface.clone(), Network {
				download,
				upload,
				total_errors_on_received: network.total_errors_on_received(),
				total_errors_on_transmitted: network.total_errors_on_transmitted(),
				total_packets_received: network.total_packets_received(),
				total_packets_transmitted: network.total_packets_transmitted(),
				refreshed: now
			});
		}
	}

	pub fn componenet(&mut self, now: Duration){
		self.components.refresh(true);

		for component in self.components.list() {
			let label = component.label().to_string();

			if !self.settings.components.is_empty() && self.settings.components.contains(&label){
				continue; // Skip if not in the user-defined components list
			}

			self.component_list.insert(component.label().to_string(), Component {
				label: component.label().to_string(),
				temperature: component.temperature(),
				critical: component.critical(),
				max: component.max(),
				refreshed: now
			});
		}
	}

	pub fn ups(&mut self, now: Duration){
		for ups_name in &self.settings.upses{
			let ups_data = UPS::get_ups_data(ups_name, now).unwrap_or(UPS::new());
			self.upses.insert(ups_name.to_string(), ups_data);
		}
	}

	pub fn processes(&mut self, now: Duration){
		let mut pids_to_refresh = Vec::new();

		for key in &self.settings.processes {
			if let Ok(pid_num) = key.parse::<u32>() {
				let pid = Pid::from_u32(pid_num);
				if self.system.process(pid).is_some() {
					pids_to_refresh.push(pid);
				}
			} else {
				for (pid, process) in self.system.processes() {
					if process.name().to_string_lossy() == key.as_str() {
						pids_to_refresh.push(*pid);
					}
				}
			}
		}

		self.system.refresh_processes_specifics(
			sysinfo::ProcessesToUpdate::Some(&pids_to_refresh),
			true,
			ProcessRefreshKind::nothing().with_cpu().with_memory(),
		);

		for pid in pids_to_refresh {
			if let Some(sys_proc) = self.system.process(pid) {
				let pid_str = pid.as_u32().to_string();

				let key = if self.process_list.contains_key(&pid_str) {
					pid_str
				} else {
					sys_proc.name().to_string_lossy().to_string()
				};

				self.process_list
					.entry(key.clone())
					.or_default()
					.pid = pid.as_u32();

				let proc_entry = self.process_list.get_mut(&key).unwrap();
				proc_entry.name = sys_proc.name().to_string_lossy().to_string();
				proc_entry.cpu = sys_proc.cpu_usage();
				proc_entry.memory = sys_proc.memory();
				proc_entry.virtual_memory = sys_proc.virtual_memory();
				proc_entry.refreshed = now;
			}
		}
	}

}

impl Default for Monitor {
	fn default() -> Self {
		Self::new()
	}
}

/*
fn get_vmstat_cpu_usage() -> Option<f32> {
	let output = Command::new("vmstat")
		.args(&["1", "2"])
		.output()
		.ok()?;

	let stdout = String::from_utf8_lossy(&output.stdout);
	let lines: Vec<&str> = stdout.lines().collect();

	// Last line should be the actual sampled data
	let data_line = lines.iter().rev().find(|line| line.trim().chars().next().map_or(false, |c| c.is_digit(10)))?;

	let columns: Vec<&str> = data_line.split_whitespace().collect();
	if columns.len() < 15 {
		return None;
	}

	// %idle is usually column 15, 0-based index 14
	let idle: f32 = columns[14].parse().ok()?;
	Some(100.0 - idle)
}
*/