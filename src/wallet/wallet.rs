use crate::config::models::WalletConfig;
use crate::crypto::account::{Account, AccountError};
use crate::proto::proto_node::node_client::NodeClient;
use crate::proto::proto_node::{PublicKey, Version};
use crate::wallet;
use tokio::runtime::Runtime;
use tonic::transport::Channel;

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
    config: WalletConfig,
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

    pub fn connect_node(&mut self, rpc_url: &str) {
        self.rt.block_on(async {
            let rpc_url: String = rpc_url.to_string();
            if let Ok(client) = NodeClient::connect(rpc_url.clone()).await {
                log::debug!("Connected to {}.", &rpc_url);
                self.client = Some(client);
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
}
