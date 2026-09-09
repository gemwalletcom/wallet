use crate::models::custom_types::{GemBigInt, GemBigUint};
use number_formatter::BigNumberFormatter;
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
        available: GemBigUint,
        frozen: GemBigUint,
        reserved: GemBigUint,
        pending_unconfirmed: GemBigUint,
    },
    Token {
        available: GemBigUint,
    },
    Stake {
        staked: GemBigUint,
        pending: GemBigUint,
        rewards: GemBigUint,
        locked: GemBigUint,
        frozen: GemBigUint,
        metadata: Option<BalanceMetadata>,
    },
    Earn {
        balance: GemBigUint,
    },
    Perpetual {
        available: GemBigUint,
        reserved: GemBigUint,
        withdrawable: GemBigUint,
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
    pub is_active: bool,
}

impl GemAssetBalance {
    pub fn zero(asset_id: AssetId) -> Self {
        Self {
            asset_id,
            available: GemBigUint::ZERO,
            frozen: GemBigUint::ZERO,
            locked: GemBigUint::ZERO,
            staked: GemBigUint::ZERO,
            pending: GemBigUint::ZERO,
            pending_unconfirmed: GemBigUint::ZERO,
            rewards: GemBigUint::ZERO,
            reserved: GemBigUint::ZERO,
            withdrawable: GemBigUint::ZERO,
            earn: GemBigUint::ZERO,
            metadata: None,
            is_active: true,
        }
    }

    pub fn votes(&self) -> u32 {
        self.metadata.as_ref().map(|metadata| metadata.votes).unwrap_or_default()
    }

    pub fn applying(&self, update: &GemBalanceUpdate) -> Self {
        let mut balance = self.clone();
        balance.is_active = update.is_active;
        match &update.update_type {
            GemBalanceUpdateType::Coin {
                available,
                frozen,
                reserved,
                pending_unconfirmed,
            } => {
                balance.available = available.clone();
                balance.frozen = frozen.clone();
                balance.reserved = reserved.clone();
                balance.pending_unconfirmed = pending_unconfirmed.clone();
            }
            GemBalanceUpdateType::Token { available } => balance.available = available.clone(),
            GemBalanceUpdateType::Stake {
                staked,
                pending,
                rewards,
                locked,
                frozen,
                metadata,
            } => {
                balance.staked = staked.clone();
                balance.pending = pending.clone();
                balance.rewards = rewards.clone();
                balance.locked = locked.clone();
                balance.frozen = frozen.clone();
                if metadata.is_some() {
                    balance.metadata = metadata.clone();
                }
            }
            GemBalanceUpdateType::Earn { balance: earn } => balance.earn = earn.clone(),
            GemBalanceUpdateType::Perpetual { available, reserved, withdrawable } => {
                balance.available = available.clone();
                balance.reserved = reserved.clone();
                balance.withdrawable = withdrawable.clone();
            }
        }
        balance
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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBalanceRecord {
    pub asset_id: AssetId,
    pub available: GemBalanceValue,
    pub frozen: GemBalanceValue,
    pub locked: GemBalanceValue,
    pub staked: GemBalanceValue,
    pub pending: GemBalanceValue,
    pub pending_unconfirmed: GemBalanceValue,
    pub rewards: GemBalanceValue,
    pub reserved: GemBalanceValue,
    pub withdrawable: GemBalanceValue,
    pub earn: GemBalanceValue,
    pub metadata: Option<BalanceMetadata>,
    pub is_active: bool,
}

impl GemBalanceRecord {
    pub fn new(balance: GemAssetBalance, decimals: u32) -> Self {
        let value = |amount: GemBigUint| GemBalanceValue {
            amount: BigNumberFormatter::value_as_f64(&amount.to_string(), decimals).unwrap_or_default(),
            value: amount,
        };
        Self {
            asset_id: balance.asset_id,
            available: value(balance.available),
            frozen: value(balance.frozen),
            locked: value(balance.locked),
            staked: value(balance.staked),
            pending: value(balance.pending),
            pending_unconfirmed: value(balance.pending_unconfirmed),
            rewards: value(balance.rewards),
            reserved: value(balance.reserved),
            withdrawable: value(balance.withdrawable),
            earn: value(balance.earn),
            metadata: balance.metadata,
            is_active: balance.is_active,
        }
    }
}
