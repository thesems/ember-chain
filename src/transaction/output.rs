use serde::{Deserialize, Serialize};

use crate::types::Satoshi;

use super::script::Script;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub value: Satoshi,
    pub script_pub_key: Script,
    // extra meta-data
    pub receiver: Vec<u8>,
}
impl Output {
    pub fn new(value: Satoshi, script_pub_key: Script, receiver: Vec<u8>) -> Self {
        Self {
            value,
            script_pub_key,
            receiver,
        }
    }

    /// Hashes the contents of the Output struct.
    /// Ignore extra meta-data.
    pub fn hash(&self) -> Vec<u8> {
        let mut result = vec![];
        for b in self.value.to_be_bytes() {
            result.push(b);
        }
        for b in self.script_pub_key.hash() {
            result.push(b);
        }
        result
    }
}
