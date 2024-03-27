use crate::crypto::hash_utils::{sha256, HashResult};

use super::{input::Input, output::Output};

#[derive(Clone)]
pub struct Transaction {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}
impl Transaction {
    pub fn new(inputs: Vec<Input>, outputs: Vec<Output>) -> Self {
        Self { inputs, outputs }
    }
    pub fn hash(&self) -> HashResult {
        let mut bytes = vec![];
        for input in self.inputs {
            bytes.append(&mut input.hash().clone());
        }
        for output in self.outputs {
            bytes.append(&mut output.hash().clone());
        }
        sha256(&bytes)
    }
}
