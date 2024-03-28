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
pub struct Config {
    pub mining: MiningConfig,
}

