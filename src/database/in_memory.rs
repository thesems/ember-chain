use crate::block::Block;

use super::database::Database;

pub struct InMemoryDatabase {
    blocks: Vec<Block>,
}

impl InMemoryDatabase {
    fn new() -> Self {
        InMemoryDatabase { blocks: vec![] }
    }
}

impl Database for InMemoryDatabase {
    fn insert_block(&mut self, block: Block) {
        println!("Inserting block into in-memory database.");
    }

    fn get_blocks(&self) -> &[Block] {
        &self.blocks
    }
}

#[cfg(test)]
mod tests {
    use crate::{block::{Block, BlockHeader}, database::{database::Database, InMemoryDatabase}};

    #[test]
    fn test_insert_block() {
        let mut in_memory_db = InMemoryDatabase::new();
        in_memory_db.insert_block(Block::new(BlockHeader::default(), vec![]));
        assert_eq!(in_memory_db.blocks.len(), 1)
    }
}
