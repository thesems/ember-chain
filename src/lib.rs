use sha2::{Sha256, Digest};

pub mod blockchain;
mod block;
mod block_header;
mod transaction;
pub mod merkle_tree;
mod pow_utils;

pub type Address = String;
pub type HashResult = [u8; 32];

pub fn sha256(bytes: &[u8]) -> HashResult {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

