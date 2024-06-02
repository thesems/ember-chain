use std::collections::{HashMap, HashSet};

use chrono::DateTime;

use crate::block::BlockHeader;
use crate::crypto::hash_utils::Address;
use crate::crypto::merkle_tree::generate_merkle_root;
use crate::types::Satoshi;
use crate::{block::Block, crypto::hash_utils::HashResult, transaction::Transaction};

use super::database::Database;

pub struct InMemoryDatabase {
    blocks: HashMap<String, Block>,
    chains: HashMap<String, Vec<String>>,
    longest_chain_tip_hash: String,
    pending_transactions: Vec<Transaction>,
    transactions: HashMap<HashResult, Transaction>,
    unspent_outputs: HashSet<(HashResult, u32)>,
    address_to_txs: HashMap<Vec<u8>, HashSet<HashResult>>,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        InMemoryDatabase {
            blocks: HashMap::new(),
            chains: HashMap::new(),
            longest_chain_tip_hash: String::new(),
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
    fn create_genesis_block(&mut self) {
        let ts = DateTime::parse_from_rfc3339("2009-01-03T18:15:05-00:00")
            .unwrap()
            .timestamp() as u64;
        let merkle_root = generate_merkle_root(vec![]);
        let header = BlockHeader::from(merkle_root, [0u8; 32], 0, ts, 0);
        let block = Block {
            hash: header.finalize(),
            header,
            transactions: vec![],
        };
        let block_hash = block.get_hash_as_string(false).clone();

        self.blocks.insert(block_hash.clone(), block.clone());
        self.chains
            .insert(block_hash.clone(), vec![block_hash.clone()]);
        self.longest_chain_tip_hash = block_hash.clone();

        log::info!(
            "★★★ GENESIS BLOCK ({}) ★★★",
            hex::encode(block.hash.get(..5).unwrap())
        );
    }
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

        log::info!(
            "Block ({}) added at height {} with {} transactions.",
            hex::encode(block.hash.get(..5).unwrap()),
            block_height,
            block.transactions.len()
        );

        let block_hash = block.get_hash_as_string(false);
        self.blocks.insert(block_hash.clone(), block.clone());

        if let Some(chain) = self
            .chains
            .get(&hex::encode(block.header.previous_block_hash))
        {
            let mut new_chain = chain.clone();
            new_chain.push(block_hash.clone());

            let new_chain_height = new_chain.len();
            self.chains.insert(block_hash.clone(), new_chain);

            if new_chain_height > self.chains.get(&self.longest_chain_tip_hash).unwrap().len() {
                self.longest_chain_tip_hash = block_hash.clone();
            }
        } else {
            // Orphan blocks
            self.chains
                .insert(block_hash.clone(), vec![block_hash.clone()]);
        }
    }

    fn get_blocks(&self) -> Vec<&Block> {
        let mut blocks = vec![];
        if let Some(block_hashes) = self.chains.get(&self.longest_chain_tip_hash) {
            for block_hash in block_hashes {
                blocks.push(self.blocks.get(block_hash).unwrap());
            }
        }
        blocks
    }

    fn resolve_fork(&mut self) {
        let mut longest_chain_tip_hash = self.longest_chain_tip_hash.clone();
        let mut max_length = self.chains.get(&longest_chain_tip_hash).unwrap().len();

        for (tip, chain) in &self.chains {
            if chain.len() > max_length {
                longest_chain_tip_hash = tip.clone();
                max_length = chain.len();
            }
        }

        if longest_chain_tip_hash != self.longest_chain_tip_hash {
            log::warn!(
                "Fork detected. Changed head to {}.",
                longest_chain_tip_hash.get(..5).unwrap()
            )
        }

        self.longest_chain_tip_hash = longest_chain_tip_hash;
    }

    fn block_height(&self) -> usize {
        if let Some(block_hashes) = self.chains.get(&self.longest_chain_tip_hash) {
            block_hashes.len()
        } else {
            0
        }
    }

    fn head(&self) -> Option<&Block> {
        if let Some(block_hashes) = self.chains.get(&self.longest_chain_tip_hash) {
            if let Some(block) = self.blocks.get(block_hashes.last().unwrap()) {
                return Some(block);
            } else {
                panic!("No block for hash found!")
            }
        }
        None
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
