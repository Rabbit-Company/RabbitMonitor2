use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{response::Html, routing::get, Router};
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

	/// Network interface name for monitoring network
	#[arg(short, long, default_value_t = String::from("eth0"))]
	interface: String,

	/// Logger level
	#[arg(short, long, default_value_t = 1)]
	logger: u8,

	/// Bearer token for authentication (optional)
	#[arg(short, long)]
	token: Option<String>,
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
			temp.settings.interface = args.interface;
			temp.settings.logger = args.logger;
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
	State((state, _)): State<(Arc<Mutex<Monitor>>, Option<String>)>
) -> Html<String> {
	Html(utils::main_page(state))
}

async fn metrics(
	auth: Option<TypedHeader<Authorization<Bearer>>>,
	State((state, token)): State<(Arc<Mutex<Monitor>>, Option<String>)>
) -> impl IntoResponse {
	if let Some(token) = &token {
		if let Some(TypedHeader(auth)) = auth {
			if auth.token() == token {
				return Html(utils::create_metrics(state)).into_response();
			}
		}
		return (StatusCode::UNAUTHORIZED, "Invalid or missing token").into_response();
	}

	Html(utils::create_metrics(state)).into_response()
}