use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::utils::mega_bits;

struct NetDevCounters {
	rx_bytes: u64,
	tx_bytes: u64,
	rx_packets: u64,
	tx_packets: u64,
	rx_errors: u64,
	tx_errors: u64,
}

pub struct DockerContainer {
	pub name: String,
	pub pid: u32,
	pub cpu_percent: f64,
	pub memory_usage: u64,
	pub memory_limit: u64,
	pub memory_percent: f64,
	pub download: f64,
	pub upload: f64,
	pub net_rx_bytes: u64,
	pub net_tx_bytes: u64,
	pub total_packets_received: u64,
	pub total_packets_transmitted: u64,
	pub total_errors_on_received: u64,
	pub total_errors_on_transmitted: u64,
	pub block_read_bytes: u64,
	pub block_write_bytes: u64,
	pub pids: u64,
	pub refreshed: Duration,
}

impl DockerContainer {
	pub fn new() -> Self {
		DockerContainer {
			name: String::new(),
			pid: 0,
			cpu_percent: 0.0,
			memory_usage: 0,
			memory_limit: 0,
			memory_percent: 0.0,
			download: 0.0,
			upload: 0.0,
			net_rx_bytes: 0,
			net_tx_bytes: 0,
			total_packets_received: 0,
			total_packets_transmitted: 0,
			total_errors_on_received: 0,
			total_errors_on_transmitted: 0,
			block_read_bytes: 0,
			block_write_bytes: 0,
			pids: 0,
			refreshed: Duration::from_secs(0),
		}
	}
}

impl Default for DockerContainer {
	fn default() -> Self {
		Self::new()
	}
}

struct PrevNetState {
	rx_bytes: u64,
	tx_bytes: u64,
	timestamp: Duration,
}

pub struct DockerMonitor {
	pub containers: Arc<Mutex<HashMap<String, DockerContainer>>>,
	prev_net: HashMap<String, PrevNetState>,
	child: Option<Child>,
}

impl DockerMonitor {
	pub fn new() -> Self {
		DockerMonitor {
			containers: Arc::new(Mutex::new(HashMap::new())),
			prev_net: HashMap::new(),
			child: None,
		}
	}

	pub fn start(&mut self, filter: Vec<String>) {
		let containers = Arc::clone(&self.containers);

		let mut cmd = Command::new("docker");
		cmd
			.arg("stats")
			.arg("--format")
			.arg("{{json .}}")
			.stdout(Stdio::piped())
			.stderr(Stdio::null());

		if !filter.is_empty() {
			for name in &filter {
				cmd.arg(name);
			}
		} else {
			cmd.arg("--all");
		}

		match cmd.spawn() {
			Ok(mut child) => {
				let stdout = child
					.stdout
					.take()
					.expect("Failed to capture docker stats stdout");
				self.child = Some(child);

				thread::spawn(move || {
					let reader = BufReader::new(stdout);
					for line in reader.lines() {
						let line = match line {
							Ok(l) => l,
							Err(_) => break,
						};

						if line.trim().is_empty() {
							continue;
						}

						let json_str = match extract_json(&line) {
							Some(s) => s,
							None => continue,
						};

						if let Some(container) = parse_docker_stats_line(json_str) {
							let name = container.name.clone();
							let mut map = containers.lock().unwrap();
							map.insert(name, container);
						}
					}
				});
			}
			Err(e) => {
				eprintln!("Failed to start docker stats: {}", e);
			}
		}
	}

	pub fn stop(&mut self) {
		if let Some(ref mut child) = self.child {
			let _ = child.kill();
			let _ = child.wait();
		}
		self.child = None;
	}

	pub fn is_docker_available() -> bool {
		Command::new("docker")
			.arg("info")
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.status()
			.map(|s| s.success())
			.unwrap_or(false)
	}

	pub fn list_containers() -> Vec<String> {
		let output = Command::new("docker")
			.args(["ps", "--format", "{{.Names}}"])
			.output();

		match output {
			Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
				.lines()
				.map(|s| s.trim().to_string())
				.filter(|s| !s.is_empty())
				.collect(),
			_ => Vec::new(),
		}
	}

