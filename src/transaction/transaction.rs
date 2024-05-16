use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

use crate::{
    crypto::{
        account::Account,
        hash_utils::{HashResult, sha256},
    },
    database::database::DatabaseType,
    mining::pow_utils::get_random_range,
    types::Satoshi,
};

use super::{
    input::Input,
    output::Output,
    script::{Item, Operation, Script, ScriptRunner},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: Vec<u8>,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}
impl Transaction {
    pub fn new(sender: Vec<u8>, inputs: Vec<Input>, outputs: Vec<Output>) -> Self {
        Self {
            sender,
            inputs,
            outputs,
        }
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
            if input.utxo_tx_hash == [0u8; 32] && input.utxo_output_index == 0 {
                // coinbase transaction
                total_input += current_block_reward;

                if self.inputs.len() > 1 {
                    log::error!("Coinbase transaction can only have a single input.");
                    return false;
                }

                continue;
            }

            if let Some(tx) = database
                .lock()
                .unwrap()
                .get_transaction(&input.utxo_tx_hash)
            {
                if let Some(amount) = tx.get_amount(input.utxo_output_index) {
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
            if let Some(prev_tx) = database
                .lock()
                .unwrap()
                .get_transaction(&input.utxo_tx_hash)
            {
                if let Some(prev_tx_output) = prev_tx.outputs.get(input.utxo_output_index as usize)
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
    /// Creates a coinbase transaction, which contains the block reward.
    ///
    /// Parameters
    ///
    /// - reward: amount of block reward in satoshis
    /// - pub_key: public key of the receiver
    ///
    pub fn create_coinbase(reward: Satoshi, pub_key: Vec<u8>) -> Transaction {
        Transaction::new(
            [0u8; 32].to_vec(),
            vec![Input::new(
                [0u8; 32],
                0,
                Script::new(vec![
                    Item::Operation(Operation::Nop),
                    Item::Data(get_random_range(0, u64::MAX).to_le_bytes().to_vec(), None),
                ]),
            )],
            vec![Output::new(
                reward,
                Script::new(vec![
                    Item::Data(pub_key.clone(), None),
                    Item::Operation(Operation::Dup),
                    Item::Operation(Operation::Equal),
                ]),
                pub_key,
            )],
        )
    }
    /// Creates a pay-to-public-key transaction.
    /// This is the bread-and-butter of the transaction system.
    ///
    /// Parameters
    ///
    /// - prev_tx_hash: Transaction hash of the transaction that contains the output to be spent
    /// - prev_tx_output_index: Index of the spendable output of the transaction
    /// - amount: Amount to be spent. Rest will be taken as miner fee.
    /// - fee:  Amount to be given to the miner.
    /// - account: Receiver's public key used for unlocking the funds.
    ///
    /// OPTIMIZE: if miner equals sender, avoid adding additional output.
    ///
    pub fn create_pay_to_pub_key_hash(
        inputs: Vec<(HashResult, u32, u64)>,
        amount: u64,
        fee: u64,
        account: &Account,
        rx_pub_key: &[u8],
    ) -> Result<Transaction, String> {
        let mut total_input_value = 0;
        let mut tx_inputs = Vec::new();

        for (prev_tx_hash, prev_tx_output_index, value) in &inputs {
            total_input_value += value;
            tx_inputs.push(Input::new(
                *prev_tx_hash,
                *prev_tx_output_index,
                Script::new(vec![
                    Item::Data(vec![], Some("tx_hash".to_string())),
                    Item::Data(account.public_key().to_vec(), None),
                ]),
            ));
            if total_input_value >= amount {
                break;
            }
        }

        let total_input_value = total_input_value;
        if total_input_value < amount + fee {
            return Err("Insufficient funds for the transaction".to_string());
        }

        let create_operations = |receiver_pub_key: Vec<u8>| {
            vec![
                Item::Operation(Operation::Dup),
                Item::Operation(Operation::Hash256),
                Item::Data(receiver_pub_key, None),
                Item::Operation(Operation::EqualVerify),
                Item::Operation(Operation::CheckSig),
            ]
        };

        let mut tx_outputs = vec![Output::new(
            amount,
            Script::new(create_operations(rx_pub_key.to_vec())),
            rx_pub_key.to_vec(),
        )];

        let change = total_input_value - amount - fee;
        if change > 0 {
            tx_outputs.push(Output::new(
                change,
                Script::new(create_operations(account.public_key().to_vec())),
                account.public_key().to_vec(),
            ))
        }

        let mut tx = Transaction::new(account.public_key().to_vec(), tx_inputs, tx_outputs);
        let tx_hash = tx.hash();
        for input in &mut tx.inputs {
            let item = input.script_sig.items.iter_mut().find(|item| match item {
                Item::Data(data, name) => {
                    if let Some(name) = name {
                        if name == "tx_hash" {
                            return true;
                        }
                    }
                    false
                }
                _ => false,
            });

            let item = item.expect("Cannot find prepared script item for transaction hash.");
            *item = Item::Data(account.sign(&tx_hash).to_vec(), None);
        }
        Ok(tx)
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
    /// Removes all UTXOs from inputs and adds all UTXOs from outputs.
    ///
    /// Parameters
    ///
    /// - database: Stores UTXOs in a some database
    ///
    pub fn update_utxos(&self, database: &mut MutexGuard<'_, DatabaseType>) {
        let tx_hash = self.hash();

        for input in self.inputs.iter() {
            database.remove_utxo(&input.utxo_tx_hash, input.utxo_output_index);
        }

        for output_index in 0..self.outputs.len() {
            database.add_utxo(tx_hash, output_index as u32);
        }
    }
}
