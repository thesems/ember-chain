use clap::Parser;
use dotenv::dotenv;
use ember_chain::{
    config::{loader::load_toml_wallet, models::WalletConfig},
    wallet::wallet::Wallet,
};
use tokio::runtime::Runtime;

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

    #[arg(short, long)]
    action: String,
}

fn load_config(config_path: &str) -> WalletConfig {
    let config = load_toml_wallet(config_path);
    log::info!("{:#?}", config);
    config
}

#[derive(Debug)]
enum Action {
    QueryBalance,
    CreateTransaction,
    ConnectNode,
}
impl Action {
    fn from(action: &str) -> Action {
        match action {
            "query_balance" => Action::QueryBalance,
            "create_transaction" => Action::CreateTransaction,
            "connect_node" => Action::ConnectNode,
            _ => panic!("Invalid action {}", action),
        }
    }
}

fn run_action(wallet: &mut Wallet, action: Action) {
    match action {
        Action::QueryBalance => wallet.query_balance(),
        action => panic!("Unimplemented action {:?}", action),
    };
}

fn main() {
    dotenv().ok();
    env_logger::init();
    let cli = Args::parse();
    let config = load_config(cli.config_path.as_str());
    let rt = Runtime::new().unwrap();
    let mut wallet = Wallet::new(&rt, config.clone()).unwrap();

    wallet.connect_node(&config.rpc_url);
    run_action(&mut wallet, Action::from(cli.action.as_str()));
}
