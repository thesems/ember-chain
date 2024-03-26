use crate::crypto::hash_utils::HashResult;

use super::{BlockHeader, Transaction};

pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: HashResult,
}
impl Default for Block {
    fn default() -> Block {
        Block {
            header: BlockHeader::default(),
            transactions: vec![],
            hash: [0u8; 32]
        }
    }
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
