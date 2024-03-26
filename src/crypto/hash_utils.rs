use sha2::{Digest, Sha256};

pub type Address = String;
pub type HashResult = [u8; 32];

pub fn sha256(bytes: &[u8]) -> HashResult {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}
