use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::Utc;
use sysinfo::{CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, Networks, System};
use system_info::SystemInfo;
use crate::utils::mega_bits;
use self::{processor::Processor, memory::Memory, swap::Swap, storage::Storage, network::Network, settings::Settings};

pub mod settings;
pub mod system_info;
pub mod processor;
pub mod memory;
pub mod swap;
pub mod storage;
pub mod network;

pub struct Monitor{
	pub system: System,
	pub disks: Disks,
	pub networks: Networks,
	pub settings: Settings,
	pub system_info: SystemInfo,
	pub processor: Processor,
	pub memory: Memory,
	pub swap: Swap,
	pub storage_devices: HashMap<String, Storage>,
	pub network_interfaces: HashMap<String, Network>,
	pub refreshed: Instant
}

impl Monitor{

	pub fn new() -> Self{
		let mut sys = System::new_all();
		sys.refresh_all();

		let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::nothing().with_storage().with_io_usage());
		let networks = Networks::new_with_refreshed_list();

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
		processor.threads = sys.cpus().len() as u64;

		Monitor {system: sys, disks, networks, settings: Settings::new(), system_info, processor, memory: Memory::new(), swap: Swap::new(), storage_devices: HashMap::new(), network_interfaces: HashMap::new(), refreshed: Instant::now() }
	}

	pub fn refresh(&mut self){
		let now = Duration::from_millis(Utc::now().timestamp_millis() as u64);

		self.cpu(now);
		self.memory(now);
		self.swap(now);
		self.storage(now);
		self.network(now);

		self.refreshed = Instant::now();
	}

	pub fn cpu(&mut self, now: Duration){
		let load_average = System::load_average();
		self.processor.min1 = load_average.one;
		self.processor.min5 = load_average.five;
		self.processor.min15 = load_average.fifteen;

		self.system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());
		self.processor.percent = self.system.global_cpu_usage();

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

	pub fn storage(&mut self, now: Duration){
		let monitoring_time: u64 = self.refreshed.elapsed().as_millis() as u64;
		self.disks.refresh_specifics(true, DiskRefreshKind::nothing().with_storage().with_io_usage());

		let mut millis: f64 = monitoring_time as f64 / 1000.0;
		if millis == 0.0 {
			millis = 1.0;
		}

		for disk in self.disks.list() {
			let name = disk.name().to_string_lossy().to_string();
			let mount = disk.mount_point().to_string_lossy().to_string();
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

		for iface in &self.settings.interfaces {
			if let Some(network) = self.networks.get(iface) {
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
	}

}

impl Default for Monitor {
	fn default() -> Self {
		Self::new()
	}
}