use std::time::Instant;

use sysinfo::{CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, Networks, System};

use crate::utils::mega_bits;

use self::{processor::Processor, memory::Memory, swap::Swap, storage::Storage, network::Network, settings::Settings};

pub mod settings;
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
	pub processor: Processor,
	pub memory: Memory,
	pub swap: Swap,
	pub storage: Storage,
	pub network: Network,
	pub refreshed: Instant
}

impl Monitor{

	pub fn new() -> Self{
		let mut sys = System::new_all();
		sys.refresh_all();

		let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::nothing().with_storage());
		let networks = Networks::new_with_refreshed_list();

		Monitor {system: sys, disks, networks, settings: Settings::new(), processor: Processor::new(), memory: Memory::new(), swap: Swap::new(), storage: Storage::new(), network: Network::new(), refreshed: Instant::now() }
	}

	pub fn refresh(&mut self){
		self.cpu();
		self.memory();
		self.swap();
		self.storage();
		self.network();

		self.refreshed = Instant::now();
	}

	pub fn cpu(&mut self){
		let load_average = System::load_average();
		self.processor.min1 = load_average.one;
		self.processor.min5 = load_average.five;
		self.processor.min15 = load_average.fifteen;

		self.system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());
		self.processor.percent = self.system.global_cpu_usage();
	}

	pub fn memory(&mut self){
		self.system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
		self.memory.total = self.system.total_memory();
		self.memory.available = self.system.available_memory();
		self.memory.used = self.system.used_memory();
		self.memory.free = self.system.free_memory();

		let memory_percent: f64 = (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0;
		self.memory.percent = if !f64::is_nan(memory_percent) { memory_percent } else { 0.0 };
	}

	pub fn swap(&mut self){
		self.system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
		self.swap.total = self.system.total_swap();
		self.swap.used = self.system.used_swap();
		self.swap.free = self.system.free_swap();

		let swap_percent: f64 = (self.system.used_swap() as f64 / self.system.total_swap() as f64) * 100.0;
		self.swap.percent = if !f64::is_nan(swap_percent) { swap_percent } else { 0.0 };
	}

	pub fn storage(&mut self){
		let mut available_storage: u64 = 0;
		let mut total_storage: u64 = 0;

		self.disks.refresh_specifics(true, DiskRefreshKind::nothing().with_storage());
		for disk in self.disks.list() {
			if disk.mount_point().to_str().unwrap() == "/" {
				available_storage += disk.available_space();
				total_storage += disk.total_space();
				break;
			}
		}

		let used_storage: u64 = total_storage - available_storage;

		self.storage.free = available_storage;
		self.storage.total = total_storage;
		self.storage.used = used_storage;

		let storage_percent: f64 = (used_storage as f64 / total_storage as f64) * 100.0;
		self.storage.percent = if !f64::is_nan(storage_percent) { storage_percent }else{ 0.0 };
	}

	pub fn network(&mut self){
		let monitoring_time: u64 = self.refreshed.elapsed().as_millis() as u64;
		self.networks.refresh(true);

		let network = self.networks.get(&self.settings.interface);
		if let Some(network) = network {
			let mut millis: f64 = monitoring_time as f64 / 1000.0;
			if millis == 0.0 { millis = 1.0 }

			self.network.download = mega_bits(network.received() as f64 / millis);
			self.network.upload = mega_bits(network.transmitted() as f64 / millis);
		}
	}

}

impl Default for Monitor {
	fn default() -> Self {
		Self::new()
	}
}