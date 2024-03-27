use super::script::Script;

#[derive(Clone)]
pub struct Output {
    value: u64,
    script_pub_key: Script,
}
impl Output {
    pub fn new(value: u64, script_pub_key: Script) -> Self {
        Self {
            value,
            script_pub_key,
        }
    }
    pub fn hash(&self) -> Vec<u8> {
        let mut result = vec![];
        for b in self.value.to_be_bytes() {
            result.push(b);
        }
        for b in self.script_pub_key.hash() {
            result.push(b);
        }
        result
    }
}