	pub fn snapshot(&mut self, now: Duration) -> HashMap<String, DockerContainer> {
		let mut map = self.containers.lock().unwrap();
		let mut result = HashMap::new();

		let names_needing_pid: Vec<String> = map
			.iter()
			.filter(|(_, c)| c.pid == 0)
			.map(|(name, _)| name.clone())
			.collect();

		if !names_needing_pid.is_empty() {
			let pids = lookup_container_pids(&names_needing_pid);
			for (name, pid) in pids {
				if let Some(c) = map.get_mut(&name) {
					c.pid = pid;
				}
			}
		}

		for (name, c) in map.iter() {
			let net_counters = if c.pid > 0 { read_net_dev(c.pid) } else { None };

			let (net_rx, net_tx, packets_rx, packets_tx, errors_rx, errors_tx) = match &net_counters {
				Some(nc) => (
					nc.rx_bytes,
					nc.tx_bytes,
					nc.rx_packets,
					nc.tx_packets,
					nc.rx_errors,
					nc.tx_errors,
				),
				None => (c.net_rx_bytes, c.net_tx_bytes, 0, 0, 0, 0),
			};

			let (download, upload) = if let Some(prev) = self.prev_net.get(name) {
				let elapsed = now.as_secs_f64() - prev.timestamp.as_secs_f64();
				if elapsed > 0.0 {
					let rx_delta = net_rx.saturating_sub(prev.rx_bytes) as f64;
					let tx_delta = net_tx.saturating_sub(prev.tx_bytes) as f64;
					(mega_bits(rx_delta / elapsed), mega_bits(tx_delta / elapsed))
				} else {
					(0.0, 0.0)
				}
			} else {
				(0.0, 0.0)
			};

			self.prev_net.insert(
				name.clone(),
				PrevNetState {
					rx_bytes: net_rx,
					tx_bytes: net_tx,
					timestamp: now,
				},
			);

			result.insert(
				name.clone(),
				DockerContainer {
					name: c.name.clone(),
					pid: c.pid,
					cpu_percent: c.cpu_percent,
					memory_usage: c.memory_usage,
					memory_limit: c.memory_limit,
					memory_percent: c.memory_percent,
					download,
					upload,
					net_rx_bytes: net_rx,
					net_tx_bytes: net_tx,
					total_packets_received: packets_rx,
					total_packets_transmitted: packets_tx,
					total_errors_on_received: errors_rx,
					total_errors_on_transmitted: errors_tx,
					block_read_bytes: c.block_read_bytes,
					block_write_bytes: c.block_write_bytes,
					pids: c.pids,
					refreshed: now,
				},
			);
		}
		result
	}
}

impl Drop for DockerMonitor {
	fn drop(&mut self) {
		self.stop();
	}
}

fn lookup_container_pids(names: &[String]) -> HashMap<String, u32> {
	let mut result = HashMap::new();

	for name in names {
		let output = Command::new("docker")
			.args(["inspect", "--format", "{{.State.Pid}}", name])
			.output();

		if let Ok(out) = output {
			if out.status.success() {
				let pid_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
				if let Ok(pid) = pid_str.parse::<u32>() {
					if pid > 0 {
						result.insert(name.clone(), pid);
					}
				}
			}
		}
	}

	result
}

fn read_net_dev(pid: u32) -> Option<NetDevCounters> {
	let path = format!("/proc/{}/net/dev", pid);
	let content = fs::read_to_string(&path).ok()?;

	let mut counters = NetDevCounters {
		rx_bytes: 0,
		tx_bytes: 0,
		rx_packets: 0,
		tx_packets: 0,
		rx_errors: 0,
		tx_errors: 0,
	};

	for line in content.lines() {
		let line = line.trim();

		let Some((iface, rest)) = line.split_once(':') else {
			continue;
		};

		let iface = iface.trim();

		if iface == "lo" {
			continue;
		}

		let fields: Vec<&str> = rest.split_whitespace().collect();
		if fields.len() < 11 {
			continue;
		}

		// Receive fields:  0=bytes 1=packets 2=errs 3=drop 4=fifo 5=frame 6=compressed 7=multicast
		// Transmit fields: 8=bytes 9=packets 10=errs 11=drop 12=fifo 13=colls 14=carrier 15=compressed
		if let Ok(v) = fields[0].parse::<u64>() {
			counters.rx_bytes += v;
		}
		if let Ok(v) = fields[1].parse::<u64>() {
			counters.rx_packets += v;
		}
		if let Ok(v) = fields[2].parse::<u64>() {
			counters.rx_errors += v;
		}
		if let Ok(v) = fields[8].parse::<u64>() {
			counters.tx_bytes += v;
		}
		if let Ok(v) = fields[9].parse::<u64>() {
			counters.tx_packets += v;
		}
		if let Ok(v) = fields[10].parse::<u64>() {
			counters.tx_errors += v;
		}
	}

	Some(counters)
}

