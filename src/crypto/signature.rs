use ring::signature;

use super::account::AccountError;

pub fn verify(message: &[u8], public_key: &[u8], signature: &[u8]) -> Result<(), AccountError> {
    let public_key = ring::signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
    public_key.verify(message, signature)?;
    Ok(())
}
