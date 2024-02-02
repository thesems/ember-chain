pub fn target_from_difficulty_bit(bit: u32) -> ethnum::U256 {
    ethnum::U256::from(2u32).pow(256 - bit)
}

pub fn compare_difficulty(target: ethnum::U256, hash_int: ethnum::U256) -> bool {
    if hash_int.lt(&target) {
        return true;
    }
    false
}
