use tokio::runtime::Runtime;
use tonic::transport::Channel;

use crate::config::models::WalletConfig;
use crate::crypto::account::Account;
use crate::crypto::hash_utils::{hash_from_vec_u8, Address, HashResult};
use crate::proto::proto_node::node_client::NodeClient;
use crate::proto::proto_node::{PublicKey, Transaction, UnspentOutputs};

#[derive(Debug)]
pub enum WalletError {
    AccountError,
}

impl From<crate::crypto::account::AccountError> for WalletError {
    fn from(_: crate::crypto::account::AccountError) -> Self {
        Self::AccountError
    }
}

pub struct Wallet<'a> {
    rt: &'a Runtime,
    pub config: WalletConfig,
    pub account: Account,
    client: Option<NodeClient<Channel>>,
}

impl<'a> Wallet<'a> {
    pub fn new(rt: &'a Runtime, config: WalletConfig) -> Result<Self, WalletError> {
        Ok(Wallet {
            rt,
            config: config.clone(),
            account: Account::load_or_create(config.account.clone())?,
            client: None,
        })
    }

    pub fn connect_node(&mut self) -> Result<(), tonic::transport::Error> {
        self.rt.block_on(async {
            let rpc_url: String = self.config.rpc_url.to_string();
            match NodeClient::connect(rpc_url.clone()).await {
                Ok(client) => {
                    log::debug!("Connected to {}.", &rpc_url);
                    self.client = Some(client);
                    Ok(())
                }
                Err(err) => {
                    log::error!("Failed to connect to {}. Error: {}", &rpc_url, &err);
                    Err(err)
                }
            }
        })
    }

    fn get_unspent_outputs(&mut self, address: Address) -> UnspentOutputs {
        self.rt
            .block_on(
                self.client
                    .as_mut()
                    .unwrap()
                    .get_utxo(PublicKey { key: address }),
            )
            .unwrap()
            .into_inner()
    }

    pub fn get_address(&self) -> String {
        format!("{:02X?}", self.account.public_key_as_hex())
    }

    pub fn get_address_from_string(&self, address: &str) -> Option<Address> {
        self.account.public_key_from_hex(address)
    }

    pub fn get_balance(&mut self, address: Address) -> u64 {
        let unspent_outputs = self.get_unspent_outputs(address);
        unspent_outputs
            .unspent_outputs
            .into_iter()
            .map(|unspent_output| unspent_output.amount)
            .sum()
    }

    pub fn create_transaction(
        &mut self,
        rx_pub_key: &Address,
        amount: u64,
        fee: u64,
    ) -> Result<HashResult, String> {
        let unspent_outputs = self.get_unspent_outputs(self.account.public_key().to_vec());
        let inputs = unspent_outputs
            .unspent_outputs
            .into_iter()
            .map(|unspent_output| {
                (
                    hash_from_vec_u8(&unspent_output.previous_transaction_hash),
                    unspent_output.previous_transaction_output_index,
                    unspent_output.amount,
                )
            })
            .collect();

        if let Ok(tx) = crate::transaction::Transaction::create_pay_to_pub_key_hash(
            inputs,
            amount,
            fee,
            &self.account,
            rx_pub_key,
        ) {
            log::debug!("Send-Transaction={:?}", hex::encode(tx.hash()));
            if let Ok(encoded_tx) = serde_json::to_string(&tx) {
                self.rt
                    .block_on(self.client.as_mut().unwrap().add_transaction(Transaction {
                        tx_json: encoded_tx,
                    }))
                    .unwrap();
                return Ok(tx.hash());
            }
            return Err("Failed to encode transaction.".to_string());
        }
        Err("Failed to create transaction.".to_string())
    }
}
