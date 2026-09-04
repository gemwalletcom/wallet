use crate::models::custom_types::{GemBigInt, GemBigUint};
use primitives::{AssetId, asset_balance::BalanceMetadata};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBalanceRequirement {
    pub required: GemBigInt,
    pub available: GemBigInt,
    pub shortfall: GemBigInt,
}

impl GemBalanceRequirement {
    pub fn new(required: GemBigInt, available: GemBigInt) -> Self {
        let shortfall = (&required - &available).max(GemBigInt::ZERO);
        Self { required, available, shortfall }
    }
}

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

impl GemAssetBalance {
    pub fn votes(&self) -> u32 {
        self.metadata.as_ref().map(|metadata| metadata.votes).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemBalanceRow {
    Available { value: GemBigUint },
    Staked { value: GemBigUint },
    Earn { value: GemBigUint },
    PendingUnconfirmed { value: GemBigUint },
    Reserved { value: GemBigUint, url: Option<String> },
}
