use crate::{block_header::BlockHeader, transaction::Transaction};

pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}
impl Block {
    pub fn build(header: BlockHeader, transactions: Vec<Transaction>) -> Block {
        Block {
            header,
            transactions,
        }
    }
}
