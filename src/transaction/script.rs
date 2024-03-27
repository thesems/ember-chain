use std::collections::VecDeque;

use crate::crypto::{
    hash_utils::{sha256, HashResult},
    signature::verify,
};

type StackItem = Vec<u8>;

#[derive(Debug, Copy, Clone)]
pub enum Operation {
    True = 81,
    Nop = 97,
    Verify = 105,
    Return = 106,
    Dup = 118,
    Equal = 135,
    EqualVerify = 136,
    Hash256 = 170,
    CheckSig = 172,
}

#[derive(Clone)]
pub enum Item {
    Data(StackItem),
    Operation(Operation),
}

#[derive(Clone)]
pub struct Script {
    items: Vec<Item>,
}
impl Script {
    pub fn new(items: Vec<Item>) -> Self {
        Self { items }
    }
    pub fn hash(&self) -> Vec<u8> {
        let mut result = vec![];
        for item in self.items.iter() {
            match item {
                Item::Data(data) => data.iter().for_each(|x| result.push(*x)),
                Item::Operation(op) => result.push(*op as u8),
            }
        }
        result
    }
}

pub struct ScriptRunner {
    stack: VecDeque<StackItem>,
    hashed_tx_data: HashResult,
}
impl ScriptRunner {
    pub fn new(hashed_tx_data: HashResult) -> Self {
        ScriptRunner {
            stack: VecDeque::new(),
            hashed_tx_data,
        }
    }
    pub fn execute_script(&mut self, items: Vec<Item>) -> bool {
        for item in items {
            match item {
                Item::Data(data) => self.push_stack(data),
                Item::Operation(op) => {
                    if !self.execute_operation(op) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn execute_operation(&mut self, op: Operation) -> bool {
        match op {
            Operation::True => {
                self.push_stack(vec![1]);
                true
            }
            Operation::Nop => true,
            Operation::Verify => self.verify(),
            Operation::Return => false,
            Operation::Dup => self.dup(),
            Operation::Hash256 => self.hash256(),
            Operation::Equal => self.equal(),
            Operation::EqualVerify => self.equal_verify(),
            Operation::CheckSig => self.check_signature(),
            _ => false,
        }
    }

    fn push_stack(&mut self, item: StackItem) {
        self.stack.push_front(item);
    }

    fn pop_stack(&mut self) -> Option<StackItem> {
        self.stack.pop_front()
    }

    fn first_stack(&self) -> Option<&StackItem> {
        self.stack.front()
    }

    fn dup(&mut self) -> bool {
        if let Some(item) = self.first_stack() {
            self.push_stack(item.clone());
            true
        } else {
            log::warn!("Script operation OP_DUP failed. Reason: stack empty.");
            false
        }
    }

    fn verify(&mut self) -> bool {
        if let Some(item) = self.pop_stack() {
            item == vec![1]
        } else {
            log::warn!("Script operation OP_VERIFY failed. Reason: stack empty.");
            false
        }
    }

    fn hash256(&mut self) -> bool {
        if let Some(item) = self.pop_stack() {
            self.push_stack(sha256(&item).to_vec());
            true
        } else {
            log::warn!("Script operation OP_HASH256 failed. Reason: stack empty.");
            false
        }
    }

    fn equal(&mut self) -> bool {
        if let Some(item1) = self.pop_stack() {
            if let Some(item2) = self.pop_stack() {
                self.push_stack(vec![(item1 == item2).into()]);
                true
            } else {
                log::warn!("Script operation OP_EQUALVERIFY failed. Reason: stack empty.");
                false
            }
        } else {
            log::warn!("Script operation OP_EQUALVERIFY failed. Reason: stack empty.");
            false
        }
    }

    fn equal_verify(&mut self) -> bool {
        if !self.equal() {
            return false;
        }
        self.verify()
    }

    fn check_signature(&mut self) -> bool {
        if let (Some(sig), Some(pubkey)) = (self.pop_stack(), self.pop_stack()) {
            verify(&sig, &pubkey, &self.hashed_tx_data).is_ok()
        } else {
            log::warn!("Script operation CheckSig failed. Reason: stack items missing.");
            false
        }
    }
}
