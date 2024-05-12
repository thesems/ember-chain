use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

use crossbeam::channel::{Receiver, TryRecvError};
use ethnum::U256;
use rand::prelude::*;
use rand::Rng;

use crate::{block::block_header::BlockHeader, crypto::hash_utils::HashResult};

pub fn target_from_difficulty_bit(bit: u8) -> U256 {
    U256::new(2).checked_pow(256 - bit as u32).unwrap()
}

pub fn compare_difficulty(target: U256, hash_int: U256) -> bool {
    if hash_int <= target {
        return true;
    }
    false
}

pub fn get_random_range(min: u64, max: u64) -> u64 {
    let mut seed = [0u8; 32];
    let now_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let elapsed_time_bytes = now_since_epoch.as_micros().to_le_bytes();
    seed[0..16].copy_from_slice(&elapsed_time_bytes);
    let mut rng = StdRng::from_seed(seed);
    rng.gen_range(min..max)
}

/// Mines a block for a given difficulty.
///
pub fn proof_of_work(
    difficulty: u8,
    block_header: &mut BlockHeader,
    cancel_mine_rx: Receiver<()>,
    hash_count: &mut u32,
    fake_mining: bool,
) -> Option<HashResult> {
    let mut block_hash = block_header.finalize();
    let target = target_from_difficulty_bit(difficulty);
    let time_started = Instant::now();
    let wait_secs = get_random_range(8, 9);

    for i in 0..u32::MAX {
        if !fake_mining {
            let hash_int = U256::from_be_bytes(block_hash);
            if compare_difficulty(target, hash_int) {
                return Some(block_hash);
            }

            block_header.nonce = i;
            block_hash = block_header.finalize();
            *hash_count += 1;
        } else {
            std::thread::sleep(Duration::from_millis(1));
            if time_started.elapsed().as_secs() >= wait_secs {
                block_header.nonce = get_random_range(0, u32::MAX as u64) as u32;
                block_hash = block_header.finalize();
                return Some(block_hash);
            }
        }

        if i % 10000 == 0 {
            match cancel_mine_rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => return None,
                Err(TryRecvError::Empty) => {}
            }
        }
    }
    None
}
