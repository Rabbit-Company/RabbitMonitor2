use axum::extract::State;
use monitor::Monitor;
use clap::Parser;
use std::{thread::sleep, time::Duration};
use std::sync::{Arc, Mutex, MutexGuard};
use axum::{routing::get,Router, response::Html};

pub mod utils;
pub mod monitor;

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
}

#[tokio::main]
async fn main() {

	let args: Args = Args::parse();
	let monitor: Arc<Mutex<Monitor>> = Arc::new(Mutex::new(Monitor::new()));
	let cloned: Arc<Mutex<Monitor>> = monitor.clone();

	std::thread::spawn(move || {
		{
			let mut temp: MutexGuard<Monitor> = monitor.lock().unwrap();
			temp.settings.cache = args.cache;
			temp.settings.interface = args.interface;
			temp.settings.logger = args.logger;
		}

		loop{
			{
				let mut temp: MutexGuard<Monitor> = monitor.lock().unwrap();
				temp.refresh();
			}
			sleep(Duration::from_millis(args.cache * 1000));
		}
	});

	let app: Router<_, _> = Router::new()
		.route("/", get(index))
		.route("/metrics", get(metrics))
		.with_state(cloned);

	let address: String = args.address + ":" + &args.port.to_string();
	axum::Server::bind(&address.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
}

async fn index(State(state): State<Arc<Mutex<Monitor>>>) -> Html<String>{
	Html(utils::main_page(state))
}

async fn metrics(State(state): State<Arc<Mutex<Monitor>>>) -> Html<String>{
	Html(utils::create_metrics(state))
}