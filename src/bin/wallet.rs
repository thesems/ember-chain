use std::io::{self, stdout, Write};

use clap::Parser;
use dotenv::dotenv;
use tokio::runtime::Runtime;

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
    Address,
    GetBalance,
    CreateTransaction,
}
impl ConsoleAction {
    fn from(action: &str) -> ConsoleAction {
        match action {
            "address" => ConsoleAction::Address,
            "balance" => ConsoleAction::GetBalance,
            "create_transaction" => ConsoleAction::CreateTransaction,
            "help" => ConsoleAction::Help,
            "quit" => ConsoleAction::Quit,
            "exit" => ConsoleAction::Quit,
            _ => ConsoleAction::Invalid,
        }
    }
    pub fn into_iter() -> core::array::IntoIter<ConsoleAction, 6> {
        [
            ConsoleAction::Invalid,
            ConsoleAction::Quit,
            ConsoleAction::Help,
            ConsoleAction::Address,
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

        let tokens: Vec<&str> = buff.trim().split(' ').collect();
        let action = ConsoleAction::from(tokens[0]);

        match action {
            ConsoleAction::Quit => break,
            ConsoleAction::Address => {
                log::info!("Address: {}", wallet.get_address());
            }
            ConsoleAction::GetBalance => {
                log::info!(
                    "Balance: {} satoshis.",
                    wallet.get_balance(wallet.account.public_key().to_vec())
                );
            }
            ConsoleAction::CreateTransaction => 'create_tx: {
                if tokens.len() != 3 {
                    log::debug!("usage: create_transaction [receiver_address] [amount]");
                    break 'create_tx;
                }
                let rx_address = wallet.get_address_from_string(tokens[1]);
                if rx_address.is_none() {
                    log::debug!("usage: address must be in a valid hex form");
                    break 'create_tx;
                }
                let rx_address = rx_address.unwrap();

                let amount = tokens[2].parse::<u64>();
                if amount.is_err() {
                    log::debug!("usage: amount must be an integer");
                    break 'create_tx;
                }
                let amount = amount.unwrap();

                match wallet.create_transaction(&rx_address, amount, 0) {
                    Ok(tx_hash) => {
                        log::info!("You sent {} satoshis to {:?}", amount, &rx_address);
                        log::debug!("Transaction hash: {:?}", &tx_hash);
                    }
                    Err(err) => {
                        log::error!("Failed to create transaction: {}", err);
                    }
                }
            }
            ConsoleAction::Help => println!(
                "Actions: {:?}",
                ConsoleAction::into_iter().collect::<Vec<ConsoleAction>>()
            ),
            ConsoleAction::Invalid => log::warn!("Action `{}` not valid.", buff.trim()),
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
