use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct MiningConfig {
    pub block_time_secs: usize,
    pub block_adjustment_frequency: usize,
    pub start_difficulty_bit: u8,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub mining: MiningConfig,
}

