use crate::{
    block::Block,
    block_header::BlockHeader,
    pow_utils::{compare_difficulty, target_from_difficulty_bit},
    transaction::Transaction,
};
use blockchain_utxo::Hash;
use rand::prelude::*;
use std::{
    collections::{HashMap, VecDeque},
    thread,
    time::{Duration, Instant},
};

const BLOCK_TIME_SECS: u64 = 10;
const BLOCK_ADJUSTMENT_FREQUENCY: usize = 10;
const START_DIFFICULTY_BIT: u32 = 16;
const TEST_MINING_PROBABILITY: f32 = 1.0;

pub struct Blockchain {
    block_uids: HashMap<Hash, usize>,
    blocks: Vec<Block>,
    target: u64,
    hash_per_secs: f64,
    last_mining_times: VecDeque<f64>,
    pending_transactions: Vec<Transaction>,
}
impl Blockchain {
    pub fn new() -> Self {
        Self {
            block_uids: HashMap::new(),
            blocks: vec![],
            target: target_from_difficulty_bit(START_DIFFICULTY_BIT),
            hash_per_secs: 0.0,
            last_mining_times: VecDeque::with_capacity(BLOCK_ADJUSTMENT_FREQUENCY),
            pending_transactions: vec![],
        }
    }
    pub fn run(&mut self) {
        println!("Blockchain started.");
        loop {
            let transactions = vec![];
            let block_header = self.mine_or_receive(merkle_root);
            let block = self.build_block(block_header, transactions);
            self.add_block(block);
            self.adjust_difficulty();
        }
    }
    fn mine_or_receive(&mut self, merkle_root: &Hash) -> BlockHeader {
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < TEST_MINING_PROBABILITY {
            if let Some((block_hash, block_header)) = self.mine(merkle_root) {
                self.block_uids.insert(block_hash, self.blocks.len());
                return block_header;
            }
        }

        // generate block manually
        thread::sleep(Duration::new(BLOCK_TIME_SECS, 0));
        println!("A participant has mined a block before you!");

        let mut previous_block_hash = [0u8; 32];
        if let Some(previous_block) = self.previous_block() {
            previous_block_hash = previous_block.header.merkle_root;
        }
        let mut block_header = BlockHeader::from(previous_block_hash, 0);
        block_header.merkle_root = [0u8; 32];
        block_header.nonce = 10;
        block_header
    }
    fn mine(&mut self, merkle_root: &Hash) -> Option<(Hash, BlockHeader)> {
        let start = Instant::now();
        let mut previous_block_hash = [0u8; 32];
        if let Some(previous_block) = self.previous_block() {
            previous_block_hash = previous_block.header.merkle_root;
        }
        let mut block_header = BlockHeader::from(previous_block_hash, 0);
        block_header.merkle_root = merkle_root.clone();
        if let Some(block_hash) = self.proof_of_work(&mut block_header) {
            self.add_mining_time(start.elapsed(), block_header.nonce);
            return Some((block_hash, block_header));
        }
        return None;
    }
    fn add_mining_time(&mut self, duration: Duration, nonce: u32) {
        if self.last_mining_times.len() >= BLOCK_ADJUSTMENT_FREQUENCY {
            self.last_mining_times.pop_front();
        }
        self.last_mining_times.push_back(duration.as_secs_f64());
        self.hash_per_secs = nonce as f64 / duration.as_millis() as f64;
    }
    fn proof_of_work(&self, block: &mut BlockHeader) -> Option<Hash> {
        let mut block_hash = block.finalize();

        for i in 0..u32::MAX {
            let hash_int: u64 = u64::from_be_bytes(block_hash[..8].try_into().unwrap());
            if compare_difficulty(self.target, hash_int) {
                println!("Succesfully mined a block!");
                return Some(block_hash);
            }

            block.nonce = i;
            block_hash = block.finalize();
        }
        None 
    }
    pub fn adjust_difficulty(&mut self) {
        if self.blocks.len() % BLOCK_ADJUSTMENT_FREQUENCY != 0 {
            return;
        }

        let avg_mining_time = self.get_average_mining_time();
        let previous_difficulty = self.target;

        if avg_mining_time < BLOCK_TIME_SECS as f64 * 0.9
            || avg_mining_time > BLOCK_TIME_SECS as f64 * 1.1
        {
            let total_time: f64 = self.last_mining_times.iter().sum();
            let mut modifier =
                total_time / (BLOCK_ADJUSTMENT_FREQUENCY * BLOCK_TIME_SECS as usize) as f64;

            println!("total time: {}", total_time);
            dbg!(&self.last_mining_times);

            if modifier < 1.0 {
                modifier = modifier.max(0.9);
            } else {
                modifier = modifier.min(1.1);
            }

            self.target = (self.target as f64 * modifier) as u64;
            println!(
                "Adjust difficulty target by from {} to {}.",
                previous_difficulty, self.target
            );
        }
        println!(
            "Average block time during last {} blocks was {} seconds.",
            BLOCK_ADJUSTMENT_FREQUENCY,
            self.get_average_mining_time()
        );
    }
    fn get_average_mining_time(&self) -> f64 {
        self.last_mining_times.iter().sum::<f64>() / self.last_mining_times.len() as f64
    }
    pub fn build_block(&self, block_header: BlockHeader, transactions: Vec<Transaction>) -> Block {
        Block::build(block_header, transactions)
    }
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
        println!("Added a new block.");
        println!("Average hash per second: {:.2}", self.hash_per_secs);
    }
    fn previous_block(&self) -> Option<&Block> {
        self.blocks.last()
    }
    pub fn get_transactions(&self) -> &Vec<Transaction> {
        return &self.pending_transactions;
    }
}
