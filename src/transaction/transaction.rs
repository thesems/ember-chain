use crate::crypto::hash_utils::{sha256, HashResult};

use super::{
    input::Input,
    output::Output,
    script::{Item, Operation, Script},
};

#[derive(Clone)]
pub struct Transaction {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}
impl Transaction {
    pub fn new(inputs: Vec<Input>, outputs: Vec<Output>) -> Self {
        Self { inputs, outputs }
    }
    pub fn create_coinbase(reward: u64, pub_key: &[u8]) -> Transaction {
        Transaction::new(
            vec![Input::new(
                [0u8; 32],
                0,
                Script::new(vec![Item::Operation(Operation::Nop)]),
            )],
            vec![Output::new(
                reward,
                Script::new(Self::get_coinbase(pub_key)),
            )],
        )
    }
    pub fn create_pay_to_pubkey_hash(
        prev_tx_hash: HashResult,
        prev_tx_output_index: u32,
        amount: u64,
        pub_key: &[u8],
        sig: &[u8],
    ) -> Transaction {
        Transaction::new(
            vec![Input::new(
                prev_tx_hash,
                prev_tx_output_index,
                Script::new(vec![Item::Data(sig.to_vec()), Item::Data(pub_key.to_vec())]),
            )],
            vec![Output::new(
                amount,
                Script::new(Self::get_pay_to_pubkey_hash(pub_key)),
            )],
        )
    }
    pub fn hash(&self) -> HashResult {
        let mut bytes = vec![];
        for input in &self.inputs {
            bytes.append(&mut input.hash().clone());
        }
        for output in &self.outputs {
            bytes.append(&mut output.hash().clone());
        }
        sha256(&bytes)
    }
    fn get_coinbase(pub_key: &[u8]) -> Vec<Item> {
        vec![
            Item::Data(pub_key.to_vec()),
            Item::Operation(Operation::Dup),
            Item::Operation(Operation::Equal),
        ]
    }
    fn get_pay_to_pubkey_hash(pub_key: &[u8]) -> Vec<Item> {
        vec![
            Item::Operation(Operation::Dup),
            Item::Operation(Operation::Hash256),
            Item::Data(pub_key.to_vec()),
            Item::Operation(Operation::EqualVerify),
            Item::Operation(Operation::CheckSig),
        ]
    }
}
