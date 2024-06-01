use std::collections::{HashMap, HashSet};

use crate::crypto::hash_utils::Address;
use crate::types::Satoshi;
use crate::{block::Block, crypto::hash_utils::HashResult, transaction::Transaction};

use super::database::Database;

pub struct InMemoryDatabase {
    blocks: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    transactions: HashMap<HashResult, Transaction>,
    unspent_outputs: HashSet<(HashResult, u32)>,
    address_to_txs: HashMap<Vec<u8>, HashSet<HashResult>>,
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

        for tx in block.transactions.iter() {
            let tx_hash = tx.hash();
            self.add_transaction(tx_hash, tx.clone());

            // remove spent utxo's
            for input in tx.inputs.iter().filter(|x| x.utxo_tx_hash != [0u8; 32]) {
                self.remove_utxo(&input.utxo_tx_hash, input.utxo_output_index);
            }

            // add new unspent outputs
            for output_index in 0..tx.outputs.len() {
                self.add_utxo(tx_hash, output_index as u32);
            }

            // update transaction mappings
            self.map_address_to_transaction_hash(&tx.sender, tx_hash);
            for output in tx.outputs.iter() {
                self.map_address_to_transaction_hash(&output.receiver, tx_hash);
            }
        }

        if block_height == 0 {
            log::info!("★★★ GENESIS BLOCK ★★★");
        } else {
            log::info!(
                "Block added at height {} with {} transactions.",
                block_height,
                block.transactions.len()
            );
        }

        self.blocks.push(block);
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

    fn add_utxo(&mut self, tx_hash: HashResult, output_index: u32) {
        self.unspent_outputs.insert((tx_hash, output_index));
    }

    fn remove_utxo(&mut self, tx_hash: &HashResult, output_index: u32) {
        if !self.unspent_outputs.remove(&(*tx_hash, output_index)) {
            log::warn!("Could not remove a UTXO!");
        }
    }

    fn is_utxo(&self, tx_hash: &HashResult, output_index: u32) -> bool {
        self.unspent_outputs.contains(&(*tx_hash, output_index))
    }

    fn get_utxo(&self, public_key: &Address) -> Vec<(HashResult, u32, Satoshi)> {
        let mut unspent_outputs: Vec<(HashResult, u32, u64)> = vec![];
        if let Some(tx_hashes) = self.address_to_txs.get(public_key) {
            for tx_hash in tx_hashes {
                if let Some(tx) = self.get_transaction(tx_hash) {
                    for (output_idx, output) in tx.outputs.iter().enumerate() {
                        if &output.receiver == public_key
                            && self.is_utxo(tx_hash, output_idx as u32)
                        {
                            unspent_outputs.push((*tx_hash, output_idx as u32, output.value));
                        }
                    }
                }
            }
        };
        unspent_outputs
    }

    fn add_transaction(&mut self, tx_hash: HashResult, transaction: Transaction) {
        log::debug!(
            "Added a transaction: {}",
            hex::encode(transaction.hash()).get(..6).unwrap()
        );
        self.transactions.insert(tx_hash, transaction);
    }

    fn remove_transaction(&mut self, tx_hash: HashResult) -> Option<Transaction> {
        self.transactions.remove(&tx_hash)
    }

    fn get_transaction(&self, tx_hash: &HashResult) -> Option<&Transaction> {
        self.transactions.get(tx_hash)
    }

    fn map_address_to_transaction_hash(&mut self, address: &[u8], tx_hash: HashResult) {
        if let Some(hashes) = self.address_to_txs.get_mut(address) {
            hashes.insert(tx_hash);
        } else {
            let mut hash_set = HashSet::new();
            hash_set.insert(tx_hash);
            self.address_to_txs.insert(address.to_vec(), hash_set);
        }
    }

    fn get_transaction_hashes(&self, address: &[u8]) -> Vec<HashResult> {
        if let Some(txs) = self.address_to_txs.get(address) {
            txs.iter().cloned().collect()
        } else {
            vec![]
        }
    }

    fn add_pending_transaction(&mut self, transaction: Transaction) {
        log::debug!(
            "Added a pending transaction {} from {}.",
            hex::encode(transaction.hash()),
            hex::encode(&transaction.sender),
        );
        self.pending_transactions.push(transaction);
    }

    fn get_pending_transactions(&self) -> &[Transaction] {
        &self.pending_transactions
    }

    fn clear_pending_transactions(&mut self) {
        self.pending_transactions.clear();
    }

    fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
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
