use ethnum::U256;

use crate::{block_header::BlockHeader, hash_utils::HashResult};

pub fn target_from_difficulty_bit(bit: u8) -> U256 {
    U256::new(2).checked_pow(256 - bit as u32).unwrap()
}

pub fn compare_difficulty(target: U256, hash_int: U256) -> bool {
    if hash_int <= target {
        return true;
    }
    false
}

pub fn proof_of_work(difficulty: u8, block: &mut BlockHeader) -> Option<HashResult> {
    let mut block_hash = block.finalize();
    let target = target_from_difficulty_bit(difficulty);

    for i in 0..u32::MAX {
        let hash_int = U256::from_be_bytes(block_hash);
        if compare_difficulty(target, hash_int) {
            return Some(block_hash);
        }

        block.nonce = i;
        block_hash = block.finalize();
    }
    None
}
