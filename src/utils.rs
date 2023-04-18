use sysinfo::{NetworkExt, System, SystemExt, DiskExt, Disk};

pub struct Processor {
	threads: usize,
	min1: f64,
	min5: f64,
	min15: f64,
	percent: f64
}

pub struct Memory {
	total: u64,
	available: u64,
	used: u64,
	free: u64,
	percent: f64,
}

pub struct Swap {
	total: u64,
	used: u64,
	free: u64,
	percent: f64,
}

pub struct Storage {
	total: u64,
	used: u64,
	free: u64,
	percent: f64,
}

pub struct Network {
	download: u64,
	upload: u64
}

pub static mut CPU: Processor = Processor{
	threads: 0,
	min1: 0.0,
	min5: 0.0,
	min15: 0.0,
	percent: 0.0
};

pub static mut MEMORY: Memory = Memory{
	total: 0,
	available: 0,
	used: 0,
	free: 0,
	percent: 0.0,
};

pub static mut SWAP: Swap = Swap{
	total: 0,
	used: 0,
	free: 0,
	percent: 0.0,
};

pub static mut STORAGE: Storage = Storage{
	total: 0,
	used: 0,
	free: 0,
	percent: 0.0,
};

pub static mut NETWORK: Network = Network{
	download: 0,
	upload: 0
};

pub static mut CACHE: u64 = 0;

pub fn initialize(cache: u64, interface: String){
	let mut sys: System = System::new_all();

	unsafe{
		CACHE = cache;
		CPU.threads =	sys.cpus().len();
	}

	loop {
		sys.refresh_all();

		unsafe{
			CPU.min1 = sys.load_average().one;
			CPU.min5 = sys.load_average().five;
			CPU.min15 = sys.load_average().fifteen;
			CPU.percent = (sys.load_average().one / CPU.threads as f64) * 100.0;

			MEMORY.total = sys.total_memory();
			MEMORY.available = sys.available_memory();
			MEMORY.used = sys.used_memory();
			MEMORY.free = sys.free_memory();
			MEMORY.percent = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;

			SWAP.total = sys.total_swap();
			SWAP.used = sys.used_swap();
			SWAP.free = sys.free_swap();
			SWAP.percent = (sys.used_swap() as f64 / sys.total_swap() as f64) * 100.0;

			let mut available_storage: u64 = 0;
			let mut total_storage: u64 = 0;
			for disk in sys.disks() {
				if Disk::mount_point(disk).to_str().unwrap().eq("/") {
					available_storage += Disk::available_space(disk);
					total_storage += Disk::total_space(disk);
					break;
				}
			}
			STORAGE.free = available_storage;
			STORAGE.total = total_storage;

			let used_storage: u64 = total_storage - available_storage;
			STORAGE.used = used_storage;
			STORAGE.percent = (used_storage as f64 / total_storage as f64) * 100.0;

			for (interface_name, data) in sys.networks(){
				if interface_name.eq(&interface) {
					NETWORK.download = data.received();
					NETWORK.upload = data.transmitted();
				}
			}
		}

		std::thread::sleep(std::time::Duration::from_millis(cache * 1000));
	}
}

pub fn create_metric(mtype: &str, name: &str, description: &str, value: &str) -> String{
	return "# HELP rabbit_".to_owned() + name + " " + description + "\n# TYPE rabbit_" + name + " " + mtype + "\nrabbit_" + name + " " + value + "\n";
}

pub fn create_metrics() -> String{
	let mut metrics: String = String::from("");
	// CPU
	metrics += &create_metric("gauge", "cpu_load_1min", "CPU load recorded in last minute", unsafe { &CPU.min1.to_string() });
	metrics += &create_metric("gauge", "cpu_load_5min", "CPU load recorded in last 5 minutes", unsafe { &CPU.min5.to_string() });
  metrics += &create_metric("gauge", "cpu_load_15min", "CPU load recorded in last 15 minutes", unsafe { &CPU.min15.to_string() });
  metrics += &create_metric("gauge", "cpu_load_percent", "CPU load in percent", unsafe { &CPU.percent.to_string() });
  // Memory
  metrics += &create_metric("gauge", "memory_total", "Total memory in bytes", unsafe { &MEMORY.total.to_string() });
  metrics += &create_metric("gauge", "memory_available", "Available memory in bytes", unsafe { &MEMORY.available.to_string() });
  metrics += &create_metric("gauge", "memory_percent", "Used memory in percent", unsafe { &MEMORY.percent.to_string() });
  metrics += &create_metric("gauge", "memory_used", "Used memory in bytes", unsafe { &MEMORY.used.to_string() });
  metrics += &create_metric("gauge", "memory_free", "Free memory in bytes", unsafe { &MEMORY.free.to_string() });
  // Swap
  metrics += &create_metric("gauge", "swap_total", "Total swap storage in bytes", unsafe { &SWAP.total.to_string() });
  metrics += &create_metric("gauge", "swap_used", "Used swap storage in bytes", unsafe { &SWAP.used.to_string() });
  metrics += &create_metric("gauge", "swap_free", "Free swap storage in bytes", unsafe { &SWAP.free.to_string() });
  metrics += &create_metric("gauge", "swap_percent", "Used swap storage in percent", unsafe { &SWAP.percent.to_string() });
  // Storage
  metrics += &create_metric("gauge", "storage_total", "Total storage in bytes", unsafe { &STORAGE.total.to_string() });
  metrics += &create_metric("gauge", "storage_used", "Used storage in bytes", unsafe { &STORAGE.used.to_string() });
  metrics += &create_metric("gauge", "storage_free", "Free storage in bytes", unsafe { &STORAGE.free.to_string() });
  metrics += &create_metric("gauge", "storage_percent", "Used storage in percent", unsafe { &STORAGE.percent.to_string() });
	// Network
	metrics += &create_metric("gauge", "network_download", "Download speed in bytes", unsafe { &NETWORK.download.to_string() });
	metrics += &create_metric("gauge", "network_upload", "Upload speed in bytes", unsafe { &NETWORK.upload.to_string() });
	return metrics;
}

pub fn main_page() -> String{
 return "
 <style>
	td, th {
		border-bottom: 1px solid #000;
		border-right: 1px solid #000;
		text-align: center;
		padding: 8px;
	}
	</style>
	<h1>Rabbit Monitor</h1>
	<b>Version:</b> v3.0.0</br>
	<b>Fetch every:</b> ".to_owned() + unsafe { &CACHE.to_string() } + " seconds</br></br>
	<table>
	<tr>
		<th>CPU Load</th>
		<td>" + unsafe { &format!("{:.2}", CPU.percent) } + "%</td>
	</tr>
	<tr>
		<th>RAM Usage</th>
		<td>" + unsafe { &format!("{:.2}", MEMORY.percent) } + "%</td>
	</tr>
	<tr>
		<th>Swap Usage</th>
		<td>" + unsafe { &format!("{:.2}", SWAP.percent) } + "%</td>
	</tr>
	<tr>
		<th>Storage Usage</th>
		<td>" + unsafe { &format!("{:.2}", STORAGE.percent) } + "%</td>
	</tr>
	</table>
 ";
}