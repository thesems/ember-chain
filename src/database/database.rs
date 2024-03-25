use crate::block::Block;

pub trait Database {
    /// Method to insert a block into the database
    fn insert_block(&mut self, block: Block);

    /// Method to retrieve the blockchain from the database
    fn get_blocks(&self) -> &[Block];

    /// Method to add a new unspent transaction output (UTXO) to the database
    fn add_utxo(&mut self, tx_hash: String, output_index: usize);

    /// Method to remove a spent transaction output (UTXO) from the database
    fn remove_utxo(&mut self, tx_hash: &str, output_index: usize);

    /// Method to check if a transaction output is unspent
    fn is_utxo(&self, tx_hash: &str, output_index: usize) -> bool;
}
