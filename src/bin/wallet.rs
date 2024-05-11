use clap::Parser;
use ember_chain::{
    config::{loader::load_toml_wallet, models::WalletConfig},
    wallet::wallet::Wallet,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value_t = String::from("./configs/wallet.toml")
    )]
    config_path: String,

    #[arg(short, long, value_name = "URL")]
    rpc_url: String,
}

fn load_config() -> WalletConfig {
    let cli = Args::parse();

    let config = load_toml_wallet(cli.config_path.as_str());
    log::info!("{:#?}", config);

    config
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = load_config();

    let app_name = "ember-wallet";
    log::info!("{} started.", app_name);

    let mut wallet = Wallet::new(config).unwrap();
    wallet.load_account().unwrap();
    wallet.connect_node("path".to_string()).await;
}
