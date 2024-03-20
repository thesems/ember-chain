// Reference implementation:
// https://medium.com/coinmonks/merkle-tree-a-simple-explanation-and-implementation-48903442bc08

use crate::hash_utils::{sha256, HashResult};


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct ProofItem {
    pub hash: HashResult,
    pub direction: Direction,
}

/// Get leaf Direction (left/right) based on its array index.
pub fn get_leaf_direction(hash: &HashResult, hashes: &[Vec<HashResult>]) -> Direction {
    let idx = hashes[0].iter().position(|x| x == hash).unwrap();
    if idx % 2 == 0 {
        Direction::Left
    } else {
        Direction::Right
    }
}

/// Calculate merkle root of the hashes.
pub fn generate_merkle_root(mut hashes: Vec<HashResult>) -> HashResult {
    if hashes.len() % 2 != 0 {
        hashes.push(*hashes.last().unwrap());
    }

    if hashes.is_empty() {
        return [0u8; 32];
    }

    let combined_hashes = _hash_by_two(&hashes);
    if combined_hashes.len() == 1 {
        return combined_hashes[0];
    }

    generate_merkle_root(combined_hashes)
}

/// Generate a tree of hashes until merkle root is reached.
pub fn generate_merkle_tree(mut hashes: Vec<HashResult>) -> Vec<Vec<HashResult>> {
    if hashes.is_empty() {
        return vec![vec![]];
    }

    ensure_even(&mut hashes);
    let mut tree = vec![hashes.clone()];
    let mut combined_hashes = hashes;

    loop {
        if combined_hashes.len() == 1 {
            break;
        }
        if combined_hashes.len() % 2 != 0 {
            combined_hashes.push(*combined_hashes.last().unwrap());
        }
        ensure_even(&mut combined_hashes);
        let mut next_combined = _hash_by_two(&combined_hashes);
        ensure_even(&mut next_combined);
        tree.push(next_combined.clone());
        combined_hashes = next_combined;
    }

    tree
}

fn ensure_even(hashes: &mut Vec<HashResult>) {
    if hashes.len() % 2 != 0 && hashes.len() > 1 {
        hashes.push(*hashes.last().unwrap());
    }
}

pub fn generate_merkle_proof(hash: HashResult, hashes: Vec<HashResult>) -> Vec<ProofItem> {
    if hashes.is_empty() {
        return vec![];
    }

    let tree = generate_merkle_tree(hashes);

    let mut proof = vec![ProofItem {
        hash,
        direction: get_leaf_direction(&hash, &tree),
    }];

    let mut idx = tree[0].iter().position(|h| *h == hash).unwrap();

    for level in tree.iter().take(tree.len() - 1) {
        let is_left = idx % 2 == 0;
        let sibling_direction = match is_left {
            true => Direction::Right,
            false => Direction::Left,
        };
        let sibling_idx = match is_left {
            true => idx + 1,
            false => idx - 1,
        };
        let sibling = ProofItem {
            hash: level[sibling_idx],
            direction: sibling_direction,
        };
        proof.push(sibling);
        idx /= 2;
    }

    proof
}

pub fn get_merkle_root_from_merkle_proof(proof: Vec<ProofItem>) -> HashResult {
    if proof.is_empty() {
        return [0u8; 32];
    }

    let root = proof
        .iter()
        .copied()
        .reduce(|hash_proof_1, hash_proof_2| {
            if hash_proof_2.direction == Direction::Right {
                let hash = sha256(&[hash_proof_1.hash, hash_proof_2.hash].concat());
                return ProofItem {
                    hash,
                    direction: Direction::Left,
                };
            }
            let hash = sha256(&[hash_proof_2.hash, hash_proof_1.hash].concat());
            ProofItem {
                hash,
                direction: Direction::Left,
            }
        })
        .unwrap();

    root.hash
}

fn _hash_by_two(hashes: &[HashResult]) -> Vec<HashResult> {
    let mut combined_hashes = vec![];
    for hash_pair in hashes.windows(2).step_by(2) {
        let hash = sha256(&[hash_pair[0], hash_pair[1]].concat());
        combined_hashes.push(hash);
    }
    combined_hashes
}


#[cfg(test)]
mod tests {
    use super::*;
    use ethnum::U256;

