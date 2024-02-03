pub fn target_from_difficulty_bit(bit: u32) -> u64 {
    u64::from(2u32).pow(64 - bit)
}

pub fn compare_difficulty(target: u64, hash_int: u64) -> bool {
    if hash_int < target {
        return true;
    }
    false
}
