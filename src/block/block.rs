use crate::hash_utils::HashResult;

use super::{BlockHeader, Transaction};

pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: HashResult,
}
impl Block {
    pub fn build(header: BlockHeader, transactions: Vec<Transaction>, hash: HashResult) -> Block {
        Block {
            header,
            transactions,
            hash,
        }
    }
}