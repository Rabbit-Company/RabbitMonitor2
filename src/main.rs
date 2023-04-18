#[macro_use] extern crate rocket;
use rocket::response::content;
use clap::Parser;

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
	#[arg(short, long, default_value_t = 5)]
	cache: u64,

	/// Network interface name for monitoring network
	#[arg(short, long, default_value_t = String::from("eth0"))]
	interface: String,
}

#[get("/")]
fn index() -> content::RawHtml<String> {
	return content::RawHtml(utils::main_page());
}

#[get("/metrics")]
fn metrics() -> String {
	return utils::create_metrics();
}

#[launch]
fn rocket() -> _ {

	let args: Args = Args::parse();

	std::thread::spawn(move || {
		utils::initialize(args.cache, args.interface);
	});

	let figment: rocket::figment::Figment = rocket::Config::figment()
		.merge(("address", args.address))
		.merge(("port", args.port));

	rocket::custom(figment)
		.mount("/", routes![index])
		.mount("/", routes![metrics])
}