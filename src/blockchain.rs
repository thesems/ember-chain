use crate::{
    block::{block_header::BlockHeader, transaction::Transaction, Block},
    constants::{BLOCK_ADJUSTMENT_FREQUENCY, BLOCK_TIME_SECS, START_DIFFICULTY_BIT},
    hash_utils::HashResult,
    merkle_tree::generate_merkle_root,
    mining::miner::Miner,
    server::Server,
};
use crossbeam::channel::{select, unbounded, Receiver};
use rand::prelude::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub struct Blockchain {
    block_uids: HashMap<HashResult, usize>,
    blocks: Vec<Block>,
    pending_transactions: Arc<Mutex<Vec<Transaction>>>,
    running: bool,
    server: Arc<Server>,
    tx_recv: Receiver<Transaction>,
    miner: Miner,
}

impl Default for Blockchain {
    fn default() -> Self {
        let (tx_sender, tx_recv) = unbounded::<Transaction>();
        Self {
            block_uids: HashMap::new(),
            blocks: vec![],
            pending_transactions: Arc::new(Mutex::new(vec![])),
            running: true,
            server: Arc::new(Server::new(tx_sender)),
            tx_recv,
            miner: Miner::new(START_DIFFICULTY_BIT),
        }
    }
}

impl Blockchain {
    pub fn run(&mut self) {
        self.add_genesis_block();

        let server = self.server.clone();
        let server_handle = thread::spawn(move || {
            server.listen();
        });

        while self.running {
            let block = self.mine_or_receive();
            self.add_block(block);
            if self.blocks.len() % BLOCK_ADJUSTMENT_FREQUENCY == 0 {
                self.miner.adjust_difficulty();
            }
        }

        server_handle.join().unwrap();
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
    fn mine_or_receive(&mut self) -> Block {
        let txs = self.pending_transactions.lock().unwrap().clone();
        let tx_hashes = txs.iter().map(|x| x.hash()).collect();

        let merkle_root = generate_merkle_root(tx_hashes);
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
                if let Some(block) = self.miner.mine(
                    merkle_root,
                    &txs,
                    cancel_mine_rx,
                    &mut hash_count,
                    self.head().unwrap().hash,
                ) {
                    mine_sender.send(block).unwrap();
                }
            });
            s.spawn(|| {
                let block = self.recv_block();
                net_sender.send(block).unwrap();
            });
            s.spawn(|| loop {
                match self.tx_recv.recv() {
                    Ok(tx) => {
                        self.pending_transactions.lock().unwrap().push(tx);
                    }
                    Err(_e) => {
                        todo!()
                    }
                }
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
        });

        self.block_uids.insert(final_block.hash, self.blocks.len());
        self.miner.add_mining_time(start.elapsed(), hash_count);
        final_block
    }
}
