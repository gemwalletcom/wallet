use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct AssetMetaData {
    #[serde(rename = "isEnabled")]
    pub is_enabled: bool,
    #[serde(rename = "isBalanceEnabled")]
    pub is_balance_enabled: bool,
    #[serde(rename = "isBuyEnabled")]
    pub is_buy_enabled: bool,
    #[serde(rename = "isSellEnabled")]
    pub is_sell_enabled: bool,
    #[serde(rename = "isSwapEnabled")]
    pub is_swap_enabled: bool,
    #[serde(rename = "isStakeEnabled")]
    pub is_stake_enabled: bool,
    #[serde(rename = "isEarnEnabled")]
    pub is_earn_enabled: bool,
    #[serde(rename = "isPinned")]
    pub is_pinned: bool,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "stakingApr")]
    pub staking_apr: Option<f64>,
    #[serde(rename = "earnApr")]
    pub earn_apr: Option<f64>,
    #[serde(rename = "rankScore")]
    pub rank_score: i32,
}
