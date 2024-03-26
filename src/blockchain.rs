use crate::{
    api::server::Server, block::{block_header::BlockHeader, transaction::Transaction, Block}, config::models::Config, crypto::{hash_utils::HashResult, merkle_tree::generate_merkle_root}, database::{database::Database, InMemoryDatabase}, mining::miner::Miner
};
use crossbeam::channel::{select, unbounded, Receiver};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub struct Blockchain {
    config: Config,
    block_uids: HashMap<HashResult, usize>,
    database: Arc<Mutex<dyn Database + Send + Sync>>,
    pending_transactions: Arc<Mutex<Vec<Transaction>>>,
    running: bool,
    server: Arc<Server>,
    tx_recv: Arc<Mutex<Receiver<Transaction>>>,
    miner: Miner,
}

impl Blockchain {
    pub fn new(config: Config) -> Self {
        let (tx_sender, tx_recv) = unbounded::<Transaction>();
        Self {
            block_uids: HashMap::new(),
            database: Arc::new(Mutex::new(InMemoryDatabase::new())),
            pending_transactions: Arc::new(Mutex::new(vec![])),
            running: true,
            server: Arc::new(Server::new(tx_sender)),
            tx_recv: Arc::new(Mutex::new(tx_recv)),
            miner: Miner::new(config.mining.clone()),
            config,
        }
    }
    pub fn run(&mut self) {
        let server = self.server.clone();
        let server_handle = thread::spawn(move || {
            server.listen();
        });

        let tx_recv = self.tx_recv.clone();
        let pen_txs = self.pending_transactions.clone();

        thread::scope(|s| {
            s.spawn(|| loop {
                match tx_recv.lock().unwrap().recv() {
                    Ok(tx) => {
                        pen_txs.lock().unwrap().push(tx);
                    }
                    Err(_e) => {
                        todo!()
                    }
                }
            });

            self.add_genesis_block();
            while self.running {
                let block = self.mine_or_receive();
                let mut db = self.database.lock().unwrap();
                db.insert_block(block);
                if db.block_height() % self.config.mining.block_adjustment_frequency == 0 {
                    self.miner.adjust_difficulty();
                }
            }

            server_handle.join().unwrap();
        });
    }
    fn add_genesis_block(&mut self) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");

        self.database.lock().unwrap().insert_block(Block {
            header: BlockHeader::from([0u8; 32], [0u8; 32], 0, time.as_secs()),
            transactions: vec![],
            hash: [0u8; 32],
        });
    }
    fn recv_block(&self, net_cancel_recv: Receiver<()>) -> Block {
        let mut sleep_time = 0;
        loop {
            match net_cancel_recv.try_recv() {
                Ok(_) => {
                    break;
                },
                Err(_) => {
                    sleep_time += 1;
                    if sleep_time > self.config.mining.block_time_secs {
                        break;
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }

        let mut previous_block_hash = [0u8; 32];
        if let Some(previous_block) = self.database.lock().unwrap().head() {
            previous_block_hash = previous_block.hash;
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut block_header = BlockHeader::from(previous_block_hash, [0u8; 32], 0, timestamp);

        block_header.merkle_root = [0u8; 32];
        block_header.nonce = 0;
        Block::new(block_header, vec![], [0u8; 32])
    }
    fn mine_or_receive(&mut self) -> Block {
        let start = Instant::now();
        let txs = self.pending_transactions.lock().unwrap().clone();
        let tx_hashes = txs.iter().map(|x| x.hash()).collect();

        let prev_block_hash = match self.database.lock().unwrap().head() {
            Some(block) => block.hash,
            None => [0u8; 32],
        };
        let merkle_root = generate_merkle_root(tx_hashes);
        let mut final_block = Block::default();
        let mut hash_count = 0;

        let (mine_sender, mine_recv) = unbounded::<Block>();
        let (net_sender, net_recv) = unbounded::<Block>();
        let (net_cancel_sender, net_cancel_recv) = unbounded::<()>();

        thread::scope(|s| {
            let (cancel_mine_tx, cancel_mine_rx) = unbounded::<()>();
            s.spawn(|| {
                if let Some(block) = self.miner.mine(
                    merkle_root,
                    &txs,
                    cancel_mine_rx,
                    &mut hash_count,
                    prev_block_hash,
                ) {
                    mine_sender.send(block).unwrap();
                }
            });
            s.spawn(|| {
                let block = self.recv_block(net_cancel_recv);
                net_sender.send(block).unwrap();
            });

            final_block = select! {
                recv(mine_recv) -> my_block => {
                   log::info!("★★★ You successfully mined a block! ★★★");
                    net_cancel_sender.send(()).unwrap();
                   my_block.unwrap()
                }
                recv(net_recv) -> other_block => {
                    log::info!("A participant has mined a block!");
                    cancel_mine_tx.send(()).unwrap();
                    other_block.unwrap()
                }
            };
        });

        self.block_uids.insert(
            final_block.hash,
            self.database.lock().unwrap().block_height(),
        );
        self.miner.add_mining_time(start.elapsed(), hash_count);
        final_block
    }
}
