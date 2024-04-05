use crate::{
    api::server::Server,
    block::{block_header::BlockHeader, Block},
    config::models::Config,
    crypto::account::{Account, AccountError},
    database::{database::DatabaseType, InMemoryDatabase},
    mining::miner::Miner,
    transaction::Transaction,
    types::Satoshi,
};
use crossbeam::channel::{select, unbounded, Receiver};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub struct Blockchain {
    // dependencies
    database: Arc<Mutex<DatabaseType>>,
    server: Arc<Server>,
    account: Arc<Account>,
    miner: Miner,
    // other
    config: Config,
    running: bool,
    transactions_rx: Arc<Mutex<Receiver<Transaction>>>,
    current_block_reward: Satoshi,
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
        let account = Arc::new(Account::new()?);
        let database = Arc::new(Mutex::new(InMemoryDatabase::default()));
        Ok(Self {
            running: true,
            server: Arc::new(Server::new(transactions_tx, database.clone())),
            database,
            miner: Miner::new(config.mining.clone(), account.clone()),
            account,
            transactions_rx: Arc::new(Mutex::new(transactions_rx)),
            current_block_reward: config.mining.mining_reward,
            config,
        })
    }
    pub fn run(&mut self) {
        let server = self.server.clone();
        let server_handle = thread::spawn(move || {
            server.listen();
        });

        let tx_recv = self.transactions_rx.clone();
        let mut pen_txs = self.database.lock().unwrap().get_pending_transactions().to_vec();

        thread::scope(|s| {
            s.spawn(|| loop {
                match tx_recv.lock().unwrap().recv() {
                    Ok(tx) => {
                        pen_txs.push(tx);
                    }
                    Err(_e) => {
                        todo!()
                    }
                }
            });

            self.add_genesis_block();
            while self.running {
                let block = self.get_next_block();

                if !block.verify(self.current_block_reward, &self.database) {
                    log::warn!("☠☠ Invalid block ☠☠ {:?}.", block.hash);
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

            server_handle.join().unwrap();
        });
    }
    fn add_genesis_block(&mut self) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");

        self.database.lock().unwrap().insert_block(Block {
            header: BlockHeader::from([0u8; 32], [0u8; 32], 0, time.as_secs()),
            transactions: vec![Transaction::create_coinbase(
                self.config.mining.mining_reward,
                [0u8; 32].to_vec(),
            )],
            hash: [0u8; 32],
        });
    }
    fn recv_block(&self, net_cancel_recv: Receiver<()>) -> Block {
        let mut sleep_time = 0;
        loop {
            match net_cancel_recv.try_recv() {
                Ok(_) => {
                    break;
                }
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
    fn get_next_block(&mut self) -> Block {
        let start = Instant::now();
        let txs = self.database.lock().unwrap().get_pending_transactions().to_vec();

        let prev_block_hash = match self.database.lock().unwrap().head() {
            Some(block) => block.hash,
            None => [0u8; 32],
        };
        let mut final_block = Block::default();
        let mut hash_count = 0;

        let (mining_tx, mining_rx) = unbounded::<Block>();
        let (mining_cancel_tx, mining_cancel_rx) = unbounded::<()>();
        let (net_tx, net_rx) = unbounded::<Block>();
        let (net_cancel_tx, net_cancel_rx) = unbounded::<()>();

        thread::scope(|s| {
            s.spawn(|| {
                if let Some(block) = self.miner.mine(
                    &self.database,
                    &txs,
                    mining_cancel_rx,
                    &mut hash_count,
                    prev_block_hash,
                    self.current_block_reward,
                ) {
                    mining_tx.send(block).unwrap();
                }
            });
            s.spawn(|| {
                let block = self.recv_block(net_cancel_rx);
                net_tx.send(block).unwrap();
            });

            final_block = select! {
                recv(mining_rx) -> my_block => {
                    log::info!("★★★ You successfully mined a block! ★★★");
                    net_cancel_tx.send(()).unwrap();
                    my_block.unwrap()
                }
                recv(net_rx) -> other_block => {
                    log::info!("A participant has mined a block!");
                    mining_cancel_tx.send(()).unwrap();
                    other_block.unwrap()
                }
            };
        });

        self.miner.add_mining_time(start.elapsed(), hash_count);
        final_block
    }
}
