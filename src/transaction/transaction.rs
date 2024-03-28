use std::sync::{Arc, Mutex};

use crate::{
    crypto::{
        account::Account,
        hash_utils::{sha256, HashResult},
    },
    database::database::{Database, DatabaseType},
    types::Satoshi,
};

use super::{
    input::Input,
    output::Output,
    script::{Item, Operation, Script, ScriptRunner},
};

#[derive(Clone, Debug)]
pub struct Transaction {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}
impl Transaction {
    pub fn new(inputs: Vec<Input>, outputs: Vec<Output>) -> Self {
        Self { inputs, outputs }
    }
    pub fn get_amount(&self, output_index: u32) -> Option<u64> {
        if let Some(output) = self.outputs.get(output_index as usize) {
            return Some(output.value);
        }
        None
    }
    pub fn verify(
        &self,
        current_block_reward: Satoshi,
        database: &Arc<Mutex<DatabaseType>>,
    ) -> bool {
        let mut total_input = 0;
        let mut total_output = 0;

        for input in self.inputs.iter() {
            if input.prev_tx_hash == [0u8; 32] && input.prev_tx_output_index == 0 {
                // coinbase transaction
                total_input += current_block_reward;
                
                if self.inputs.len() > 1 {
                    log::error!("Coinbase transaction can only have a single input.");
                    return false;
                }

                continue;
            }

            if let Some(tx) = database.lock().unwrap().get_transaction(input.prev_tx_hash) {
                if let Some(amount) = tx.get_amount(input.prev_tx_output_index) {
                    total_input += amount;
                } else {
                    log::error!(
                        "Transaction input is referencing an invalid transaction output index."
                    );
                    return false;
                }
            } else {
                log::error!("Transaction input is referencing a non-existent transaction output.");
                return false;
            }
        }

        for output in self.outputs.iter() {
            total_output += output.value;
        }

        if total_input != total_output {
            log::error!(
                "Total input amount {} != {} output amount.",
                total_input,
                total_output
            );
            return false;
        }
        true
    }
    pub fn verify_inputs(&self, database: &Arc<Mutex<DatabaseType>>) -> bool {
        for input in &self.inputs {
            if let Some(prev_tx) = database.lock().unwrap().get_transaction(input.prev_tx_hash) {
                if let Some(prev_tx_output) =
                    prev_tx.outputs.get(input.prev_tx_output_index as usize)
                {
                    let mut script_runner = ScriptRunner::new(prev_tx.hash());
                    let mut items = input.script_sig.items.clone();
                    items.append(&mut prev_tx_output.script_pub_key.items.clone());
                    if !script_runner.execute_script(items) {
                        return false;
                    }
                }
            }
        }

        true
    }
    pub fn create_coinbase(reward: Satoshi, pub_key: Vec<u8>) -> Transaction {
        Transaction::new(
            vec![Input::new(
                [0u8; 32],
                0,
                Script::new(vec![Item::Operation(Operation::Nop)]),
            )],
            vec![Output::new(
                reward,
                Script::new(vec![
                    Item::Data(pub_key),
                    Item::Operation(Operation::Dup),
                    Item::Operation(Operation::Equal),
                ]),
            )],
        )
    }
    pub fn create_pay_to_pubkey_hash(
        prev_tx_hash: HashResult,
        prev_tx_output_index: u32,
        amount: u64,
        account: &Account,
    ) -> Transaction {
        let mut tx = Transaction::new(
            vec![Input::new(
                prev_tx_hash,
                prev_tx_output_index,
                Script::new(vec![]),
            )],
            vec![Output::new(
                amount,
                Script::new(vec![
                    Item::Operation(Operation::Dup),
                    Item::Operation(Operation::Hash256),
                    Item::Data(account.public_key().to_vec()),
                    Item::Operation(Operation::EqualVerify),
                    Item::Operation(Operation::CheckSig),
                ]),
            )],
        );
        let tx_hash = tx.hash();
        tx.inputs[0].script_sig = Script::new(vec![
            Item::Data(account.sign(&tx_hash).to_vec()),
            Item::Data(account.public_key().to_vec()),
        ]);
        tx
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
}
