use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{crypto::hash_utils::HashResult, types::Satoshi};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub previous_block_hash: HashResult,
    pub merkle_root: HashResult,
    pub timestamp: u64,
    pub difficulty: u8,
    pub nonce: u32,
    pub reward: Satoshi,
}
impl BlockHeader {
    pub fn from(
        merkle_root: HashResult,
        previous_block_hash: HashResult,
        difficulty: u8,
        timestamp: u64,
        reward: Satoshi,
    ) -> Self {
        Self {
            version: 0,
            previous_block_hash,
            merkle_root,
            difficulty,
            timestamp,
            nonce: 0,
            reward,
        }
    }
    pub fn finalize(&self) -> HashResult {
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
        data.extend(self.reward.to_string().as_bytes());
        data
    }
    pub fn size(&self) -> usize {
        self.as_bytes().len()
    }
}
