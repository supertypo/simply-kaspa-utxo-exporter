pub struct ScriptUtxoCount {
    pub script_public_key: Vec<u8>,
    pub script_public_key_address: Option<String>,
    pub count: i64,
}
