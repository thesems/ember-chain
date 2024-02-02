use sha2::{Digest, Sha256};

pub struct BlockHeader {
    pub version: u32,
    pub previous_block_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u32,
    pub target: u32,
    pub nonce: u32,
}
impl BlockHeader {
    pub fn from(previous_block_hash: [u8; 32], difficulty: u32) -> Self {
        Self {
            version: 0,
            previous_block_hash,
            merkle_root: [0; 32],
            target: difficulty,
            timestamp: 0,
            nonce: 0,
        }
    }
    pub fn finalize(&mut self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.as_bytes());
        hasher.finalize().into()
    }
    fn as_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(self.previous_block_hash);
        data.extend(self.merkle_root);
        data.extend(self.target.to_string().as_bytes());
        data.extend(self.timestamp.to_string().as_bytes());
        data.extend(self.nonce.to_string().as_bytes());
        data
    }
    pub fn size(&self) -> usize {
        self.as_bytes().len()
    }
}
