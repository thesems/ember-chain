use tokio::io::AsyncWrite;
use tokio::runtime::Runtime;
use tonic::transport::Channel;

use crate::config::models::WalletConfig;
use crate::crypto::account::Account;
use crate::crypto::hash_utils::{Address, hash_from_vec_u8, HashResult};
use crate::proto::proto_node::{PublicKey, Transaction, TransactionReq};
use crate::proto::proto_node::node_client::NodeClient;
use crate::transaction::input::Input;
use crate::transaction::script::{Item, Script};

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
    account: Account,
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

    pub fn query_balance(&mut self) {
        let balance = self
            .rt
            .block_on(self.client.as_mut().unwrap().get_balance(PublicKey {
                key: self.account.public_key().to_vec(),
            }))
            .unwrap();
        log::info!("Balance = {}", balance.get_ref().balance);
    }

    pub fn create_transaction(&mut self, receiver: &Address, amount: u64, fee: u64) -> Result<HashResult, String> {
        let unspent_outputs = self
            .rt
            .block_on(self.client.as_mut().unwrap().get_utxo(PublicKey {
                key: self.account.public_key().to_vec(),
            }))
            .unwrap()
            .into_inner();

        let inputs = unspent_outputs.unspent_outputs.into_iter().map(|unspent_output| (
            hash_from_vec_u8(&unspent_output.previous_transaction_hash),
            unspent_output.previous_transaction_output_index,
            unspent_output.amount,
        )).collect();

        if let Ok(tx) = crate::transaction::Transaction::create_pay_to_pub_key_hash(
            inputs,
            amount,
            fee,
            &self.account,
            receiver,
        ) {
            if let Ok(encoded_tx) = serde_json::to_string(&tx) {
                self.rt.block_on(self.client.as_mut().unwrap().add_transaction(Transaction {
                    tx_json: encoded_tx,
                })).unwrap();
                return Ok(tx.hash());
            }
            return Err("Failed to encode transaction.".to_string());
        }
        return Err("Failed to create transaction.".to_string());
    }
}
