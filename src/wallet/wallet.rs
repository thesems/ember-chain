use crate::config::models::WalletConfig;
use crate::crypto::account::{Account, AccountError};
use crate::proto::proto_node::node_client::NodeClient;
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

pub struct Wallet {
    config: WalletConfig,
    account: Option<Account>,
    _client: Option<NodeClient<Channel>>,
}
impl Wallet {
    pub fn new(config: WalletConfig) -> Result<Self, WalletError> {
        Ok(Wallet {
            config,
            account: None,
            _client: None,
        })
    }

    /// Loads an account (public and private keys) into the wallet.
    /// If the supplied file doesn't exist, it creates a new account.
    pub fn load_account(&mut self) -> Result<(), AccountError> {
        self.account = Some(match Account::load(self.config.account.clone()) {
            Ok(acc) => acc,
            Err(_) => {
                let account = Account::new(self.config.account.clone())?;
                account.save();
                account
            }
        });
        Ok(())
    }

    pub async fn connect_node(&self, _rpc_url: String) {
        // if let Ok(client) = NodeClient::connect(rpc_url).await {}
    }
}
