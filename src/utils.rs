use crate::monitor::Monitor;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

pub fn mega_bits<T: Into<f64>>(bytes: T) -> f64{
	(bytes.into() / 1048576.0) * 8.0
}

pub fn format_bytes_per_sec(bytes: f64) -> String {
	format!("{:.2}", bytes / (1024.0 * 1024.0))
}

fn parse_labels(labels: &[(&str, &str)]) -> String {
	if !labels.is_empty() {
		let label_pairs = labels
			.iter()
			.map(|(k, v)| format!("{k}=\"{v}\""))
			.collect::<Vec<_>>()
			.join(",");
		format!("{{{}}}", label_pairs)
	} else {
		String::new()
	}
}

fn create_metric_header(name: &str, description: &str, metric_type: &str, unit: Option<&str>) -> String {
	let unit_line = if let Some(unit) = unit {
		format!("# UNIT rabbit_{} {}\n", name, unit)
	} else {
		String::new()
	};
	format!("# HELP rabbit_{name} {description}\n# TYPE rabbit_{name} {metric_type}\n{unit_line}")
}

fn create_info_metric(name: &str, description: &str, labels: &[(&str, &str)]) -> String {
	let header = create_metric_header(name, description, "info", None);
	let labels_str = parse_labels(labels);

	format!("{header}rabbit_{name}{{{labels_str}}} 1\n")
}

fn create_gauge_metric(name: &str, description: &str, value: &str, unit: Option<&str>, labels: &[(&str, &str)], timestamp: Duration) -> String {
	let ts = timestamp.as_secs_f64();
	let header = create_metric_header(name, description, "gauge", unit);
	let labels_str = parse_labels(labels);

  format!("{header}rabbit_{name}{labels_str} {value} {ts:.3}\n")
}

fn create_gauge_metric_line(name: &str, value: &str, labels: &[(&str, &str)], timestamp: Duration) -> String {
	let ts = timestamp.as_secs_f64();
	let label_str = parse_labels(labels);

	format!("rabbit_{}{} {} {:.3}\n", name, label_str, value, ts)
}

/*
fn create_counter_metric(name: &str, description: &str, value: &str, unit: Option<&str>, labels: &[(&str, &str)], timestamp: Duration, created: Duration) -> String {
	let ts = timestamp.as_secs_f64();
	let cr = created.as_secs_f64();
	let header = create_metric_header(name, description, "counter", unit);
	let labels_str = parse_labels(labels);

	format!("{header}rabbit_{name}_total{labels_str} {value} {ts:.3}\nrabbit_{name}_created{labels_str} {cr:.3} {ts:.3}\n")
}
*/

fn create_counter_metric_line(name: &str, value: &str, labels: &[(&str, &str)], timestamp: Duration, created: u64) -> String {
	let ts = timestamp.as_secs_f64();
	let labels_str = parse_labels(labels);

	format!("rabbit_{name}_total{labels_str} {value} {ts:.3}\nrabbit_{name}_created{labels_str} {created:.3} {ts:.3}\n")
}

