use std::{
    collections::VecDeque, sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}
};

use crossbeam::channel::Receiver;

use crate::{
    block::{Block, BlockHeader},
    config::models::MiningConfig,
    crypto::{account::Account, hash_utils::HashResult, merkle_tree::generate_merkle_root},
    mining::pow_utils::proof_of_work,
    transaction::Transaction, types::Satoshi,
};

pub struct Miner {
    config: MiningConfig,
    difficulty: u8,
    hash_per_secs: f64,
    account: Arc<Account>,
    last_mining_times: VecDeque<f64>,
}

impl Miner {
    pub fn new(config: MiningConfig, account: Arc<Account>) -> Self {
        Self {
            difficulty: config.start_difficulty_bit,
            hash_per_secs: 0.0,
            last_mining_times: VecDeque::with_capacity(config.block_adjustment_interval),
            account,
            config,
        }
    }

    pub fn mine(
        &self,
        transactions: &[Transaction],
        cancel_mine_rx: Receiver<()>,
        hash_count: &mut u64,
        prev_block_hash: HashResult,
        reward: Satoshi,
    ) -> Option<Block> {

        let coinbase = Transaction::create_coinbase(reward);
        let coinbase_hash = coinbase.hash();
        let coinbase_amount = coinbase.get_amount(0).unwrap_or(0);
        let mut txs = vec![
            coinbase,
            Transaction::create_pay_to_pubkey_hash(
                coinbase_hash,
                0,
                coinbase_amount,
                &self.account,
            ),
        ];
        txs.append(&mut transactions.to_vec());

        let tx_hashes = txs.iter().map(|x| x.hash()).collect();
        let merkle_root = generate_merkle_root(tx_hashes);

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
            let block = Block::new(block_header, txs, block_hash);
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
