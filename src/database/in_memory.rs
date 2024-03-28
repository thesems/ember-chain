use std::collections::{HashMap, HashSet};

use crate::{block::Block, crypto::hash_utils::HashResult, transaction::Transaction};

use super::database::Database;

pub struct InMemoryDatabase {
    blocks: Vec<Block>,
    transactions: HashMap<HashResult, Transaction>,
    unspent_outputs: HashSet<(String, usize)>,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        InMemoryDatabase {
            blocks: vec![],
            transactions: HashMap::new(),
            unspent_outputs: HashSet::new(),
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

    fn add_utxo(&mut self, tx_hash: String, output_index: usize) {
        self.unspent_outputs.insert((tx_hash, output_index));
    }

    fn remove_utxo(&mut self, tx_hash: &str, output_index: usize) {
        self.unspent_outputs
            .remove(&(tx_hash.to_string(), output_index));
    }

    fn is_utxo(&self, tx_hash: &str, output_index: usize) -> bool {
        self.unspent_outputs
            .contains(&(tx_hash.to_string(), output_index))
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
