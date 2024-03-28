use ember_chain::config::loader::load_toml_wallet;

fn main() {
    env_logger::init();

    let app_name = "ember-wallet";
    log::info!("{} started.", app_name);

    let config = load_toml_wallet("./configs/wallet.toml");
    log::info!("{:#?}", config);
}
