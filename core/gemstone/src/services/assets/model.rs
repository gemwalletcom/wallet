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

#[uniffi::export]
impl GemAssetAction {
    pub fn filters(&self) -> Vec<GemAssetFilter> {
        match self {
            Self::Send => vec![GemAssetFilter::Enabled, GemAssetFilter::HasBalance],
            Self::Buy => vec![GemAssetFilter::Enabled, GemAssetFilter::Buyable],
            Self::Sell => vec![GemAssetFilter::Enabled, GemAssetFilter::Sellable],
            Self::SwapPay => vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable, GemAssetFilter::HasAvailableBalance],
            Self::SwapReceive => vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GemAssetAction, GemAssetFilter};

    #[test]
    fn test_action_filters_gate_on_the_balance_each_action_can_spend() {
        assert_eq!(GemAssetAction::Send.filters(), vec![GemAssetFilter::Enabled, GemAssetFilter::HasBalance]);
        assert_eq!(
            GemAssetAction::SwapPay.filters(),
            vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable, GemAssetFilter::HasAvailableBalance]
        );
        assert_eq!(GemAssetAction::SwapReceive.filters(), vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable]);
        assert_eq!(GemAssetAction::Buy.filters(), vec![GemAssetFilter::Enabled, GemAssetFilter::Buyable]);
        assert_eq!(GemAssetAction::Sell.filters(), vec![GemAssetFilter::Enabled, GemAssetFilter::Sellable]);
    }
}
