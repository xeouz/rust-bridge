#[macro_use]
extern crate rocket;

mod cli;
mod generator;

mod route_handler;
use std::net::IpAddr;
use std::str::FromStr;

use cli::ArgCommand;
use cli::parse_args;
use generator::generate_files;

fn handle_generate_command(path: &str) {
    generate_files(path);
}

fn handle_cli() -> bool {
    let result = parse_args();
    if result.is_err() { panic!() }
    let args = result.unwrap(); 

    let should_continue = match args.get_command() {
        ArgCommand::Run => true,
        ArgCommand::Generate(path) => {
            handle_generate_command(path.as_str());
            false
        },
    };

    should_continue
}

#[tokio::main]
async fn main() -> pyo3::PyResult<()> {
    let should_continue = handle_cli();
    if !should_continue {
        return Ok(());
    }

    let (collection, routes) = route_handler::initiate().await.unwrap();

    let ip_str = collection.get_config().get_ip();
    let ip = IpAddr::from_str(ip_str).expect(format!("rocket: launch: Could not parse IP Address `{}`", ip_str).as_str());
    let figment = rocket::Config::figment()
        .merge(("port", collection.get_config().get_port()))
        .merge(("address", ip));

    let _rocket = rocket::custom(figment)
        .mount("/", routes)
        .manage(collection)
        .ignite().await.unwrap()
        .launch().await.unwrap();

    Ok(())
}