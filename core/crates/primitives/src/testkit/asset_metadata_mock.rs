use crate::AssetMetaData;

impl AssetMetaData {
    pub fn mock() -> Self {
        Self {
            is_enabled: true,
            is_balance_enabled: true,
            is_buy_enabled: false,
            is_sell_enabled: false,
            is_swap_enabled: false,
            is_stake_enabled: false,
            is_earn_enabled: false,
            is_pinned: false,
            is_active: true,
            staking_apr: None,
            earn_apr: None,
            rank_score: 0,
        }
    }
}
