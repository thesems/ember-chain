use std::io::{self, stdout, Write};

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

    #[arg(short, long, value_name = "start_console | query_balance")]
    action: String,
}

fn load_config(config_path: &str) -> WalletConfig {
    let config = load_toml_wallet(config_path);
    log::info!("{:#?}", config);
    config
}

#[derive(Debug)]
enum Action {
    StartConsole,
    QueryBalance,
    CreateTransaction,
    ConnectNode,
}
impl Action {
    fn from(action: &str) -> Action {
        match action {
            "start_console" => Action::StartConsole,
            "query_balance" => Action::QueryBalance,
            "create_transaction" => Action::CreateTransaction,
            "connect_node" => Action::ConnectNode,
            _ => panic!("Invalid action {}", action),
        }
    }
}

fn start_console(wallet: &mut Wallet) {
    loop {
        print!("(console) > ");
        stdout().flush().unwrap();

        let mut buff = String::new();
        let _ = io::stdin().read_line(&mut buff);

        let command = buff.trim();
        match command {
            "quit" => break,
            "get_balance" => wallet.query_balance(),
            _ => log::warn!("Command `{}` not found.", command),
        }
    }
    log::info!("Exit console.");
}

fn run_action(wallet: &mut Wallet, action: Action) {
    match action {
        Action::StartConsole => start_console(wallet),
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

    if wallet.connect_node(&config.rpc_url).is_err() {
        return;
    }

    run_action(&mut wallet, Action::from(cli.action.as_str()));
}
