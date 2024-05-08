use std::env;

use clap::Parser;
use dotenv::dotenv;
use ember_chain::{
    blockchain::Blockchain,
    config::{loader::load_toml, models::Config},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value_t = String::from("./configs/config.toml")
    )]
    config_path: String,

    #[arg(short, long)]
    network_port: Option<u16>,

    #[arg(short, long)]
    log_level: Option<String>,
}

fn load_config() -> Config {
    let cli = Args::parse();
    let mut config = load_toml(cli.config_path.as_str());

    if let Some(port) = cli.network_port {
        config.network.port = port;
    }

    if let Some(log_level) = cli.log_level {
        env::set_var("RUST_LOG", log_level)
    }

    config
}

fn main() {
    dotenv().ok();

    let config = load_config();
    env_logger::init();

    let app_name = env!("CARGO_PKG_NAME");
    log::info!("Application '{}' started.", app_name);
    log::debug!("{:#?}", config);

    let mut blockchain = match Blockchain::new(config) {
        Ok(bs) => bs,
        Err(e) => {
            log::error!("Failed to start blockchain: {:?}", e);
            std::process::exit(1);
        }
    };
    blockchain.run();
}
