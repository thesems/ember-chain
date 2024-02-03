use crate::{block_header::BlockHeader, transaction::Transaction};

pub struct Block {
    pub size: u32,
    pub header: BlockHeader,
    pub tx_cnt: u16,
    pub transactions: Vec<Transaction>,
}
impl Block {
    pub fn build(header: BlockHeader, transactions: Vec<Transaction>) -> Block {
        let mut tx_size = 0;
        for tx in &transactions {
            tx_size += tx.size();
        }
        let size = (header.size() + 1 + tx_size) as u32;
        Block {
            size,
            header,
            tx_cnt: transactions.len() as u16,
            transactions,
        }
    }
}
