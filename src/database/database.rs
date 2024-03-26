use crate::block::Block;

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
    fn add_utxo(&mut self, tx_hash: String, output_index: usize);

    /// Removes a spent transaction output (UTXO).
    fn remove_utxo(&mut self, tx_hash: &str, output_index: usize);

    /// Checks if a transaction output is unspent.
    fn is_utxo(&self, tx_hash: &str, output_index: usize) -> bool;
}