pub fn create_metrics(monitor: Arc<Mutex<Monitor>>) -> String{
	let mut metrics: String = String::from("");
	{
		let temp: MutexGuard<Monitor> = monitor.lock().unwrap();

		metrics += &create_info_metric("version_info", "Rabbit Monitor version", &[("version", "v7.0.0")]);
		metrics += &create_info_metric("system_info", "System information", &[
			("name", &temp.system_info.name),
			("kernel_version", &temp.system_info.kernel_version),
			("os_version", &temp.system_info.os_version),
			("long_os_version", &temp.system_info.long_os_version),
			("distribution_id", &temp.system_info.distribution_id),
			("host_name", &temp.system_info.host_name),
			("boot_time", &temp.system_info.boot_time.to_string()),
		]);
		metrics += &create_info_metric("cpu_info", "Static CPU information", &[("threads", &temp.processor.threads.to_string())]);

		if temp.settings.cpu_details {
			metrics += &create_gauge_metric("cpu_load_1min", "CPU load recorded in last minute", &temp.processor.min1.to_string(), None, &[], temp.processor.refreshed);
			metrics += &create_gauge_metric("cpu_load_5min", "CPU load recorded in last 5 minutes", &temp.processor.min5.to_string(), None, &[], temp.processor.refreshed);
			metrics += &create_gauge_metric("cpu_load_15min", "CPU load recorded in last 15 minutes", &temp.processor.min15.to_string(), None, &[], temp.processor.refreshed);
		}

		if temp.settings.memory_details {
			metrics += &create_gauge_metric("memory_total", "Total memory in bytes", &temp.memory.total.to_string(), Some("bytes"), &[], temp.memory.refreshed);
			metrics += &create_gauge_metric("memory_available", "Available memory in bytes", &temp.memory.available.to_string(), Some("bytes"), &[], temp.memory.refreshed);
			metrics += &create_gauge_metric("memory_used", "Used memory in bytes", &temp.memory.used.to_string(), Some("bytes"), &[], temp.memory.refreshed);
			metrics += &create_gauge_metric("memory_free", "Free memory in bytes", &temp.memory.free.to_string(), Some("bytes"), &[], temp.memory.refreshed);
		}

		if temp.settings.swap_details {
			metrics += &create_gauge_metric("swap_total", "Total swap storage in bytes", &temp.swap.total.to_string(), Some("bytes"), &[], temp.swap.refreshed);
			metrics += &create_gauge_metric("swap_used", "Used swap storage in bytes", &temp.swap.used.to_string(), Some("bytes"), &[], temp.swap.refreshed);
			metrics += &create_gauge_metric("swap_free", "Free swap storage in bytes", &temp.swap.free.to_string(), Some("bytes"), &[], temp.swap.refreshed);
		}

		metrics += &create_gauge_metric("cpu_load_percent", "CPU load in percent", &format!("{:.2}", temp.processor.percent), Some("percent"), &[], temp.processor.refreshed);
		metrics += &create_gauge_metric("memory_percent", "Used memory in percent", &format!("{:.2}", temp.memory.percent), Some("percent"), &[], temp.memory.refreshed);
		metrics += &create_gauge_metric("swap_percent", "Used swap storage in percent", &format!("{:.2}", temp.swap.percent), Some("percent"), &[], temp.swap.refreshed);

		if !temp.storage_devices.is_empty() {
			metrics += &create_metric_header("storage_percent", "Used storage in percent", "gauge", Some("percent"));
			for (device, storage) in &temp.storage_devices {
				metrics += &create_gauge_metric_line("storage_percent", &storage.percent.to_string(), &[("device", device), ("mount", &storage.mount_point)], storage.refreshed);
			}

			metrics += &create_metric_header("storage_read_speed", "Disk read speed in bytes/sec", "gauge", Some("bytes_per_second"));
			for (device, storage) in &temp.storage_devices {
				metrics += &create_gauge_metric_line("storage_read_speed", &storage.read_speed.to_string(), &[("device", device), ("mount", &storage.mount_point)], storage.refreshed);
			}

			metrics += &create_metric_header("storage_write_speed", "Disk write speed in bytes/sec", "gauge", Some("bytes_per_second"));
			for (device, storage) in &temp.storage_devices {
				metrics += &create_gauge_metric_line("storage_write_speed", &storage.write_speed.to_string(), &[("device", device), ("mount", &storage.mount_point)], storage.refreshed);
			}

			if temp.settings.storage_details {
				metrics += &create_metric_header("storage_used", "Used storage in bytes", "gauge", Some("bytes"));
				for (device, storage) in &temp.storage_devices {
					metrics += &create_gauge_metric_line("storage_used", &storage.used.to_string(), &[("device", device), ("mount", &storage.mount_point)], storage.refreshed);
				}

				metrics += &create_metric_header("storage_free", "Free storage in bytes", "gauge", Some("bytes"));
				for (device, storage) in &temp.storage_devices {
					metrics += &create_gauge_metric_line("storage_free", &storage.free.to_string(), &[("device", device), ("mount", &storage.mount_point)], storage.refreshed);
				}

				metrics += &create_metric_header("storage_total", "Total storage in bytes", "gauge", Some("bytes"));
				for (device, storage) in &temp.storage_devices {
					metrics += &create_gauge_metric_line("storage_total", &storage.total.to_string(), &[("device", device), ("mount", &storage.mount_point)], storage.refreshed);
				}
			}
		}

		if !temp.network_interfaces.is_empty() {
			metrics += &create_metric_header("network_download_speed", "Download speed in bytes/sec", "gauge", Some("bytes_per_second"));
			for (iface, network) in &temp.network_interfaces {
				metrics += &create_gauge_metric_line("network_download_speed", &network.download.to_string(), &[("interface", iface)], network.refreshed);
			}

			metrics += &create_metric_header("network_upload_speed", "Upload speed in bytes/sec", "gauge", Some("bytes_per_second"));
			for (iface, network) in &temp.network_interfaces {
				metrics += &create_gauge_metric_line("network_upload_speed", &network.upload.to_string(), &[("interface", iface)], network.refreshed);
			}

			if temp.settings.network_details {
				metrics += &create_metric_header("network_packets_received", "Total number of incoming packets", "counter", None);
				for (iface, network) in &temp.network_interfaces {
					metrics += &create_counter_metric_line("network_packets_received", &network.total_packets_received.to_string(), &[("interface", iface)], network.refreshed, temp.system_info.boot_time);
				}

				metrics += &create_metric_header("network_packets_transmitted", "Total number of outcoming packets", "counter", None);
				for (iface, network) in &temp.network_interfaces {
					metrics += &create_counter_metric_line("network_packets_transmitted", &network.total_packets_transmitted.to_string(), &[("interface", iface)], network.refreshed, temp.system_info.boot_time);
				}

				metrics += &create_metric_header("network_errors_received", "Total number of incoming errors", "counter", None);
				for (iface, network) in &temp.network_interfaces {
					metrics += &create_counter_metric_line("network_errors_received", &network.total_errors_on_received.to_string(), &[("interface", iface)], network.refreshed, temp.system_info.boot_time);
				}

				metrics += &create_metric_header("network_errors_transmitted", "Total number of outcoming errors", "counter", None);
				for (iface, network) in &temp.network_interfaces {
					metrics += &create_counter_metric_line("network_errors_transmitted", &network.total_errors_on_transmitted.to_string(), &[("interface", iface)], network.refreshed, temp.system_info.boot_time);
				}
			}
		}
	}
	metrics += "# EOF\n";
	metrics
}

