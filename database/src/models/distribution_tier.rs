#[derive(Eq, PartialEq, Hash)]
pub struct DistributionTier {
    pub timestamp: i64,
    pub tier: i16,
    pub count: i64,
    pub amount: i64,
}
