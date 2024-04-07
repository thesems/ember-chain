use sha2::{Digest, Sha256};

pub type Address = String;
pub type HashResult = [u8; 32];


pub fn sha256(bytes: &[u8]) -> HashResult {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

pub fn vec_u8_from_hex(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|x| {
            hex.get(x..x + 2)
                .map(|s| u8::from_str_radix(s, 16).unwrap())
                .unwrap()
        })
        .collect()
}

#[cfg(test)]
mod test {
    use crate::crypto::hash_utils::vec_u8_from_hex;

    #[test]
    fn test_vec_u8_from_hex() {
        assert_eq!(vec_u8_from_hex("0204FF"), [2, 4, 255]);
    }
}
