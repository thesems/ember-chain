use crate::types::Satoshi;

use super::script::Script;

#[derive(Clone, Debug)]
pub struct Output {
    pub value: Satoshi,
    pub script_pub_key: Script,
}
impl Output {
    pub fn new(value: Satoshi, script_pub_key: Script) -> Self {
        Self {
            value,
            script_pub_key,
        }
    }
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
