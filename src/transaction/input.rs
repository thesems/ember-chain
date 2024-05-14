use serde::{Deserialize, Serialize};

use crate::crypto::hash_utils::HashResult;

use super::script::Script;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    pub utxo_tx_hash: HashResult,
    pub utxo_output_index: u32,
    pub script_size: u16,
    pub script_sig: Script,
}
impl Input {
    pub fn new(prev_tx_hash: HashResult, prev_tx_output_index: u32, script_sig: Script) -> Self {
        let script_size = script_sig.hash().len();
        Self {
            utxo_tx_hash: prev_tx_hash,
            utxo_output_index: prev_tx_output_index,
            script_size: script_size as u16,
            script_sig,
        }
    }
    pub fn hash(&self) -> Vec<u8> {
        let mut result = Vec::from(self.utxo_tx_hash);
        for b in self.utxo_output_index.to_be_bytes() {
            result.push(b);
        }
        for b in self.script_sig.hash() {
            result.push(b);
        }
        result
    }
}
