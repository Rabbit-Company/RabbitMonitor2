use crate::monitor::Monitor;
use std::sync::{Arc, Mutex, MutexGuard};
use std::fmt::Display;

pub fn mega_bits<T: Into<f64>>(bytes: T) -> f64{
	(bytes.into() / 1048576.0) * 8.0
}

fn create_metric(name: impl Display, description: impl Display, value: impl Display) -> String {
  format!("# HELP rabbit_{name} {description}\n# TYPE rabbit_{name} gauge\nrabbit_{name} {value}\n")
}

pub fn create_metrics(monitor: Arc<Mutex<Monitor>>) -> String{
	let mut metrics: String = String::from("");
	{
		let temp: MutexGuard<Monitor> = monitor.lock().unwrap();

		if temp.settings.logger >= 1 {
			metrics += &create_metric("cpu_load_1min", "CPU load recorded in last minute", temp.processor.min1.to_string());
			metrics += &create_metric("cpu_load_5min", "CPU load recorded in last 5 minutes", temp.processor.min5.to_string());
			metrics += &create_metric("cpu_load_15min", "CPU load recorded in last 15 minutes", temp.processor.min15.to_string());

			metrics += &create_metric("memory_total", "Total memory in bytes", temp.memory.total.to_string());
			metrics += &create_metric("memory_available", "Available memory in bytes", temp.memory.available.to_string());
			metrics += &create_metric("memory_used", "Used memory in bytes", temp.memory.used.to_string());
			metrics += &create_metric("memory_free", "Free memory in bytes", temp.memory.free.to_string());

			metrics += &create_metric("swap_total", "Total swap storage in bytes", temp.swap.total.to_string());
			metrics += &create_metric("swap_used", "Used swap storage in bytes", temp.swap.used.to_string());
			metrics += &create_metric("swap_free", "Free swap storage in bytes", temp.swap.free.to_string());

			metrics += &create_metric("storage_total", "Total storage in bytes", temp.storage.total.to_string());
			metrics += &create_metric("storage_used", "Used storage in bytes", temp.storage.used.to_string());
			metrics += &create_metric("storage_free", "Free storage in bytes", temp.storage.free.to_string());
		}

		metrics += &create_metric("cpu_load_percent", "CPU load in percent", format!("{:.2}", temp.processor.percent));
		metrics += &create_metric("memory_percent", "Used memory in percent", format!("{:.2}", temp.memory.percent));
		metrics += &create_metric("swap_percent", "Used swap storage in percent", format!("{:.2}", temp.swap.percent));
		metrics += &create_metric("storage_percent", "Used storage in percent", format!("{:.2}", temp.storage.percent));
		metrics += &create_metric("network_download", "Download speed in bytes", temp.network.download.to_string());
		metrics += &create_metric("network_upload", "Upload speed in bytes", temp.network.upload.to_string());
	}
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
		<b>Version:</b> v5.0.0</br>
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
			<th>Download</th>
			<td>" + &format!("{:.2}", temp.network.download) + " Mbps</td>
		</tr>
		<tr>
			<th>Upload</th>
			<td>" + &format!("{:.2}", temp.network.upload) + " Mbps</td>
		</tr>
		</table>
		<body>
	</html>"
}