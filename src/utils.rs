use std::sync::Mutex;
use std::sync::MutexGuard;
use std::thread::sleep;
use std::time::Instant;
use std::time::Duration;
use sysinfo::{NetworkExt, System, SystemExt, DiskExt, Disk, CpuExt};

struct Settings {
	cache: u64,
	logger: u8
}

struct Processor {
	threads: usize,
	min1: f64,
	min5: f64,
	min15: f64,
	percent: f32
}

struct Memory {
	total: u64,
	available: u64,
	used: u64,
	free: u64,
	percent: f64
}

pub struct Swap {
	total: u64,
	used: u64,
	free: u64,
	percent: f64
}

pub struct Storage {
	total: u64,
	used: u64,
	free: u64,
	percent: f64
}

pub struct Network {
	download: f64,
	upload: f64
}

static GLOBAL_SETTINGS: Mutex<Settings> = Mutex::new(Settings {
	cache: 5,
	logger: 1
});

static GLOBAL_CPU: Mutex<Processor> = Mutex::new(Processor {
	threads: 0,
	min1: 0.0,
	min5: 0.0,
	min15: 0.0,
	percent: 0.0
});

static GLOBAL_MEMORY: Mutex<Memory> = Mutex::new(Memory {
	total: 0,
	available: 0,
	used: 0,
	free: 0,
	percent: 0.0
});

static GLOBAL_SWAP: Mutex<Swap> = Mutex::new(Swap {
	total: 0,
	used: 0,
	free: 0,
	percent: 0.0
});

static GLOBAL_STORAGE: Mutex<Storage> = Mutex::new(Storage {
	total: 0,
	used: 0,
	free: 0,
	percent: 0.0
});

static GLOBAL_NETWORK: Mutex<Network> = Mutex::new(Network {
	download: 0.0,
	upload: 0.0
});

pub fn extract_cpu(sys: &System){
	let mut cpu: MutexGuard<Processor> = GLOBAL_CPU.lock().unwrap();

	cpu.min1 = sys.load_average().one;
	cpu.min5 = sys.load_average().five;
	cpu.min15 = sys.load_average().fifteen;

	let mut cpu_loads: Vec<f32> = Vec::new();
	for cpu in sys.cpus(){
		cpu_loads.push(cpu.cpu_usage());
	}
	let cpu_load_sum: f32 = cpu_loads.iter().sum();
	cpu.percent = cpu_load_sum / cpu_loads.len() as f32;

	drop(cpu);
}

pub fn extract_memory(sys: &System){
	let mut memory: MutexGuard<Memory> = GLOBAL_MEMORY.lock().unwrap();

	memory.total = sys.total_memory();
	memory.available = sys.available_memory();
	memory.used = sys.used_memory();
	memory.free = sys.free_memory();

	let memory_percent: f64 = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;
	memory.percent = if !f64::is_nan(memory_percent) { memory_percent } else { 0.0 };

	drop(memory);
}

pub fn extract_swap(sys: &System){
	let mut swap: MutexGuard<Swap> = GLOBAL_SWAP.lock().unwrap();

	swap.total = sys.total_swap();
	swap.used = sys.used_swap();
	swap.free = sys.free_swap();

	let swap_percent: f64 = (sys.used_swap() as f64 / sys.total_swap() as f64) * 100.0;
	swap.percent = if !f64::is_nan(swap_percent) { swap_percent } else { 0.0 };

	drop(swap);
}

pub fn extract_storage(sys: &System){
	let mut storage: MutexGuard<Storage> = GLOBAL_STORAGE.lock().unwrap();

	let mut available_storage: u64 = 0;
	let mut total_storage: u64 = 0;
	for disk in sys.disks() {
		if Disk::mount_point(disk).to_str().unwrap().eq("/") {
			available_storage += Disk::available_space(disk);
			total_storage += Disk::total_space(disk);
			break;
		}
	}
	storage.free = available_storage;
	storage.total = total_storage;
	let used_storage: u64 = total_storage - available_storage;
	storage.used = used_storage;

	let storage_percent: f64 = (used_storage as f64 / total_storage as f64) * 100.0;
	storage.percent = if !f64::is_nan(storage_percent) { storage_percent } else { 0.0 };

	drop(storage);
}

pub fn extract_network(sys: &System, interface: &String, monitoring_time: u64){
	let mut network: MutexGuard<Network> = GLOBAL_NETWORK.lock().unwrap();

	for (interface_name, data) in sys.networks(){
		if interface_name.eq(interface) {
			let mut millis: f64 = monitoring_time as f64 / 1000.0;
			if millis == 0.0 {
				millis = 1.0;
			}
			network.download = mega_bits(data.received() as f64 / millis);
			network.upload = mega_bits(data.transmitted() as f64 / millis);
		}
	}

	drop(network);
}

pub fn initialize(cache: u64, interface: String, logger: u8){
	let mut sys: System = System::new_all();
	GLOBAL_CPU.lock().unwrap().threads = sys.cpus().len();

	let mut settings: MutexGuard<Settings> = GLOBAL_SETTINGS.lock().unwrap();
	settings.cache = cache;
	settings.logger = logger;
	drop(settings);

	let mut last_refreshed: Instant = Instant::now();

	loop {
		sys.refresh_all();

		extract_cpu(&sys);
		extract_memory(&sys);
		extract_swap(&sys);
		extract_storage(&sys);
		extract_network(&sys, &interface, last_refreshed.elapsed().as_millis() as u64);

		last_refreshed = Instant::now();
		sleep(Duration::from_millis(cache * 1000));
	}
}

