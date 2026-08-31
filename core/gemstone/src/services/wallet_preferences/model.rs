#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemDiscoveryStep {
    Assets,
    Transactions,
    Nfts,
}
