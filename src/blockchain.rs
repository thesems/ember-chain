use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use chrono::DateTime;
use crossbeam::channel::{select, unbounded, Receiver};
use tokio::runtime::Runtime;

use crate::{
    block::{block_header::BlockHeader, Block},
    config::models::Config,
    crypto::{
        account::{Account, AccountError},
        merkle_tree::generate_merkle_root,
    },
    database::{database::DatabaseType, InMemoryDatabase},
    mining::miner::Miner,
    network::node::Network,
    transaction::Transaction,
    types::Satoshi,
};

pub struct Blockchain {
    // dependencies
    database: Arc<Mutex<DatabaseType>>,
    account: Arc<Account>,
    miner: Miner,
    // other
    config: Config,
    running: bool,
    transactions_rx: Arc<Mutex<Receiver<Transaction>>>,
    current_block_reward: Satoshi,
    blocks_announce_tx_rx: (
        crossbeam::channel::Sender<Block>,
        crossbeam::channel::Receiver<Block>,
    ),
    blocks_publish_tx_rx: (
        crossbeam::channel::Sender<Block>,
        crossbeam::channel::Receiver<Block>,
    ),
}

#[derive(Debug)]
pub enum BlockchainError {
    AccountError,
}
impl From<AccountError> for BlockchainError {
    fn from(_: AccountError) -> Self {
        BlockchainError::AccountError
    }
}

impl Blockchain {
    pub fn new(config: Config) -> Result<Self, BlockchainError> {
        let (transactions_tx, transactions_rx) = unbounded::<Transaction>();
        let account = Arc::new(Account::load_or_create(config.account.clone())?);
        let database = Arc::new(Mutex::new(InMemoryDatabase::default()));

        Ok(Self {
            running: true,
            database,
            miner: Miner::new(config.mining.clone(), account.clone()),
            account,
            transactions_rx: Arc::new(Mutex::new(transactions_rx)),
            current_block_reward: config.mining.mining_reward,
            blocks_announce_tx_rx: unbounded::<Block>(),
            blocks_publish_tx_rx: unbounded::<Block>(),
            config,
        })
    }
    pub fn run(&mut self) {
        let transactions_rx = self.transactions_rx.clone();
        let mut pen_txs = self
            .database
            .lock()
            .unwrap()
            .get_pending_transactions()
            .to_vec();

        thread::scope(|s| {
            s.spawn(|| loop {
                match transactions_rx.lock().unwrap().recv() {
                    Ok(tx) => {
                        pen_txs.push(tx);
                    }
                    Err(_e) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            });
            let db = self.database.clone();
            let port = self.config.network.port;
            let seed_list = self.config.network.seed_list.clone();
            let network = Arc::new(Network::new(
                port,
                seed_list,
                db,
                self.blocks_announce_tx_rx.0.clone(),
                self.blocks_publish_tx_rx.1.clone(),
            ));

            self.add_genesis_block();

            s.spawn(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let network_cloned = network.clone();
                    let hdl = tokio::spawn(async move {
                        if network_cloned.start_network_node().await.is_err() {
                            log::error!("☠☠ network node crashed ☠☠");
                        }
                    });
                    if let Err(err) = network.start_sync().await {
                        log::error!("Synchronization failed: {}", err);
                    }

                    let network_cloned = network.clone();
                    tokio::spawn(async move { network_cloned.wait_on_publish_block().await });

                    hdl.await.unwrap();
                    log::warn!("async runtime ended");
                });
            });

            while self.running {
                let block = self.get_next_block();

                if !block.verify(&self.database) {
                    log::warn!(
                        "☠☠ Invalid block ({}) ☠☠.",
                        hex::encode(block.hash.get(..5).unwrap())
                    );
                    let mut db = self.database.lock().unwrap();
                    for tx in block.transactions.iter() {
                        db.remove_transaction(tx.hash());
                    }
                    continue;
                }

                let mut db = self.database.lock().unwrap();
                db.insert_block(block);
                if db.block_height() % self.config.mining.block_adjustment_interval == 0 {
                    self.miner.adjust_difficulty();
                }
            }
        });
    }
    fn add_genesis_block(&self) {
        let ts = DateTime::parse_from_rfc3339("2009-01-03T18:15:05-00:00")
            .unwrap()
            .timestamp() as u64;
        let transactions = vec![Transaction::create_coinbase(0, [0u8; 32].to_vec())];
        let tx_hashes = transactions.iter().map(|x| x.hash()).collect();
        let merkle_root = generate_merkle_root(tx_hashes);
        let header = BlockHeader::from(merkle_root, [0u8; 32], 0, ts, 0);
        let block = Block {
            hash: header.finalize(),
            header,
            transactions,
        };
        self.database.lock().unwrap().insert_block(block);
    }
    fn get_next_block(&mut self) -> Block {
        let start = Instant::now();
        let last_block = self.database.lock().unwrap().head().unwrap().clone();

        let (mining_tx, mining_rx) = unbounded::<Block>();
        let (mining_cancel_tx, mining_cancel_rx) = unbounded::<()>();
        let (net_tx, net_rx) = unbounded::<Block>();
        let (net_cancel_tx, net_cancel_rx) = unbounded::<()>();

        let mut hash_count = 0;
        let mut final_block = Block::default();

        thread::scope(|s| {
            s.spawn(|| {
                let block: Option<Block>;
                (block, hash_count) = self.miner.mine(
                    &self.database,
                    mining_cancel_rx,
                    last_block.hash,
                    self.current_block_reward,
                    self.config.simulation.fake_mining,
                );

                if let Some(block) = block {
                    mining_tx.send(block.clone()).unwrap();
                }
            });

            s.spawn(|| loop {
                select! {
                    recv(self.blocks_announce_tx_rx.1) -> block => {
                        let block = block.unwrap();
                        let last_block = self.database.lock().unwrap().head().unwrap().clone();

                        if block.header.previous_block_hash == last_block.hash {
                            net_tx.send(block.clone()).unwrap();
                            return;
                        } else {
                            log::warn!("Block ({}) does not fit onto latest block ({}).",
                                hex::encode(block.header.previous_block_hash.get(..5).unwrap()),
                                hex::encode(last_block.hash.get(..5).unwrap())
                            );
                        }
                    }
                    recv(net_cancel_rx) -> _ => {
                        return;
                    }
                }
            });

            final_block = select! {
                recv(mining_rx) -> my_block => {
                    _ = net_cancel_tx.send(());
                    let block = my_block.unwrap();
                    self.blocks_publish_tx_rx.0.send(block.clone()).unwrap();
                    log::info!("★★★ You successfully mined a block ({})! ★★★", hex::encode(block.hash.get(..5).unwrap()));
                    block
                }
                recv(net_rx) -> other_block => {
                    _ = mining_cancel_tx.send(());
                    let block = other_block.unwrap();
                    log::info!("A participant has mined a block! ({})!", hex::encode(block.hash.get(..5).unwrap()));
                    block
                }
            };
        });

        self.miner.add_mining_time(start.elapsed(), hash_count);
        final_block
    }
}