    #[test]
    fn test_merkletree_tree() {
        let hashes: Vec<[u8; 32]> = [
            "0x95cd603fe577fa9548ec0c9b50b067566fe07c8af6acba45f6196f3a15d511f6",
            "0x709b55bd3da0f5a838125bd0ee20c5bfdd7caba173912d4281cae816b79a201b",
            "0x27ca64c092a959c7edc525ed45e845b1de6a7590d173fd2fad9133c8a779a1e3",
            "0x1f3cb18e896256d7d6bb8c11a6ec71f005c75de05e39beae5d93bbd1e2c8b7a9",
            "0x41b637cfd9eb3e2f60f734f9ca44e5c1559c6f481d49d6ed6891f3e9a086ac78",
            "0xa8c0cce8bb067e91cf2766c26be4e5d7cfba3d3323dc19d08a834391a1ce5acf",
        ]
        .iter()
        .map(|x| U256::from_str_hex(x).unwrap().to_be_bytes())
        .collect();

        let hash_1 = sha256(&[hashes[0], hashes[1]].concat());
        let hash_2 = sha256(&[hashes[2], hashes[3]].concat());
        let hash_3 = sha256(&[hashes[4], hashes[5]].concat());
        let hash_4 = sha256(&[hash_1, hash_2].concat());
        let hash_5 = sha256(&[hash_3, hash_3].concat());
        let root = sha256(&[hash_4, hash_5].concat());

        let mut tree = vec![hashes.clone()];
        tree.push(vec![hash_1, hash_2, hash_3, hash_3]);
        tree.push(vec![hash_4, hash_5]);
        tree.push(vec![root]);

        assert_eq!(tree, generate_merkle_tree(hashes));
    }

    #[test]
    fn test_merkletree_root() {
        let hashes: Vec<[u8; 32]> = [
            "0x95cd603fe577fa9548ec0c9b50b067566fe07c8af6acba45f6196f3a15d511f6",
            "0x709b55bd3da0f5a838125bd0ee20c5bfdd7caba173912d4281cae816b79a201b",
            "0x27ca64c092a959c7edc525ed45e845b1de6a7590d173fd2fad9133c8a779a1e3",
            "0x1f3cb18e896256d7d6bb8c11a6ec71f005c75de05e39beae5d93bbd1e2c8b7a9",
            "0x41b637cfd9eb3e2f60f734f9ca44e5c1559c6f481d49d6ed6891f3e9a086ac78",
        ]
        .iter()
        .map(|x| U256::from_str_hex(x).unwrap().to_be_bytes())
        .collect();

        let hash_1 = sha256(&[hashes[0], hashes[1]].concat());
        let hash_2 = sha256(&[hashes[2], hashes[3]].concat());
        let hash_3 = sha256(&[hashes[4], hashes[4]].concat());
        let hash_4 = sha256(&[hash_1, hash_2].concat());
        let hash_5 = sha256(&[hash_3, hash_3].concat());
        let root = sha256(&[hash_4, hash_5].concat());

        assert_eq!(root, generate_merkle_root(hashes));
    }

    #[test]
    fn test_merkletree_proof() {
        let hashes: Vec<[u8; 32]> = [
            "0x95cd603fe577fa9548ec0c9b50b067566fe07c8af6acba45f6196f3a15d511f6",
            "0x709b55bd3da0f5a838125bd0ee20c5bfdd7caba173912d4281cae816b79a201b",
            "0x27ca64c092a959c7edc525ed45e845b1de6a7590d173fd2fad9133c8a779a1e3",
            "0x1f3cb18e896256d7d6bb8c11a6ec71f005c75de05e39beae5d93bbd1e2c8b7a9",
            "0x41b637cfd9eb3e2f60f734f9ca44e5c1559c6f481d49d6ed6891f3e9a086ac78",
            "0xa8c0cce8bb067e91cf2766c26be4e5d7cfba3d3323dc19d08a834391a1ce5acf",
            "0xd20a624740ce1b7e2c74659bb291f665c021d202be02d13ce27feb067eeec837",
            "0x281b9dba10658c86d0c3c267b82b8972b6c7b41285f60ce2054211e69dd89e15",
            "0xdf743dd1973e1c7d46968720b931af0afa8ec5e8412f9420006b7b4fa660ba8d",
            "0x3e812f40cd8e4ca3a92972610409922dedf1c0dbc68394fcb1c8f188a42655e2",
            "0x3ebc2bd1d73e4f2f1f2af086ad724c98c8030f74c0c2be6c2d6fd538c711f35c",
            "0x9789f4e2339193149452c1a42cded34f7a301a13196cd8200246af7cc1e33c3b",
            "0xaefe99f12345aabc4aa2f000181008843c8abf57ccf394710b2c48ed38e1a66a",
            "0x64f662d104723a4326096ffd92954e24f2bf5c3ad374f04b10fcc735bc901a4d",
            "0x95a73895c9c6ee0fadb8d7da2fac25eb523fc582dc12c40ec793f0c1a70893b4",
            "0x315987563da5a1f3967053d445f73107ed6388270b00fb99a9aaa26c56ecba2b",
            "0x09caa1de14f86c5c19bf53cadc4206fd872a7bf71cda9814b590eb8c6e706fbb",
            "0x9d04d59d713b607c81811230645ce40afae2297f1cdc1216c45080a5c2e86a5a",
            "0xab8a58ff2cf9131f9730d94b9d67f087f5d91aebc3c032b6c5b7b810c47e0132",
            "0xc7c3f15b67d59190a6bbe5d98d058270aee86fe1468c73e00a4e7dcc7efcd3a0",
            "0x27ef2eaa77544d2dd325ce93299fcddef0fae77ae72f510361fa6e5d831610b2",
        ]
        .iter()
        .map(|x| U256::from_str_hex(x).unwrap().to_be_bytes())
        .collect();

        let proof = generate_merkle_proof(hashes[4], hashes.clone());
        
        assert_eq!(proof[0].hash, hashes[4]);
        assert_eq!(proof[0].direction, Direction::Left);
        
        assert_eq!(proof[1].hash, hashes[5]);
        assert_eq!(proof[1].direction, Direction::Right);
    }

