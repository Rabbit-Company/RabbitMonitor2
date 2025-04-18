use crate::monitor::Monitor;
use std::sync::{Arc, Mutex, MutexGuard};
use std::fmt::Display;
use std::time::Duration;

pub fn mega_bits<T: Into<f64>>(bytes: T) -> f64{
	(bytes.into() / 1048576.0) * 8.0
}

pub fn format_bytes_per_sec(bytes: f64) -> String {
	format!("{:.2}", bytes / (1024.0 * 1024.0))
}

fn create_metric(name: impl Display, description: impl Display, value: impl Display, unit: Option<&str>, timestamp: Duration) -> String {
	let ts = timestamp.as_secs_f64();
	let unit_line = if let Some(unit) = unit {
		format!("# UNIT rabbit_{} {}\n", name, unit)
	} else {
		String::new()
	};
  format!("# HELP rabbit_{name} {description}\n# TYPE rabbit_{name} gauge\n{unit_line}rabbit_{name} {value} {ts:.3}\n")
}

pub fn create_metrics(monitor: Arc<Mutex<Monitor>>) -> String{
	let mut metrics: String = String::from("");
	{
		let temp: MutexGuard<Monitor> = monitor.lock().unwrap();

		if temp.settings.logger >= 1 {
			metrics += &create_metric("cpu_load_1min", "CPU load recorded in last minute", temp.processor.min1, None, temp.processor.refreshed);
			metrics += &create_metric("cpu_load_5min", "CPU load recorded in last 5 minutes", temp.processor.min5, None, temp.processor.refreshed);
			metrics += &create_metric("cpu_load_15min", "CPU load recorded in last 15 minutes", temp.processor.min15, None, temp.processor.refreshed);

			metrics += &create_metric("memory_total", "Total memory in bytes", temp.memory.total, Some("bytes"), temp.memory.refreshed);
			metrics += &create_metric("memory_available", "Available memory in bytes", temp.memory.available, Some("bytes"), temp.memory.refreshed);
			metrics += &create_metric("memory_used", "Used memory in bytes", temp.memory.used, Some("bytes"), temp.memory.refreshed);
			metrics += &create_metric("memory_free", "Free memory in bytes", temp.memory.free, Some("bytes"), temp.memory.refreshed);

			metrics += &create_metric("swap_total", "Total swap storage in bytes", temp.swap.total, Some("bytes"), temp.swap.refreshed);
			metrics += &create_metric("swap_used", "Used swap storage in bytes", temp.swap.used, Some("bytes"), temp.swap.refreshed);
			metrics += &create_metric("swap_free", "Free swap storage in bytes", temp.swap.free, Some("bytes"), temp.swap.refreshed);

			metrics += &create_metric("storage_total", "Total storage in bytes", temp.storage.total, Some("bytes"), temp.storage.refreshed);
			metrics += &create_metric("storage_used", "Used storage in bytes", temp.storage.used, Some("bytes"), temp.storage.refreshed);
			metrics += &create_metric("storage_free", "Free storage in bytes", temp.storage.free, Some("bytes"), temp.storage.refreshed);
		}

		metrics += &create_metric("cpu_load_percent", "CPU load in percent", format!("{:.2}", temp.processor.percent), Some("percent"), temp.processor.refreshed);
		metrics += &create_metric("memory_percent", "Used memory in percent", format!("{:.2}", temp.memory.percent), Some("percent"), temp.memory.refreshed);
		metrics += &create_metric("swap_percent", "Used swap storage in percent", format!("{:.2}", temp.swap.percent), Some("percent"), temp.swap.refreshed);
		metrics += &create_metric("storage_percent", "Used storage in percent", format!("{:.2}", temp.storage.percent), Some("percent"), temp.storage.refreshed);

		metrics += &create_metric("storage_read_speed", "Disk read speed in bytes/sec", temp.storage.read_speed, Some("bytes_per_second"), temp.storage.refreshed);
		metrics += &create_metric("storage_write_speed", "Disk write speed in bytes/sec", temp.storage.write_speed, Some("bytes_per_second"), temp.storage.refreshed);

		metrics += &create_metric("network_download_speed", "Download speed in bytes/sec", temp.network.download, Some("bytes_per_second"), temp.network.refreshed);
		metrics += &create_metric("network_upload_speed", "Upload speed in bytes/sec", temp.network.upload, Some("bytes_per_second"), temp.network.refreshed);
	}
	metrics += "# EOF\n";
	metrics
}

pub fn main_page(monitor: Arc<Mutex<Monitor>>) -> String{
	let temp: MutexGuard<Monitor> = monitor.lock().unwrap();
	"<!DOCTYPE html>
	<html>
	<head>
		<title>Rabbit Monitor</title>
		<meta http-equiv='refresh' content='".to_owned() + &temp.settings.cache.to_string() + "'>
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
		<b>Version:</b> v6.1.0</br>
		<b>Fetch every:</b> " + &temp.settings.cache.to_string() + " seconds</br></br>
		<table>
		<tr>
			<th>CPU Load</th>
			<td>" + &format!("{:.2}", temp.processor.percent) + "%</td>
		</tr>
		<tr>
			<th>RAM Usage</th>
			<td>" + &format!("{:.2}", temp.memory.percent) + "%</td>
		</tr>
		<tr>
			<th>Swap Usage</th>
			<td>" + &format!("{:.2}", temp.swap.percent) + "%</td>
		</tr>
		<tr>
			<th>Storage Usage</th>
			<td>" + &format!("{:.2}", temp.storage.percent) + "%</td>
		</tr>
		<tr>
			<th>Read Speed</th>
			<td>" + &format_bytes_per_sec(temp.storage.read_speed) + " MB/s</td>
		</tr>
		<tr>
			<th>Write Speed</th>
			<td>" + &format_bytes_per_sec(temp.storage.write_speed) + " MB/s</td>
		</tr>
		<tr>
			<th>Download Speed</th>
			<td>" + &format!("{:.2}", temp.network.download) + " Mbps</td>
		</tr>
		<tr>
			<th>Upload Speed</th>
			<td>" + &format!("{:.2}", temp.network.upload) + " Mbps</td>
		</tr>
		</table>
		<body>
	</html>"
}