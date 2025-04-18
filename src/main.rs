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

	/// Comma-separated list of network interfaces to monitor (e.g., "eth0,wlan0")
	#[arg(long, value_delimiter = ',')]
	interfaces: Vec<String>,

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

	std::thread::spawn(move || {
		{
			let mut temp: MutexGuard<Monitor> = monitor.lock().unwrap();
			temp.settings.cache = args.cache;
			temp.settings.interfaces = args.interfaces;
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
		return (StatusCode::NOT_FOUND, "Rabbit Monitor v7.0.2\n\n\nMain page is disabled when Bearer authentication is enabled.").into_response();
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