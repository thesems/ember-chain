use std::sync::{Arc, Mutex};

use crate::{
    crypto::hash_utils::HashResult, database::database::DatabaseType, transaction::Transaction,
    types::Satoshi,
};

use super::BlockHeader;

#[derive(Default, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: HashResult,
}
impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>, hash: HashResult) -> Block {
        Block {
            header,
            transactions,
            hash,
        }
    }
    pub fn verify(
        &self,
        current_block_reward: Satoshi,
        database: &Arc<Mutex<DatabaseType>>,
    ) -> bool {
        for tx in self.transactions.iter() {
            if !tx.verify(current_block_reward, database) || !tx.verify_inputs(database) {
                return false;
            }
        }
        true
    }
}
