use crate::models::custom_types::GemBigUint;
use primitives::{AssetId, asset_balance::BalanceMetadata};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBalanceValue {
    pub value: GemBigUint,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemBalanceUpdateType {
    Coin {
        available: GemBalanceValue,
        frozen: GemBalanceValue,
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
    Perpetual {
        available: GemBalanceValue,
        reserved: GemBalanceValue,
        withdrawable: GemBalanceValue,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBalanceUpdate {
    pub asset_id: AssetId,
    pub update_type: GemBalanceUpdateType,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetBalance {
    pub asset_id: AssetId,
    pub available: GemBigUint,
    pub frozen: GemBigUint,
    pub locked: GemBigUint,
    pub staked: GemBigUint,
    pub pending: GemBigUint,
    pub pending_unconfirmed: GemBigUint,
    pub rewards: GemBigUint,
    pub reserved: GemBigUint,
    pub withdrawable: GemBigUint,
    pub earn: GemBigUint,
    pub metadata: Option<BalanceMetadata>,
}
