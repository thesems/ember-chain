use sha2::{Digest, Sha256};

use crate::hash_utils::HashResult;

pub struct BlockHeader {
    pub version: u32,
    pub previous_block_hash: HashResult,
    pub merkle_root: HashResult,
    pub timestamp: u64,
    pub difficulty: u8,
    pub nonce: u32,
}
impl Default for BlockHeader {
    fn default() -> BlockHeader {
        BlockHeader {
            version: 0,
            previous_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: 0,
            difficulty: 0,
            nonce: 0,
        }
    }
}
impl BlockHeader {
    pub fn from(
        merkle_root: HashResult,
        previous_block_hash: HashResult,
        difficulty: u8,
        timestamp: u64,
    ) -> Self {
        Self {
            version: 0,
            previous_block_hash,
            merkle_root,
            difficulty,
            timestamp,
            nonce: 0,
        }
    }
    pub fn finalize(&mut self) -> HashResult {
        let mut hasher = Sha256::new();
        hasher.update(self.as_bytes());
        hasher.finalize().into()
    }
    fn as_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(self.previous_block_hash);
        data.extend(self.difficulty.to_string().as_bytes());
        data.extend(self.timestamp.to_string().as_bytes());
        data.extend(self.nonce.to_string().as_bytes());
        data
    }
    pub fn size(&self) -> usize {
        self.as_bytes().len()
    }
}
