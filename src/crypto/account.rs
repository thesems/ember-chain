use std::fs;

use ring::signature::{Ed25519KeyPair, KeyPair};
use ring::{
    rand,
    signature::{self},
};

use crate::config::models::AccountConfig;

#[derive(Debug)]
pub enum AccountError {
    CryptoError,
    LoadError,
}

impl From<ring::error::Unspecified> for AccountError {
    fn from(_: ring::error::Unspecified) -> Self {
        AccountError::CryptoError
    }
}
impl From<ring::error::KeyRejected> for AccountError {
    fn from(_: ring::error::KeyRejected) -> Self {
        AccountError::CryptoError
    }
}

pub struct Account {
    config: AccountConfig,
    pkcs8_data: Vec<u8>,
    key_pair: Ed25519KeyPair,
}

impl Account {
    pub fn new(config: AccountConfig) -> Result<Self, AccountError> {
        let rng = rand::SystemRandom::new();
        let pkcs8_data = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_data.as_ref())?;

        log::info!("Generated a new account.");
        Ok(Account {
            config: config.clone(),
            pkcs8_data: pkcs8_data.as_ref().to_vec(),
            key_pair,
        })
    }

    pub fn load_or_create(config: AccountConfig) -> Result<Self, AccountError> {
        Ok(match Account::load(config.clone()) {
            Ok(acc) => acc,
            Err(_) => {
                let account = Account::new(config)?;
                account.save();
                account
            }
        })
    }

    /// Attempts to load the private and public keys from the configured location.
    pub fn load(config: AccountConfig) -> Result<Self, AccountError> {
        if let Ok(key_data) = fs::read(&config.keys_path) {
            log::info!("Loaded the account data from: {}.", &config.keys_path);
            return Ok(Account {
                config: config,
                pkcs8_data: key_data.to_vec(),
                key_pair: signature::Ed25519KeyPair::from_pkcs8(&key_data)?,
            });
        }
        Err(AccountError::LoadError)
    }

    /// Saves the public and private keys into pkcs8 binary data to a file.
    /// Returns a boolean on success or failure.
    pub fn save(&self) -> bool {
        if fs::write(&self.config.keys_path, &self.pkcs8_data).is_err() {
            log::error!("Failed to write key data to {}.", &self.config.keys_path);
            return false;
        }
        log::info!("Saved the account data to: {}.", &self.config.keys_path);
        true
    }

    /// Signs the message using the generated key pair.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.key_pair.sign(message).as_ref().to_vec()
    }

    /// Public key of the generated key pair.
    pub fn public_key(&self) -> &[u8] {
        self.key_pair.public_key().as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::models::AccountConfig,
        crypto::{account::Account, signature::verify},
    };

    #[test]
    fn test_load_save() {
        let config = AccountConfig {
            keys_path: "./configs/keydata.pkcs8".to_string(),
        };
        let account = Account::new(config.clone()).expect("Failed to create account.");
        assert!(account.save());
        let loaded_account = Account::load(config.clone()).expect("Failed to load an account.");

        assert_eq!(account.public_key(), loaded_account.public_key());
    }

    #[test]
    fn test_sign_verify() {
        let config = AccountConfig {
            keys_path: "./configs/keydata.pkcs8".to_string(),
        };
        let account = Account::new(config).unwrap();
        let message = b"Fear is the mind-killer. I will face my fear. Only I will remain.";

        // Encrypt the message
        let signature = account.sign(message);

        // Validate the signature
        let is_valid = verify(message, account.public_key(), &signature).is_ok();

        assert!(is_valid);
    }
}
