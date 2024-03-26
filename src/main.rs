use ember_chain::{blockchain::Blockchain, config::loader::load_toml};
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    env_logger::init();

    let app_name = "ember-chain";
    log::info!("Application '{}' started.", app_name);

    let config = load_toml("./config.toml");
    log::info!("{:#?}", config);

    let mut blockchain = Blockchain::new(config);
    blockchain.run();
}
