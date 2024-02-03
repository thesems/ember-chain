struct Input {
    count: u8, // must be >= 1
    previous_output_txid: [u8; 32],
    previous_output_index: u32,
    length: u8,
    sequence: u32,
}
impl Input {
    fn serialize(&self) -> [u8; 42] {
        let mut data = [0u8; 42];
        data[0] = self.count;
        for i in 1..33 {
            data[i] = self.previous_output_txid[i];
        }
        let previous_output_index = self.previous_output_index.to_be_bytes();
        for i in 33..37 {
            data[i] = previous_output_index[i];
        }
        data[37] = self.length;
        let previous_output_index = self.sequence.to_be_bytes();
        for i in 38..42 {
            data[i] = previous_output_index[i];
        }
        data
    }
}
struct Output {
    count: u8,
    amount: u64,
    length: u8,
    script: [u8; 36],
}
impl Output {
    fn serialize(&self) -> [u8; 46] {
        let mut data = [0u8; 46];
        data[0] = self.count;
        let amount = self.amount.to_be_bytes();
        for i in 1..9 {
            data[i] = amount[i];
        }
        data[9] = self.length;
        for i in 10..46 {
            data[i] = self.script[i];
        }
        data
    }
}

pub enum TxVersion {
    Version1 = 0x01000000,
    Version2 = 0x02000000, // BIP68
}

pub struct Transaction {
    version: TxVersion,
    marker: u8, // must be 0 if witness is included
    flag: u8,   // must be non-0 if witness is included
    inputs: [u8; 42],
    outputs: [u8; 78],
    witness: [u8; 66],
    lock_time: u32,
}
impl Transaction {
    pub fn from(inputs: [u8; 42], outputs: [u8; 78]) -> Self {
        Self {
            version: TxVersion::Version1,
            marker: 0,
            flag: 1,
            inputs: [0u8; 42],
            outputs: [0u8; 78],
            witness: [0u8; 66],
            lock_time: 0,
        }
    }
    pub fn size(&self) -> usize {
        194        
    }
}
