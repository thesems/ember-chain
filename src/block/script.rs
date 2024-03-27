use std::collections::VecDeque;

use crate::crypto::{hash_utils::{sha256, HashResult}, signature::verify};

type StackItem = Vec<u8>;

#[derive(Debug, Copy, Clone)]
pub enum Operation {
    Dup = 118,
    Verify = 105,
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
            Operation::Dup => {
                if let Some(item) = self.first_stack() {
                    self.push_stack(item.clone());
                    true
                } else {
                    log::warn!("Script operation {:?} failed. Reason: stack empty.", op);
                    false
                }
            }
            Operation::Hash256 => {
                if let Some(item) = self.pop_stack() {
                    self.push_stack(sha256(&item).to_vec());
                    true
                } else {
                    log::warn!("Script operation {:?} failed. Reason: stack empty.", op);
                    false
                }
            }
            Operation::Equal => {
                if let Some(item1) = self.pop_stack() {
                    if let Some(item2) = self.pop_stack() {
                        item1 == item2
                    } else {
                        log::warn!("Script operation {:?} failed. Reason: stack empty.", op);
                        false
                    }
                } else {
                    log::warn!("Script operation {:?} failed. Reason: stack empty.", op);
                    false
                }
            }
            Operation::EqualVerify => {
                if let Some(item1) = self.pop_stack() {
                    if let Some(item2) = self.pop_stack() {
                        item1 == item2
                    } else {
                        log::warn!("Script operation {:?} failed. Reason: stack empty.", op);
                        false
                    }
                } else {
                    log::warn!("Script operation {:?} failed. Reason: stack empty.", op);
                    false
                }
            }
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

    fn check_signature(&mut self) -> bool {
        if let (Some(sig), Some(pubkey)) = (self.pop_stack(), self.pop_stack()) {
            verify(&sig, &pubkey, &self.hashed_tx_data).is_ok()
        } else {
            log::warn!("Script operation CheckSig failed. Reason: stack items missing.");
            false
        }
    }
}

