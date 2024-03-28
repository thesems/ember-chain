use crate::crypto::hash_utils::HashResult;

use super::script::Script;

#[derive(Clone, Debug)]
pub struct Input {
    prev_tx_hash: HashResult,
    prev_tx_output_index: u32,
    script_sig: Script,
}
impl Input {
    pub fn new(prev_tx_hash: HashResult, prev_tx_output_index: u32, script_sig: Script) -> Self {
        Self {
            prev_tx_hash,
            prev_tx_output_index,
            script_sig,
        }
    }
    pub fn set_script(&mut self, script_sig: Script) {
        self.script_sig = script_sig;
    }
    pub fn hash(&self) -> Vec<u8> {
        let mut result = Vec::from(self.prev_tx_hash);
        for b in self.prev_tx_output_index.to_be_bytes() {
            result.push(b);
        }
        // Skip hashing the script, since it contains a signature.
        // Advance implementation would hash script operations,
        // except the signature itself.
        //
        // for b in self.script_sig.hash() {
        //     result.push(b);
        // }
        result
    }
}
