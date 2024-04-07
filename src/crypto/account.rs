use ring::signature::{Ed25519KeyPair, KeyPair};
use ring::{
    rand,
    signature::{self},
};

#[derive(Debug)]
pub enum AccountError {
    CryptoError,
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
    key_pair: Ed25519KeyPair,
}

impl Account {
    pub fn new() -> Result<Self, AccountError> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;

        let pb = key_pair
            .public_key()
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        log::info!("Generated a new account. Public key: {:x?}", pb);

        Ok(Account { key_pair })
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.key_pair.sign(message).as_ref().to_vec()
    }

    pub fn public_key(&self) -> &[u8] {
        self.key_pair.public_key().as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{account::Account, signature::verify};

    #[test]
    fn test_create_account() {
        let account = Account::new().unwrap();
        let message = b"Fear is the mind-killer. I will face my fear. Only I will remain.";

        // Encrypt the message
        let signature = account.sign(message);

        // Validate the signature
        let is_valid = verify(message, account.public_key(), &signature).is_ok();

        assert!(is_valid);
    }
}
