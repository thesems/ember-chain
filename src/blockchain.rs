use crate::{block::Block, block_header::BlockHeader, hash_utils::HashResult, pow_utils::proof_of_work};
use chrono::{NaiveDate, NaiveDateTime};
use rand::prelude::*;
use std::{
    collections::{HashMap, VecDeque},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const BLOCK_TIME_SECS: u64 = 10;
const BLOCK_ADJUSTMENT_FREQUENCY: usize = 100;
const START_DIFFICULTY_BIT: u8 = 20;
const TEST_MINING_PROBABILITY: f32 = 1.0;

pub struct Blockchain {
    block_uids: HashMap<HashResult, usize>,
    blocks: Vec<Block>,
    difficulty: u8,
    hash_per_secs: f64,
    last_mining_times: VecDeque<f64>,
}
impl Blockchain {
    pub fn new() -> Self {
        Self {
            block_uids: HashMap::new(),
            blocks: vec![],
            difficulty: START_DIFFICULTY_BIT,
            hash_per_secs: 0.0,
            last_mining_times: VecDeque::with_capacity(BLOCK_ADJUSTMENT_FREQUENCY),
        }
    }
    fn add_genesis_block(&mut self) {
        let date_time: NaiveDateTime = NaiveDate::from_ymd_opt(2017, 11, 12)
            .unwrap()
            .and_hms_opt(17, 33, 44)
            .unwrap();
        self.blocks.push(Block {
            header: BlockHeader::from([0u8; 32], [0u8; 32], 0, date_time.timestamp() as u64),
            transactions: vec![],
        });
        println!("Genesis block added.");
    }
    pub fn run(&mut self) {
        println!("Blockchain started.");
        self.add_genesis_block();
        loop {
            let transactions = vec![];
            let merkle_root = [0u8; 32];
            let block_header = self.mine_or_receive(merkle_root);
            let block = Block::build(block_header, transactions);
            self.add_block(block);
            self.adjust_difficulty();
        }
    }
    fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
        println!("Added a new block.");
        println!("Average hash per second: {:.2}", self.hash_per_secs);
    }
    fn head(&self) -> Option<&Block> {
        self.blocks.last()
    }
    fn mine_or_receive(&mut self, merkle_root: HashResult) -> BlockHeader {
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
        if let Some(previous_block) = self.head() {
            previous_block_hash = previous_block.header.merkle_root;
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut block_header = BlockHeader::from(previous_block_hash, [0u8; 32], 0, timestamp);

        block_header.merkle_root = [0u8; 32];
        block_header.nonce = 10;
        block_header
    }
    fn mine(&mut self, _merkle_root: HashResult) -> Option<(HashResult, BlockHeader)> {
        let start = Instant::now();
        let mut previous_block_hash = [0u8; 32];
        if let Some(previous_block) = self.head() {
            previous_block_hash = previous_block.header.merkle_root;
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut block_header = BlockHeader::from(previous_block_hash, [0u8; 32], 0, timestamp);
        if let Some(block_hash) = proof_of_work(self.difficulty, &mut block_header) {
            self.add_mining_time(start.elapsed(), block_header.nonce);
            return Some((block_hash, block_header));
        }
        None
    }
    pub fn adjust_difficulty(&mut self) {
        if self.blocks.len() % BLOCK_ADJUSTMENT_FREQUENCY != 0 {
            return;
        }

        let avg_mining_time =
            self.last_mining_times.iter().sum::<f64>() / self.last_mining_times.len() as f64;
        let previous_difficulty = self.difficulty;

        if avg_mining_time < BLOCK_TIME_SECS as f64 * 0.8 {
            self.difficulty += 1;
        } else if avg_mining_time > BLOCK_TIME_SECS as f64 * 1.2 {
            self.difficulty -= 1;
        }

        if previous_difficulty != self.difficulty {
            println!(
                "Adjust difficulty from {} to {}.",
                previous_difficulty, self.difficulty
            );
        }

        println!(
            "Average block time during last {} blocks was {} seconds.",
            BLOCK_ADJUSTMENT_FREQUENCY, avg_mining_time
        );
    }
    fn add_mining_time(&mut self, duration: Duration, nonce: u32) {
        if self.last_mining_times.len() >= BLOCK_ADJUSTMENT_FREQUENCY {
            self.last_mining_times.pop_front();
        }
        self.last_mining_times.push_back(duration.as_secs_f64());
        self.hash_per_secs = nonce as f64 / duration.as_millis() as f64;
    }
}