    #[test]
    fn test_merkle_root_from_proof() {
        let hashes: Vec<[u8; 32]> = [
            "0x95cd603fe577fa9548ec0c9b50b067566fe07c8af6acba45f6196f3a15d511f6",
            "0x709b55bd3da0f5a838125bd0ee20c5bfdd7caba173912d4281cae816b79a201b",
            "0x27ca64c092a959c7edc525ed45e845b1de6a7590d173fd2fad9133c8a779a1e3",
            "0x1f3cb18e896256d7d6bb8c11a6ec71f005c75de05e39beae5d93bbd1e2c8b7a9",
            "0x41b637cfd9eb3e2f60f734f9ca44e5c1559c6f481d49d6ed6891f3e9a086ac78",
            "0xa8c0cce8bb067e91cf2766c26be4e5d7cfba3d3323dc19d08a834391a1ce5acf",
            "0xd20a624740ce1b7e2c74659bb291f665c021d202be02d13ce27feb067eeec837",
            "0x281b9dba10658c86d0c3c267b82b8972b6c7b41285f60ce2054211e69dd89e15",
            "0xdf743dd1973e1c7d46968720b931af0afa8ec5e8412f9420006b7b4fa660ba8d",
            "0x3e812f40cd8e4ca3a92972610409922dedf1c0dbc68394fcb1c8f188a42655e2",
            "0x3ebc2bd1d73e4f2f1f2af086ad724c98c8030f74c0c2be6c2d6fd538c711f35c",
            "0x9789f4e2339193149452c1a42cded34f7a301a13196cd8200246af7cc1e33c3b",
            "0xaefe99f12345aabc4aa2f000181008843c8abf57ccf394710b2c48ed38e1a66a",
            "0x64f662d104723a4326096ffd92954e24f2bf5c3ad374f04b10fcc735bc901a4d",
            "0x95a73895c9c6ee0fadb8d7da2fac25eb523fc582dc12c40ec793f0c1a70893b4",
            "0x315987563da5a1f3967053d445f73107ed6388270b00fb99a9aaa26c56ecba2b",
            "0x09caa1de14f86c5c19bf53cadc4206fd872a7bf71cda9814b590eb8c6e706fbb",
            "0x9d04d59d713b607c81811230645ce40afae2297f1cdc1216c45080a5c2e86a5a",
            "0xab8a58ff2cf9131f9730d94b9d67f087f5d91aebc3c032b6c5b7b810c47e0132",
            "0xc7c3f15b67d59190a6bbe5d98d058270aee86fe1468c73e00a4e7dcc7efcd3a0",
            "0x27ef2eaa77544d2dd325ce93299fcddef0fae77ae72f510361fa6e5d831610b2",
        ]
        .iter()
        .map(|x| U256::from_str_hex(x).unwrap().to_be_bytes())
        .collect();

        let root = generate_merkle_root(hashes.clone());
        let proof = generate_merkle_proof(hashes[4], hashes.clone());
        let root_from_proof = get_merkle_root_from_merkle_proof(proof);

        assert_eq!(root, root_from_proof);
    }
}