fn extract_json(line: &str) -> Option<&str> {
	let start = line.find('{')?;
	let end = line.rfind('}')?;
	if end >= start {
		Some(&line[start..=end])
	} else {
		None
	}
}

fn parse_docker_stats_line(line: &str) -> Option<DockerContainer> {
	let parsed: serde_json::Value = serde_json::from_str(line).ok()?;

	let name = parsed.get("Name")?.as_str()?.to_string();

	let cpu_percent = parse_percent(parsed.get("CPUPerc")?.as_str()?);
	let mem_percent = parse_percent(parsed.get("MemPerc")?.as_str()?);

	let mem_usage_str = parsed.get("MemUsage")?.as_str()?;
	let (memory_usage, memory_limit) = parse_mem_usage(mem_usage_str);

	let net_io_str = parsed.get("NetIO")?.as_str()?;
	let (net_rx_bytes, net_tx_bytes) = parse_io_pair(net_io_str);

	let block_io_str = parsed.get("BlockIO")?.as_str()?;
	let (block_read_bytes, block_write_bytes) = parse_io_pair(block_io_str);

	let pids = parsed
		.get("PIDs")?
		.as_str()?
		.trim()
		.parse::<u64>()
		.unwrap_or(0);

	Some(DockerContainer {
		name,
		pid: 0,
		cpu_percent,
		memory_usage,
		memory_limit,
		memory_percent: mem_percent,
		download: 0.0,
		upload: 0.0,
		net_rx_bytes,
		net_tx_bytes,
		total_packets_received: 0,
		total_packets_transmitted: 0,
		total_errors_on_received: 0,
		total_errors_on_transmitted: 0,
		block_read_bytes,
		block_write_bytes,
		pids,
		refreshed: Duration::from_secs(0),
	})
}

fn parse_percent(s: &str) -> f64 {
	s.trim().trim_end_matches('%').parse::<f64>().unwrap_or(0.0)
}

fn parse_mem_usage(s: &str) -> (u64, u64) {
	let parts: Vec<&str> = s.split('/').collect();
	if parts.len() == 2 {
		(parse_size(parts[0].trim()), parse_size(parts[1].trim()))
	} else {
		(0, 0)
	}
}

fn parse_io_pair(s: &str) -> (u64, u64) {
	let parts: Vec<&str> = s.split('/').collect();
	if parts.len() == 2 {
		(parse_size(parts[0].trim()), parse_size(parts[1].trim()))
	} else {
		(0, 0)
	}
}

fn parse_size(s: &str) -> u64 {
	let s = s.trim();

	if s == "0B" {
		return 0;
	}

	let units: &[(&str, f64)] = &[
		("TiB", 1024.0 * 1024.0 * 1024.0 * 1024.0),
		("TB", 1000.0 * 1000.0 * 1000.0 * 1000.0),
		("GiB", 1024.0 * 1024.0 * 1024.0),
		("GB", 1000.0 * 1000.0 * 1000.0),
		("MiB", 1024.0 * 1024.0),
		("MB", 1000.0 * 1000.0),
		("KiB", 1024.0),
		("KB", 1000.0),
		("kB", 1000.0),
		("B", 1.0),
	];

	for (suffix, multiplier) in units {
		if let Some(num_str) = s.strip_suffix(suffix) {
			if let Ok(num) = num_str.trim().parse::<f64>() {
				return (num * multiplier) as u64;
			}
		}
	}

	0
}
