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
}

fn load_config(config_path: &str) -> WalletConfig {
    let config = load_toml_wallet(config_path);
    log::info!("{:#?}", config);
    config
}

#[derive(Debug)]
enum ConsoleAction {
    Invalid,
    Quit,
    Help,
    GetBalance,
    CreateTransaction,
}
impl ConsoleAction {
    fn from(action: &str) -> ConsoleAction {
        match action {
            "get_balance" => ConsoleAction::GetBalance,
            "create_transaction" => ConsoleAction::CreateTransaction,
            "help" => ConsoleAction::Help,
            "quit" => ConsoleAction::Quit,
            "exit" => ConsoleAction::Quit,
            _ => ConsoleAction::Invalid,
        }
    }
    pub fn into_iter() -> core::array::IntoIter<ConsoleAction, 5> {
        [
            ConsoleAction::Invalid,
            ConsoleAction::Quit,
            ConsoleAction::Help,
            ConsoleAction::GetBalance,
            ConsoleAction::CreateTransaction,
        ]
        .into_iter()
    }
}

fn start_console(wallet: &mut Wallet) {
    if wallet.connect_node().is_err() {
        return;
    }

    loop {
        print!("(console) > ");
        stdout().flush().unwrap();

        let mut buff = String::new();
        let _ = io::stdin().read_line(&mut buff);

        let action = ConsoleAction::from(buff.trim());

        match action {
            ConsoleAction::Quit => break,
            ConsoleAction::GetBalance => wallet.query_balance(),
            ConsoleAction::CreateTransaction => {}
            ConsoleAction::Help => println!(
                "Actions: {:?}",
                ConsoleAction::into_iter().collect::<Vec<ConsoleAction>>()
            ),
            ConsoleAction::Invalid => log::warn!("Action `{}` not valid.", buff.trim()),
            _ => log::warn!("Action `{:?}` not implemented.", action),
        }
    }
    log::info!("Exit console.");
}

fn main() {
    dotenv().ok();
    env_logger::init();
    let cli = Args::parse();
    let config = load_config(cli.config_path.as_str());
    let rt = Runtime::new().unwrap();
    let mut wallet = Wallet::new(&rt, config.clone()).unwrap();

    start_console(&mut wallet);
}
