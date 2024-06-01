use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::{
    crypto::hash_utils::HashResult, database::database::DatabaseType, transaction::Transaction,
};

use super::BlockHeader;

#[derive(Default, Debug, Serialize, Deserialize)]
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
        for tx in self.transactions.iter() {
            if !tx.verify(self.header.reward, database, &self.transactions)
                || !tx.verify_inputs(database)
            {
                return false;
            }
        }
        true
    }
}
