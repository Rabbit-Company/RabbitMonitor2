use axum::extract::State;
use axum::response::IntoResponse;
use axum::{response::Html, routing::get, Router};
use axum::http::{StatusCode, header, HeaderValue};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use clap::Parser;
use monitor::Monitor;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{thread::sleep, time::Duration};

use crate::monitor::energy::Energy;
use crate::monitor::settings::EnergySettings;
use crate::monitor::ups::UPS;

pub mod monitor;
pub mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
	struct Args {
	/// Bind the server to specific address
	#[arg(short, long, default_value_t = String::from("0.0.0.0"))]
	address: String,

	/// Bind the server to specific port
	#[arg(short, long, default_value_t = 8088)]
	port: u16,

	/// Cache time in seconds
	#[arg(short, long, default_value_t = 3)]
	cache: u64,

	/// Bearer token for authentication (optional)
	#[arg(short, long)]
	token: Option<String>,

	/// Show available network interfaces and exit
	#[arg(long)]
	interface_list: bool,

	/// Show available storage devices and exit
	#[arg(long)]
	storage_list: bool,

	/// Show available batteries and exit
	#[arg(long)]
	battery_list: bool,

	/// Show available UPS and exit
	#[arg(long)]
	ups_list: bool,

	/// Show available components and exit
	#[arg(long)]
	component_list: bool,

	/// Show all processes and exit
	#[arg(long)]
	process_list: bool,

	/// Comma-separated list of network interfaces to monitor (e.g., "eth0,wlan0")
	#[arg(long, value_delimiter = ',')]
	interfaces: Vec<String>,

	/// Comma-separated list of mount points to monitor (e.g., "/,/mnt/data")
	#[arg(long, value_delimiter = ',')]
	mounts: Vec<String>,

	/// Comma-separated list of components to monitor (e.g., "GPU,Battery")
	#[arg(long, value_delimiter = ',')]
	components: Vec<String>,

	/// Comma-separated list of process PIDs or names to monitor (e.g., "18295,rabbitmonitor")
	#[arg(long, value_delimiter = ',')]
	processes: Vec<String>,

	/// Enable all detailed metrics
	#[arg(long, default_value_t = false)]
	all_metrics: bool,

	/// Enable detailed CPU metrics
	#[arg(long, default_value_t = false)]
	cpu_details: bool,

	/// Enable detailed memory metrics
	#[arg(long, default_value_t = false)]
	memory_details: bool,

	/// Enable detailed swap metrics
	#[arg(long, default_value_t = false)]
	swap_details: bool,

	/// Enable detailed storage metrics
	#[arg(long, default_value_t = false)]
	storage_details: bool,

	/// Enable detailed network metrics
	#[arg(long, default_value_t = false)]
	network_details: bool,
}

