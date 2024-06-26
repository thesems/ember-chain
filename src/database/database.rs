use crate::crypto::hash_utils::Address;
use crate::types::Satoshi;
use crate::{block::Block, crypto::hash_utils::HashResult, transaction::Transaction};

pub type DatabaseType = dyn Database + Send + Sync;

pub trait Database {
    /// Retrieves the program's version
    fn get_version(&self) -> String;

    /// Creates and inserts the genesis block.
    fn create_genesis_block(&mut self);

    /// Inserts a block into the database.
    fn insert_block(&mut self, block: Block);

    /// Retrieves the blocks of the longest chain.
    fn get_blocks(&self) -> Vec<&Block>;

    /// Resolves the fork by determining the longest chain.
    fn resolve_fork(&mut self);

    /// Retrieves the number of blocks.
    fn block_height(&self) -> usize;

    /// Retrieves the last block inserted.
    fn head(&self) -> Option<&Block>;

    /// Adds a new unspent transaction output (UTXO).
    ///
    /// Parameters
    ///
    /// - tx_hash: a HashResult of the transaction
    /// - output_index: index of the unspent output within the transacion
    ///
    fn add_utxo(&mut self, tx_hash: HashResult, output_index: u32);

    /// Removes a spent transaction output (UTXO).
    fn remove_utxo(&mut self, tx_hash: &HashResult, output_index: u32);

    /// Checks if a transaction output is unspent.
    fn is_utxo(&self, tx_hash: &HashResult, output_index: u32) -> bool;

    /// Retrieves all unspent outputs of a public key.
    fn get_utxo(&self, public_key: &Address) -> Vec<(HashResult, u32, Satoshi)>;

    /// Adds a transaction identified by its hash
    fn add_transaction(&mut self, tx_hash: HashResult, transaction: Transaction);

    /// Removes a transaction identified by its hash
    fn remove_transaction(&mut self, tx_hash: HashResult) -> Option<Transaction>;

    /// Searches for a transaction given its hash
    fn get_transaction(&self, tx_hash: &HashResult) -> Option<&Transaction>;

    /// Maps a public key address to a transaction hash
    fn map_address_to_transaction_hash(&mut self, address: &[u8], tx_hash: HashResult);

    /// Maps a public key address to a transaction hash
    fn get_transaction_hashes(&self, address: &[u8]) -> Vec<HashResult>;

    /// Adds a pending transaction
    fn add_pending_transaction(&mut self, transaction: Transaction);

    /// Retrieves all the pending transactions
    fn get_pending_transactions(&self) -> &[Transaction];

    /// Clears (removes) all the pending transactions
    fn clear_pending_transactions(&mut self);
}
