use super::script::Script;

#[derive(Clone)]
pub struct Output {
    value: u64,
    script_pub_key: Script,
}
impl Output {
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
