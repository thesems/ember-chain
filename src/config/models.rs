use serde::Deserialize;

use crate::types::Satoshi;

#[derive(Deserialize, Clone, Debug)]
pub struct MiningConfig {
    pub block_time_secs: usize,
    pub block_adjustment_interval: usize,
    pub start_difficulty_bit: u8,
    pub mining_reward: Satoshi,
    pub reward_halvening_interval: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SimulationConfig {
    pub fake_mining: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NetworkConfig {
    pub port: u16,
    pub seed_list: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AccountConfig {
    pub keys_path: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub mining: MiningConfig,
    pub simulation: SimulationConfig,
    pub network: NetworkConfig,
    pub account: AccountConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WalletConfig {
    pub account: AccountConfig,
}
