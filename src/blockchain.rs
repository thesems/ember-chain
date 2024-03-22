use crate::{
    block::Block, block_header::BlockHeader, hash_utils::HashResult,
    merkle_tree::generate_merkle_root, pow_utils::proof_of_work, transaction::Transaction,
};
use crossbeam::channel::{select, unbounded, Receiver, Sender};
use rand::prelude::*;
use std::{
    collections::{HashMap, VecDeque},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const BLOCK_TIME_SECS: u64 = 10;
const BLOCK_ADJUSTMENT_FREQUENCY: usize = 100;
const START_DIFFICULTY_BIT: u8 = 20;


pub struct Blockchain {
    block_uids: HashMap<HashResult, usize>,
    blocks: Vec<Block>,
    difficulty: u8,
    hash_per_secs: f64,
    last_mining_times: VecDeque<f64>,
    pending_transactions: Vec<Transaction>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self {
            block_uids: HashMap::new(),
            blocks: vec![],
            difficulty: START_DIFFICULTY_BIT,
            hash_per_secs: 0.0,
            last_mining_times: VecDeque::with_capacity(BLOCK_ADJUSTMENT_FREQUENCY),
            pending_transactions: vec![],
        }
    }
}

impl Blockchain {
    pub fn run(&mut self) {
        self.add_genesis_block();
        loop {
            let tx_hashes = vec![];
            let merkle_root = generate_merkle_root(tx_hashes);
            let block = self.mine_or_receive(merkle_root);
            self.add_block(block);
            self.adjust_difficulty();
        }
    }
    fn add_genesis_block(&mut self) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");

        self.blocks.push(Block {
            header: BlockHeader::from([0u8; 32], [0u8; 32], 0, time.as_secs()),
            transactions: vec![],
            hash: [0u8; 32],
        });
        log::info!("Genesis block added.");
    }
    fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
        log::info!("Added a new block with height {}.", self.block_height());
        log::debug!("Average hash per second: {:.2}", self.hash_per_secs);
    }
    fn head(&self) -> Option<&Block> {
        self.blocks.last()
    }
    fn block_height(&self) -> usize {
        self.blocks.len()
    }

    fn recv_block(&self) -> Block {
        let mut rng = rand::thread_rng();
        let wait_time = rng.gen::<f32>().max(0.8);
        thread::sleep(Duration::new(
            (wait_time * BLOCK_TIME_SECS as f32) as u64,
            0,
        ));

        let mut previous_block_hash = [0u8; 32];
        if let Some(previous_block) = self.head() {
            previous_block_hash = previous_block.hash;
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut block_header = BlockHeader::from(previous_block_hash, [0u8; 32], 0, timestamp);

        block_header.merkle_root = [0u8; 32];
        block_header.nonce = 0;
        Block::build(block_header, vec![], [0u8; 32])
    }

    fn mine_or_receive(&mut self, merkle_root: HashResult) -> Block {
        let txs = self.pending_transactions.clone();
        let mut final_block = Block::build(
            BlockHeader::from([0u8; 32], [0u8; 32], 0, 0),
            vec![],
            [0u8; 32],
        );

        let start = Instant::now();
        let mut hash_count = 0;
        let (mine_sender, mine_recv) = unbounded::<Block>();
        let (net_sender, net_recv) = unbounded::<Block>();

        thread::scope(|s| {
            let (cancel_mine_tx, cancel_mine_rx) = unbounded::<()>();
            s.spawn(|| {
                if let Some(block) = self.mine(merkle_root, txs, cancel_mine_rx, &mut hash_count) {
                    mine_sender.send(block).unwrap();
                }
            });
            s.spawn(|| {
                println!("pre: {}", self.block_height());
                let block = self.recv_block();
                net_sender.send(block).unwrap();
                println!("post: {}", self.block_height());
            });
            final_block = select! {
                recv(mine_recv) -> my_block => {
                   log::info!("You succesfully mined a block!");
                   my_block.unwrap()
                }
                recv(net_recv) -> other_block => {
                    log::info!("A participant has mined a block!");
                    let _ = cancel_mine_tx.send(());
                    other_block.unwrap()
                }
            };
            println!("post sele: {}", self.block_height());
        });

        self.block_uids.insert(final_block.hash, self.blocks.len());
        self.add_mining_time(start.elapsed(), hash_count);
        final_block
    }
    fn mine(
        &self,
        merkle_root: HashResult,
        transactions: Vec<Transaction>,
        cancel_mine_rx: Receiver<()>,
        hash_count: &mut u64,
    ) -> Option<Block> {
        let previous_block_hash = match self.head() {
            Some(previous_block) => previous_block.hash,
            None => [0u8; 32],
        };
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut block_header =
            BlockHeader::from(merkle_root, previous_block_hash, self.difficulty, timestamp);
        if let Some(block_hash) = proof_of_work(self.difficulty, &mut block_header, cancel_mine_rx, hash_count)
        {
            let block = Block::build(block_header, transactions, block_hash);
            return Some(block);
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
            log::info!(
                "Adjust difficulty from {} to {}.",
                previous_difficulty,
                self.difficulty
            );
        }

        log::info!(
            "Average block time during last {} blocks was {} seconds.",
            BLOCK_ADJUSTMENT_FREQUENCY,
            avg_mining_time
        );
    }
    fn add_mining_time(&mut self, duration: Duration, counter: u64) {
        if self.last_mining_times.len() >= BLOCK_ADJUSTMENT_FREQUENCY {
            self.last_mining_times.pop_front();
        }
        self.last_mining_times.push_back(duration.as_secs_f64());
        self.hash_per_secs = (counter as f64 / duration.as_millis() as f64) * 1000.0;
    }
}
