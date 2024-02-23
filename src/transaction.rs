use std::hash::{Hash, Hasher};
use std::collections::HashMap;

use crate::Address;

pub struct Transaction {
    inputs: Vec<u32>,
    outputs: HashMap<Address, u32>,
}
impl Transaction {
    pub fn from(inputs: Vec<u32>, outputs: HashMap<Address, u32>) -> Self {
        Self {
            inputs,
            outputs,
        }
    }
}
impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for input in self.inputs.iter() {
            input.hash(state);
        }
        for output in self.outputs.iter() {
            output.hash(state);
        }
    }
}
