use std::collections::HashMap;

use crate::hash_utils::{sha256, Address, HashResult};

#[derive(Clone)]
pub struct Transaction {
    inputs: Vec<u32>,
    outputs: HashMap<Address, u32>,
}
impl Transaction {
    pub fn from(inputs: Vec<u32>, outputs: HashMap<Address, u32>) -> Self {
        Self { inputs, outputs }
    }
    pub fn hash(&self) -> HashResult {
        let mut input_bytes: Vec<u8> = self
            .inputs
            .iter()
            .flat_map(|x| x.to_be_bytes())
            .collect();

        self
            .outputs
            .iter()
            .for_each(|x| {
                let int_string: Vec<u32> = x.0.chars().map(|ch| ch as u32).collect();
                let mut outputs: Vec<u8> = int_string
                    .iter()
                    .flat_map(|x| x.to_be_bytes())
                    .collect();

                for y in x.1.to_be_bytes() {
                    outputs.push(y);
                }
                input_bytes.append(&mut outputs);
            });
        
        sha256(&input_bytes)
    }
}
