use blockchain_utxo::blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new();
    blockchain.run();
}