pub fn create_metric(mtype: &str, name: &str, description: &str, value: &str) -> String{
	"# HELP rabbit_".to_owned() + name + " " + description + "\n# TYPE rabbit_" + name + " " + mtype + "\nrabbit_" + name + " " + value + "\n"
}

pub fn create_metrics() -> String{
	let mut metrics: String = String::from("");

	let settings: MutexGuard<Settings> = GLOBAL_SETTINGS.lock().unwrap();
	let cpu: MutexGuard<Processor> = GLOBAL_CPU.lock().unwrap();
	let memory: MutexGuard<Memory> = GLOBAL_MEMORY.lock().unwrap();
	let swap: MutexGuard<Swap> = GLOBAL_SWAP.lock().unwrap();
	let storage: MutexGuard<Storage> = GLOBAL_STORAGE.lock().unwrap();
	let network: MutexGuard<Network> = GLOBAL_NETWORK.lock().unwrap();

	if settings.logger >= 1 {
		metrics += &create_metric("gauge", "cpu_load_1min", "CPU load recorded in last minute", &cpu.min1.to_string());
		metrics += &create_metric("gauge", "cpu_load_5min", "CPU load recorded in last 5 minutes", &cpu.min5.to_string());
		metrics += &create_metric("gauge", "cpu_load_15min", "CPU load recorded in last 15 minutes", &cpu.min15.to_string());

		metrics += &create_metric("gauge", "memory_total", "Total memory in bytes", &memory.total.to_string());
		metrics += &create_metric("gauge", "memory_available", "Available memory in bytes", &memory.available.to_string());
		metrics += &create_metric("gauge", "memory_used", "Used memory in bytes", &memory.used.to_string());
		metrics += &create_metric("gauge", "memory_free", "Free memory in bytes", &memory.free.to_string());

		metrics += &create_metric("gauge", "swap_total", "Total swap storage in bytes", &swap.total.to_string());
		metrics += &create_metric("gauge", "swap_used", "Used swap storage in bytes", &swap.used.to_string());
		metrics += &create_metric("gauge", "swap_free", "Free swap storage in bytes", &swap.free.to_string());

		metrics += &create_metric("gauge", "storage_total", "Total storage in bytes", &storage.total.to_string());
		metrics += &create_metric("gauge", "storage_used", "Used storage in bytes", &storage.used.to_string());
		metrics += &create_metric("gauge", "storage_free", "Free storage in bytes", &storage.free.to_string());
	}

  metrics += &create_metric("gauge", "cpu_load_percent", "CPU load in percent", &format!("{:.2}", cpu.percent));
	metrics += &create_metric("gauge", "memory_percent", "Used memory in percent", &format!("{:.2}", memory.percent));
  metrics += &create_metric("gauge", "swap_percent", "Used swap storage in percent", &format!("{:.2}", swap.percent));
  metrics += &create_metric("gauge", "storage_percent", "Used storage in percent", &format!("{:.2}", storage.percent));
	metrics += &create_metric("gauge", "network_download", "Download speed in bytes", &network.download.to_string());
	metrics += &create_metric("gauge", "network_upload", "Upload speed in bytes", &network.upload.to_string());

	drop(settings);
	drop(cpu);
	drop(memory);
	drop(swap);
	drop(storage);
	drop(network);

	metrics
}

pub fn mega_bits<T: Into<f64>>(bytes: T) -> f64{
	(bytes.into() / 1048576.0) * 8.0
}

pub fn main_page() -> String{

	let settings: MutexGuard<Settings> = GLOBAL_SETTINGS.lock().unwrap();
	let cpu: MutexGuard<Processor> = GLOBAL_CPU.lock().unwrap();
	let memory: MutexGuard<Memory> = GLOBAL_MEMORY.lock().unwrap();
	let swap: MutexGuard<Swap> = GLOBAL_SWAP.lock().unwrap();
	let storage: MutexGuard<Storage> = GLOBAL_STORAGE.lock().unwrap();
	let network: MutexGuard<Network> = GLOBAL_NETWORK.lock().unwrap();

	"<!DOCTYPE html>
	<html>
	<head>
		<title>Rabbit Monitor</title>
		<meta http-equiv='refresh' content='".to_owned() + &settings.cache.to_string() + "'>
	</head>
	<body>
		<style>
			td, th {
				border-bottom: 1px solid #000;
				border-right: 1px solid #000;
				text-align: center;
				padding: 8px;
			}
		</style>
		<h1>Rabbit Monitor</h1>
		<b>Version:</b> v4.0.0</br>
		<b>Fetch every:</b> " + &settings.cache.to_string() + " seconds</br></br>
		<table>
		<tr>
			<th>CPU Load</th>
			<td>" + &format!("{:.2}", cpu.percent) + "%</td>
		</tr>
		<tr>
			<th>RAM Usage</th>
			<td>" + &format!("{:.2}", memory.percent) + "%</td>
		</tr>
		<tr>
			<th>Swap Usage</th>
			<td>" + &format!("{:.2}", swap.percent) + "%</td>
		</tr>
		<tr>
			<th>Storage Usage</th>
			<td>" + &format!("{:.2}", storage.percent) + "%</td>
		</tr>
		<tr>
			<th>Download</th>
			<td>" + &format!("{:.2}", network.download) + " Mbps</td>
		</tr>
		<tr>
			<th>Upload</th>
			<td>" + &format!("{:.2}", network.upload) + " Mbps</td>
		</tr>
		</table>
		<body>
	</html>"
}