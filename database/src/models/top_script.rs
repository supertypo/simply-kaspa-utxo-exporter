#[derive(Eq, PartialEq, Hash)]
pub struct TopScript {
    pub timestamp: i64,
    pub rank: i16,
    pub script_public_key: Vec<u8>,
    pub amount: i64,
}
