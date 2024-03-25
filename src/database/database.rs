use crate::block::Block;

pub trait Database {
    /// Method to insert a block into the database
    fn insert_block(&mut self, block: Block);

    /// Method to retrieve the blockchain from the database
    fn get_blocks(&self) -> &[Block];
}
