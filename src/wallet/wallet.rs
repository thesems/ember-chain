use crate::config::models::WalletConfig;
use crate::crypto::account::Account;
use crate::proto::proto_node::node_client::NodeClient;
use crate::proto::proto_node::PublicKey;
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
    _config: WalletConfig,
    account: Account,
    client: Option<NodeClient<Channel>>,
}
impl<'a> Wallet<'a> {
    pub fn new(rt: &'a Runtime, config: WalletConfig) -> Result<Self, WalletError> {
        Ok(Wallet {
            rt,
            _config: config.clone(),
            account: Account::load_or_create(config.account.clone())?,
            client: None,
        })
    }

    pub fn connect_node(&mut self, rpc_url: &str) {
        self.rt.block_on(async {
            let rpc_url: String = rpc_url.to_string();
            match NodeClient::connect(rpc_url.clone()).await {
                Ok(client) => {
                    log::debug!("Connected to {}.", &rpc_url);
                    self.client = Some(client);
                }
                Err(err) => {
                    log::error!("Failed to connect to {}. Error: {}", &rpc_url, &err);
                }
            }
        })
    }

    pub fn query_balance(&mut self) {
        if self.client.is_none() {
            log::warn!("You have to connect to the RPC node first!");
            return;
        }

        let balance = self
            .rt
            .block_on(self.client.as_mut().unwrap().get_balance(PublicKey {
                key: self.account.public_key().to_vec(),
            }))
            .unwrap();
        log::info!("Balance = {}", balance.get_ref().balance);
    }
}
