use std::collections::{HashMap, HashSet};

use crate::{block::Block, crypto::hash_utils::HashResult, transaction::Transaction};

use super::database::Database;

pub struct InMemoryDatabase {
    blocks: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    transactions: HashMap<HashResult, Transaction>,
    unspent_outputs: HashSet<(HashResult, usize)>,
    address_to_txs: HashMap<Vec<u8>, Vec<HashResult>>,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        InMemoryDatabase {
            blocks: vec![],
            pending_transactions: Vec::new(),
            transactions: HashMap::new(),
            unspent_outputs: HashSet::new(),
            address_to_txs: HashMap::new(),
        }
    }
}

impl Default for InMemoryDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl Database for InMemoryDatabase {
    fn insert_block(&mut self, block: Block) {
        let block_height = self.block_height();
        log::debug!("Block ({}): {:?}", block_height, block);

        for tx in block.transactions.iter() {
            self.add_transaction(tx.hash(), tx.clone());
        }

        self.blocks.push(block);

        if block_height == 0 {
            log::info!("★★★ GENESIS BLOCK ★★★");
        } else {
            log::info!("Block added at height {}.", block_height);
        }
    }

    fn get_blocks(&self) -> &[Block] {
        &self.blocks
    }

    fn block_height(&self) -> usize {
        self.blocks.len()
    }

    fn head(&self) -> Option<&Block> {
        self.blocks.last()
    }

    fn add_utxo(&mut self, tx_hash: HashResult, output_index: usize) {
        self.unspent_outputs.insert((tx_hash, output_index));
    }

    fn remove_utxo(&mut self, tx_hash: &HashResult, output_index: usize) {
        self.unspent_outputs.remove(&(*tx_hash, output_index));
    }

    fn is_utxo(&self, tx_hash: &HashResult, output_index: usize) -> bool {
        self.unspent_outputs.contains(&(*tx_hash, output_index))
    }

    fn add_transaction(&mut self, tx_hash: HashResult, transaction: Transaction) {
        self.transactions.insert(tx_hash, transaction);
    }

    fn remove_transaction(&mut self, tx_hash: HashResult) -> Option<Transaction> {
        self.transactions.remove(&tx_hash)
    }

    fn get_transaction(&mut self, tx_hash: HashResult) -> Option<&Transaction> {
        self.transactions.get(&tx_hash)
    }

    fn map_address_to_transaction_hash(&mut self, address: &[u8], tx_hash: HashResult) {
        if let Some(hashes) = self.address_to_txs.get_mut(address) {
            hashes.push(tx_hash);
        } else {
            self.address_to_txs.insert(address.to_vec(), vec![tx_hash]);
        }
    }

    fn get_transaction_hashes(&mut self, address: &[u8]) -> &[HashResult] {
        log::warn!("{:#?}", self.address_to_txs);

        if let Some(txs) = self.address_to_txs.get(address) {
            txs
        } else {
            &[]
        }
    }

    fn add_pending_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    fn get_pending_transactions(&self) -> &[Transaction] {
        &self.pending_transactions
    }

    fn clear_pending_transactions(&mut self) {
        self.pending_transactions.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block::Block,
        database::{database::Database, InMemoryDatabase},
    };

    #[test]
    fn test_insert_block() {
        let mut in_memory_db = InMemoryDatabase::new();
        in_memory_db.insert_block(Block::default());
        assert_eq!(in_memory_db.blocks.len(), 1);
        in_memory_db.insert_block(Block::default());
        assert_eq!(in_memory_db.blocks.len(), 2);
    }
}
