use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::{
    crypto::hash_utils::HashResult, database::database::DatabaseType, transaction::Transaction,
};

use super::BlockHeader;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
    pub fn verify(&self, database: &Arc<Mutex<DatabaseType>>) -> bool {
        if let Some(block) = database.lock().unwrap().head() {
            if block.hash != self.header.previous_block_hash {
                return false;
            }
        } else {
            panic!("No previous block found to verify.")
        }

        for tx in self.transactions.iter() {
            if !tx.verify(self.header.reward, database, &self.transactions)
                || !tx.verify_inputs(database)
            {
                return false;
            }
        }
        true
    }
    pub fn get_hash_as_string(&self, clipped: bool) -> String {
        if clipped {
            hex::encode(self.hash.get(..5).unwrap())
        } else {
            hex::encode(self.hash)
        }
    }
}