#[tokio::main]
async fn main() {
	let args: Args = Args::parse();
	let monitor: Arc<Mutex<Monitor>> = Arc::new(Mutex::new(Monitor::new()));
	let cloned: Arc<Mutex<Monitor>> = monitor.clone();
	let token: Option<String> = args.token.clone();

	if args.interface_list {
		let interfaces = sysinfo::Networks::new_with_refreshed_list();
		println!("Available network interfaces:");
		for (name, _) in interfaces.iter() {
			println!("- {}", name);
		}
		return;
	}

	if args.storage_list {
		let disks = sysinfo::Disks::new_with_refreshed_list();
		println!("Available storage devices:");
		for disk in disks.iter() {
			let name = disk.name().to_string_lossy();
			let mount = disk.mount_point().to_string_lossy();
			println!("- {} (mount: {})", name, mount);
		}
		return;
	}

	if args.component_list {
		let components = sysinfo::Components::new_with_refreshed_list();
		println!("Available components:");
		for component in &components {
			println!("- {} ({}°C)", component.label(), component.temperature().unwrap_or(0.0));
		}
		return;
	}

	if args.process_list {
		let mut system = sysinfo::System::new_all();
		system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

		let mut processes: Vec<_> = system.processes().iter().collect();
		processes.sort_by_key(|(_, p)| p.name().to_string_lossy());

		println!("Available processes:");
		for (pid, process) in processes {
			println!("- {} ({}) [{}]", process.name().to_string_lossy(), pid, process.status());
		}
		return;
	}

	if args.ups_list {
		let upses = UPS::detect_ups().unwrap_or_else(|| vec![]);

    if upses.is_empty() {
			println!("No UPS devices detected.");
			return;
    }

		println!("Available UPS devices:");
		for ups_name in upses {
			println!("- {}", ups_name);

			match UPS::get_ups_data(&ups_name, Duration::from_secs(0)) {
				Some(ups) => {
					println!("  - Model: {}", ups.model);
					println!("  - Status: {}", ups.status);
					println!("  - Load: {:.1}%", ups.load_percent);
					println!("  - Charge: {:.1}%", ups.charge_percent);
				}
				None => {
					println!("  - Failed to fetch UPS data.");
				}
			}
    }
		return;
	}

	if args.battery_list {
		match starship_battery::Manager::new() {
			Ok(manager) => {
				match manager.batteries() {
					Ok(batteries) => {
						println!("Available batteries:");
						for (i, battery) in batteries.enumerate() {
							match battery {
								Ok(bat) => {
									println!("\nBattery #{}:", i + 1);
									println!("  - Vendor: {}", bat.vendor().unwrap_or("Unknown"));
									println!("  - Model: {}", bat.model().unwrap_or("Unknown"));
									println!("  - Serial: {}", bat.serial_number().unwrap_or("Unknown"));
									println!("  - Technology: {:?}", bat.technology());
									println!("  - State: {:?}", bat.state());
									println!("  - Charge: {:.1}%", bat.state_of_charge().value * 100.0);
									println!("  - Energy: {:.2} Wh / {:.2} Wh", bat.energy().value / 3600.0, bat.energy_full().value / 3600.0);
									println!("  - Health: {:.2}%", bat.state_of_health().value * 100.0);
									println!("  - Voltage: {:.2} V", bat.voltage().value);
									if let Some(temp) = bat.temperature() {
										println!("  - Temperature: {:.1}°C", temp.value);
									}
									if let Some(cycles) = bat.cycle_count() {
										println!("  - Cycle count: {}", cycles);
									}
									if let Some(time) = bat.time_to_full() {
										println!("  - Time to charge: {:.2} minutes", time.value / 60.0);
									}
									if let Some(time) = bat.time_to_empty() {
										println!("  - Time to discharge: {:.2} minutes", time.value / 60.0);
									}
									println!();
								},
								Err(e) => {
									println!("  - Error reading battery info: {}", e);
								}
							}
						}
					},
					Err(e) => {
						println!("Failed to list batteries: {}", e);
					}
				}
			},
			Err(e) => {
				println!("Failed to initialize battery manager: {}", e);
			}
		}
		return;
  }

	let enable_ipmitool = Energy::get_power_usage_w().is_some();

	let power_usage_interval = Energy::get_dcmi_power_with_info()
    .and_then(|dcmi| dcmi.power.and(dcmi.sampling_period_seconds));

	let upses = UPS::detect_ups().unwrap_or(Vec::new());

	std::thread::spawn(move || {
		{
			let mut temp: MutexGuard<Monitor> = monitor.lock().unwrap();
			temp.settings.cache = args.cache;
			temp.settings.interfaces = args.interfaces;
			temp.settings.energy = EnergySettings{ enabled: enable_ipmitool, interval: power_usage_interval };
			temp.settings.upses = upses;
			temp.settings.mounts = args.mounts;
			temp.settings.components = args.components;
			temp.settings.processes = args.processes;
			temp.settings.all_metrics = args.all_metrics;
			temp.settings.cpu_details = args.cpu_details;
			temp.settings.memory_details = args.memory_details;
			temp.settings.swap_details = args.swap_details;
			temp.settings.storage_details = args.storage_details;
			temp.settings.network_details = args.network_details;
		}

		loop {
			{
				let mut temp: MutexGuard<Monitor> = monitor.lock().unwrap();
				temp.refresh();
			}
			sleep(Duration::from_millis(args.cache * 1000));
		}
	});

	let app = Router::new()
		.route("/", get(index))
		.route("/metrics", get(metrics))
		.with_state((cloned, token));

	let address = format!("{}:{}", args.address, args.port);
	let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
	println!(
		"Rabbit Monitor listening on {} (Auth: {})",
		&address,
		if args.token.is_some() { "Enabled" } else { "Disabled" }
	);
	axum::serve(listener, app).await.unwrap();
}

async fn index(
	State((state, token)): State<(Arc<Mutex<Monitor>>, Option<String>)>
) -> impl IntoResponse {
	if token.is_some() {
		return (StatusCode::NOT_FOUND, "Rabbit Monitor v10.1.0\n\n\nMain page is disabled when Bearer authentication is enabled.").into_response();
	}

	Html(utils::main_page(state)).into_response()
}

async fn metrics(
	auth: Option<TypedHeader<Authorization<Bearer>>>,
	State((state, token)): State<(Arc<Mutex<Monitor>>, Option<String>)>
) -> impl IntoResponse {
	if let Some(token) = &token {
		if let Some(TypedHeader(auth)) = auth {
			if auth.token() == token {
				let body = utils::create_metrics(state);
				return (
					StatusCode::OK,
					[(header::CONTENT_TYPE, HeaderValue::from_static("application/openmetrics-text; version=1.0.0; charset=utf-8"))],
					body,
				).into_response();
			}
		}
		return (StatusCode::UNAUTHORIZED, "Unauthorized: A valid Bearer token is required to access this endpoint.").into_response();
	}

	let body = utils::create_metrics(state);
	(
		StatusCode::OK,
		[(header::CONTENT_TYPE, HeaderValue::from_static("application/openmetrics-text; version=1.0.0; charset=utf-8"))],
		body,
	).into_response()
}