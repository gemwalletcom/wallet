use primitives::{Asset, AssetType, Chain, RecentActivityType};
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetList {
    Buy,
    Sell,
    Swap,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAssetNetworkDestination {
    Asset { asset: Asset },
    Assets { chain: Chain },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetAction {
    Open,
    Send,
    Receive,
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
            Self::Open => Vec::new(),
            Self::Send => vec![GemAssetFilter::Enabled, GemAssetFilter::HasBalance],
            Self::Receive => vec![GemAssetFilter::Enabled],
            Self::Buy => vec![GemAssetFilter::Enabled, GemAssetFilter::Buyable],
            Self::Sell => vec![GemAssetFilter::Enabled, GemAssetFilter::Sellable],
            Self::SwapPay => vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable, GemAssetFilter::HasAvailableBalance],
            Self::SwapReceive => vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable],
        }
    }

    pub fn recent_activity_types(&self) -> Vec<RecentActivityType> {
        match self {
            Self::SwapPay | Self::SwapReceive => vec![RecentActivityType::SwapSelect, RecentActivityType::Swap],
            Self::Open | Self::Send | Self::Receive | Self::Buy | Self::Sell => RecentActivityType::iter().collect(),
        }
    }
}

impl GemAssetAction {
    pub fn recent_activity_type(&self, asset: &Asset) -> Option<RecentActivityType> {
        match self {
            Self::Open => Some(match asset.asset_type {
                AssetType::PERPETUAL => RecentActivityType::Perpetual,
                _ => RecentActivityType::Search,
            }),
            Self::Send => None,
            Self::Receive => Some(RecentActivityType::Receive),
            Self::Buy => Some(RecentActivityType::FiatBuy),
            Self::Sell => Some(RecentActivityType::FiatSell),
            Self::SwapPay | Self::SwapReceive => Some(RecentActivityType::SwapSelect),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Asset, AssetType, GemAssetAction, GemAssetFilter, RecentActivityType};
    use primitives::Chain;

    #[test]
    fn test_every_recorded_recent_type_is_shown_by_the_same_action() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let perpetual = Asset {
            asset_type: AssetType::PERPETUAL,
            ..Asset::from_chain(Chain::HyperCore)
        };
        let actions = [
            GemAssetAction::Open,
            GemAssetAction::Send,
            GemAssetAction::Receive,
            GemAssetAction::Buy,
            GemAssetAction::Sell,
            GemAssetAction::SwapPay,
            GemAssetAction::SwapReceive,
        ];
        for action in actions {
            for asset in [&asset, &perpetual] {
                if let Some(recorded) = action.recent_activity_type(asset) {
                    assert!(action.recent_activity_types().contains(&recorded), "{action:?}");
                }
            }
        }
        assert_eq!(GemAssetAction::Send.recent_activity_type(&asset), None);
        assert_eq!(GemAssetAction::Open.recent_activity_type(&perpetual), Some(RecentActivityType::Perpetual));
        assert_eq!(GemAssetAction::Open.recent_activity_type(&asset), Some(RecentActivityType::Search));
        assert!(!GemAssetAction::SwapPay.recent_activity_types().contains(&RecentActivityType::Receive));
    }

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
