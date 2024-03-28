use dotenv::dotenv;
use ember_chain::{blockchain::Blockchain, config::loader::load_toml};

fn main() {
    dotenv().ok();
    env_logger::init();

    let app_name = "ember-chain";
    log::info!("Application '{}' started.", app_name);

    let config = load_toml("./config.toml");
    log::info!("{:#?}", config);

    let mut blockchain = match Blockchain::new(config) {
        Ok(bs) => bs,
        Err(e) => {
            log::error!("Failed to start blockchain: {:?}", e);
            std::process::exit(1);
        }
    };
    blockchain.run();
}
