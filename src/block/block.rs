use crate::crypto::hash_utils::HashResult;

use super::{BlockHeader, Transaction};

#[derive(Default)]
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
}
