use crate::crypto::hash_utils::HashResult;

use super::script::Script;

#[derive(Clone)]
pub struct Input {
    prev_tx_hash: HashResult,
    prev_tx_output_index: u32,
    script_sig: Script,
}
impl Input {
    pub fn hash(&self) -> Vec<u8> {
        let mut result = Vec::from(self.prev_tx_hash);
        for b in self.prev_tx_output_index.to_be_bytes() {
            result.push(b);
        }
        for b in self.script_sig.hash() {
            result.push(b);
        }
        result
    }
}
