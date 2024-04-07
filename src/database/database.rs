use crate::{block::Block, crypto::hash_utils::HashResult, transaction::Transaction};

pub type DatabaseType = dyn Database + Send + Sync;

pub trait Database {
    /// Inserts a block into the database.
    fn insert_block(&mut self, block: Block);

    /// Retrieves the blocks.
    fn get_blocks(&self) -> &[Block];

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
    fn add_utxo(&mut self, tx_hash: HashResult, output_index: usize);

    /// Removes a spent transaction output (UTXO).
    fn remove_utxo(&mut self, tx_hash: &HashResult, output_index: usize);

    /// Checks if a transaction output is unspent.
    fn is_utxo(&self, tx_hash: &HashResult, output_index: usize) -> bool;

    /// Adds a transaction identified by its hash
    fn add_transaction(&mut self, tx_hash: HashResult, transaction: Transaction);

    /// Removes a transaction identified by its hash
    fn remove_transaction(&mut self, tx_hash: HashResult) -> Option<Transaction>;

    /// Searches for a transaction given its hash
    fn get_transaction(&mut self, tx_hash: HashResult) -> Option<&Transaction>;

    /// Maps a public key address to a transaction hash
    fn map_address_to_transaction_hash(&mut self, address: &[u8], tx_hash: HashResult);

    /// Maps a public key address to a transaction hash
    fn get_transaction_hashes(&mut self, address: &[u8]) -> &[HashResult];

    fn add_pending_transaction(&mut self, transaction: Transaction);

    fn get_pending_transactions(&self) -> &[Transaction];

    fn clear_pending_transactions(&mut self);
}
