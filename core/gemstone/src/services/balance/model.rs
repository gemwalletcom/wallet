use primitives::{AssetId, asset_balance::BalanceMetadata};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBalanceValue {
    pub value: String,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemBalanceUpdateType {
    Coin {
        available: GemBalanceValue,
        reserved: GemBalanceValue,
        pending_unconfirmed: GemBalanceValue,
    },
    Token {
        available: GemBalanceValue,
    },
    Stake {
        staked: GemBalanceValue,
        pending: GemBalanceValue,
        rewards: GemBalanceValue,
        locked: GemBalanceValue,
        frozen: GemBalanceValue,
        metadata: Option<BalanceMetadata>,
    },
    Earn {
        balance: GemBalanceValue,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBalanceUpdate {
    pub asset_id: AssetId,
    pub update_type: GemBalanceUpdateType,
    pub is_active: bool,
}
