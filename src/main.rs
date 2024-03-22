use ember_chain::blockchain::Blockchain;
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    env_logger::init();

    let app_name = "ember-chain";
    log::info!("Application '{}' started.", app_name);

    let mut blockchain = Blockchain::default();
    blockchain.run();
}
