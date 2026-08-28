#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetList {
    Buy,
    Sell,
    Swap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetAction {
    Send,
    Buy,
    Sell,
    SwapPay,
    SwapReceive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetFilter {
    Enabled,
    Buyable,
    Sellable,
    Swappable,
    HasBalance,
    HasAvailableBalance,
}
