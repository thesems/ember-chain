use std::{
    collections::VecDeque,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossbeam::channel::Receiver;

use crate::{
    block::{Block, BlockHeader}, config::models::MiningConfig, crypto::hash_utils::HashResult, mining::pow_utils::proof_of_work, transaction::Transaction
};

pub struct Miner {
    config: MiningConfig,
    difficulty: u8,
    hash_per_secs: f64,
    last_mining_times: VecDeque<f64>,
}

impl Miner {
    pub fn new(config: MiningConfig) -> Self {
        Self {
            difficulty: config.start_difficulty_bit,
            hash_per_secs: 0.0,
            last_mining_times: VecDeque::with_capacity(config.block_adjustment_interval),
            config,
        }
    }

    pub fn mine(
        &self,
        merkle_root: HashResult,
        transactions: &[Transaction],
        cancel_mine_rx: Receiver<()>,
        hash_count: &mut u64,
        prev_block_hash: HashResult,
    ) -> Option<Block> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut block_header =
            BlockHeader::from(merkle_root, prev_block_hash, self.difficulty, timestamp);
        if let Some(block_hash) = proof_of_work(
            self.difficulty,
            &mut block_header,
            cancel_mine_rx,
            hash_count,
        ) {
            let block = Block::new(block_header, transactions.to_vec(), block_hash);
            return Some(block);
        }
        None
    }

    pub fn adjust_difficulty(&mut self) {
        let avg_mining_time =
            self.last_mining_times.iter().sum::<f64>() / self.last_mining_times.len() as f64;
        let previous_difficulty = self.difficulty;

        if avg_mining_time < self.config.block_time_secs as f64 * 0.8 {
            self.difficulty += 1;
        } else if avg_mining_time > self.config.block_time_secs as f64 * 1.2 {
            self.difficulty -= 1;
        }

        if previous_difficulty != self.difficulty {
            log::info!(
                "Adjust difficulty from {} to {}.",
                previous_difficulty,
                self.difficulty
            );
        }

        log::info!(
            "Average block time during last {} blocks was {} seconds.",
            self.config.block_adjustment_interval,
            avg_mining_time
        );
    }

    pub fn add_mining_time(&mut self, duration: Duration, hash_count: u64) {
        if self.last_mining_times.len() >= self.config.block_adjustment_interval {
            self.last_mining_times.pop_front();
        }
        self.last_mining_times.push_back(duration.as_secs_f64());
        self.hash_per_secs = (hash_count as f64 / duration.as_millis() as f64) * 1000.0;
        log::debug!("Average hash per second: {:.2}", self.hash_per_secs);
    }
}