pub fn main_page(monitor: Arc<Mutex<Monitor>>) -> String {
	let temp: MutexGuard<Monitor> = monitor.lock().unwrap();

	let mut html = format!(
		r#"<!DOCTYPE html>
<html>
<head>
	<title>Rabbit Monitor</title>
	<meta http-equiv='refresh' content='{}'>
</head>
<body>
	<style>
		td, th {{
			border-bottom: 1px solid #000;
			border-right: 1px solid #000;
			text-align: center;
			padding: 8px;
		}}
	</style>
	<h1>Rabbit Monitor</h1>
	<b>Version:</b> v7.0.0</br>
	<b>Fetch every:</b> {} seconds</br></br>
	<table>
	<tr><th>CPU Load</th><td>{:.2}%</td></tr>
	<tr><th>RAM Usage</th><td>{:.2}%</td></tr>
	<tr><th>Swap Usage</th><td>{:.2}%</td></tr>"#,
		temp.settings.cache,
		temp.settings.cache,
		temp.processor.percent,
		temp.memory.percent,
		temp.swap.percent
	);

	// Add all disks
	html += r#"<tr><th colspan="2">Storage Devices</th></tr>"#;
	for (name, disk) in &temp.storage_devices {
		html += &format!(
			r#"<tr><th>{} ({})</th><td>{:.2}% used — ↓ {} MB/s / ↑ {} MB/s</td></tr>"#,
			name,
			disk.mount_point,
			disk.percent,
			format_bytes_per_sec(disk.read_speed),
			format_bytes_per_sec(disk.write_speed),
		);
	}

	// Add all network interfaces
	html += r#"<tr><th colspan="2">Network Interfaces</th></tr>"#;
	for (name, iface) in &temp.network_interfaces {
		html += &format!(
			r#"<tr><th>{}</th><td>↓ {:.2} Mbps / ↑ {:.2} Mbps</td></tr>"#,
			name,
			iface.download,
			iface.upload
		);
	}

	html += r#"
	</table>
	</body>
</html>"#;

	html
}