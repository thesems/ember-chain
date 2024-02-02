mod blockchain;
mod block;
mod block_header;
mod transaction;
mod pow_utils;

use blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new();
    blockchain.run();
}

