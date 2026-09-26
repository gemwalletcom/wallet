use crate::formatted_number::GemFormattedNumber;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::list::GemListRowTitle;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, AssetData, AssetId, Balance, asset_balance::BalanceMetadata};

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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
    pub fn new(asset_id: AssetId, balance: &Balance, is_active: bool) -> Self {
        Self {
            asset_id,
            available: balance.available.clone(),
            frozen: balance.frozen.clone(),
            locked: balance.locked.clone(),
            staked: balance.staked.clone(),
            pending: balance.pending.clone(),
            pending_unconfirmed: balance.pending_unconfirmed.clone(),
            rewards: balance.rewards.clone(),
            reserved: balance.reserved.clone(),
            withdrawable: balance.withdrawable.clone(),
            earn: balance.earn.clone(),
            metadata: balance.metadata.clone(),
            is_active,
        }
    }

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

    pub fn total(&self) -> GemBigUint {
        &self.available + &self.frozen + &self.locked + &self.staked + &self.pending + &self.rewards + &self.earn
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

impl From<&AssetData> for GemAssetBalance {
    fn from(data: &AssetData) -> Self {
        Self::new(data.asset.id.clone(), &data.balance, data.metadata.is_active)
    }
}

impl From<AssetBalance> for GemAssetBalance {
    fn from(balance: AssetBalance) -> Self {
        Self::new(balance.asset_id, &balance.balance, balance.is_active)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemBalanceRowValue {
    Amount { amount: GemFormattedNumber },
    Apr { apr: Option<GemFormattedNumber> },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetBalanceRow {
    pub row: GemBalanceRow,
    pub value: GemBalanceRowValue,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemBalanceRow {
    Available { value: GemBigUint },
    Staked { value: GemBigUint },
    Earn { value: GemBigUint },
    PendingUnconfirmed { value: GemBigUint },
    Reserved { value: GemBigUint, url: Option<String> },
}

#[uniffi::export]
impl GemBalanceRow {
    pub fn title(&self) -> GemListRowTitle {
        match self {
            Self::Available { .. } => GemListRowTitle::Available,
            Self::Staked { .. } => GemListRowTitle::Stake,
            Self::Earn { .. } => GemListRowTitle::Earn,
            Self::PendingUnconfirmed { .. } => GemListRowTitle::PendingUnconfirmed,
            Self::Reserved { .. } => GemListRowTitle::Reserved,
        }
    }
}

impl GemBalanceRow {
    pub fn value(&self) -> GemBigUint {
        match self {
            Self::Available { value } | Self::Staked { value } | Self::Earn { value } | Self::PendingUnconfirmed { value } | Self::Reserved { value, .. } => value.clone(),
        }
    }
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

impl From<GemBalanceRecord> for GemAssetBalance {
    fn from(record: GemBalanceRecord) -> Self {
        Self {
            asset_id: record.asset_id,
            available: record.available.value,
            frozen: record.frozen.value,
            locked: record.locked.value,
            staked: record.staked.value,
            pending: record.pending.value,
            pending_unconfirmed: record.pending_unconfirmed.value,
            rewards: record.rewards.value,
            reserved: record.reserved.value,
            withdrawable: record.withdrawable.value,
            earn: record.earn.value,
            metadata: record.metadata,
            is_active: record.is_active,
        }
    }
}

impl GemBalanceRecord {
    pub fn new(balance: GemAssetBalance, decimals: u32) -> Self {
        let value = |amount: GemBigUint| GemBalanceValue {
            amount: BigNumberFormatter::f64_value(&amount, decimals),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_requirement_has_no_shortfall_once_the_balance_covers_it() {
        let short = GemBalanceRequirement::new(GemBigInt::from(100), GemBigInt::from(40));
        let covered = GemBalanceRequirement::new(GemBigInt::from(100), GemBigInt::from(140));

        assert_eq!(short.shortfall, GemBigInt::from(60));
        assert_eq!(covered.shortfall, GemBigInt::ZERO, "a surplus is not a negative shortfall");
    }

    #[test]
    fn test_a_coin_update_leaves_the_staking_fields_alone() {
        let staked = GemAssetBalance::mock().applying(&GemBalanceUpdate::mock(GemBalanceUpdateType::Stake {
            staked: GemBigUint::from(70u32),
            pending: GemBigUint::from(5u32),
            rewards: GemBigUint::from(3u32),
            locked: GemBigUint::from(2u32),
            frozen: GemBigUint::from(1u32),
            metadata: Some(BalanceMetadata { votes: 9, ..BalanceMetadata::default() }),
        }));

        let after_coin = staked.applying(&GemBalanceUpdate::mock(GemBalanceUpdateType::Coin {
            available: GemBigUint::from(11u32),
            frozen: GemBigUint::from(12u32),
            reserved: GemBigUint::from(13u32),
            pending_unconfirmed: GemBigUint::from(14u32),
        }));

        assert_eq!(after_coin.available, GemBigUint::from(11u32));
        assert_eq!(after_coin.frozen, GemBigUint::from(12u32));
        assert_eq!(after_coin.staked, GemBigUint::from(70u32));
        assert_eq!(after_coin.rewards, GemBigUint::from(3u32));
        assert_eq!(after_coin.votes(), 9);
    }

    #[test]
    fn test_a_stake_update_without_metadata_keeps_the_metadata_it_had() {
        let with_metadata = GemAssetBalance::mock().applying(&GemBalanceUpdate::mock(GemBalanceUpdateType::Stake {
            staked: GemBigUint::from(70u32),
            pending: GemBigUint::ZERO,
            rewards: GemBigUint::ZERO,
            locked: GemBigUint::ZERO,
            frozen: GemBigUint::ZERO,
            metadata: Some(BalanceMetadata { votes: 9, ..BalanceMetadata::default() }),
        }));

        let without = with_metadata.applying(&GemBalanceUpdate::mock(GemBalanceUpdateType::Stake {
            staked: GemBigUint::from(80u32),
            pending: GemBigUint::ZERO,
            rewards: GemBigUint::ZERO,
            locked: GemBigUint::ZERO,
            frozen: GemBigUint::ZERO,
            metadata: None,
        }));

        assert_eq!(without.staked, GemBigUint::from(80u32));
        assert_eq!(without.votes(), 9);
    }

    #[test]
    fn test_a_perpetual_update_writes_the_withdrawable_a_token_update_never_touches() {
        let perpetual = GemAssetBalance::mock().applying(&GemBalanceUpdate::mock(GemBalanceUpdateType::Perpetual {
            available: GemBigUint::from(5u32),
            reserved: GemBigUint::from(6u32),
            withdrawable: GemBigUint::from(7u32),
        }));
        let token = perpetual.applying(&GemBalanceUpdate::mock(GemBalanceUpdateType::Token { available: GemBigUint::from(9u32) }));

        assert_eq!(token.available, GemBigUint::from(9u32));
        assert_eq!(token.reserved, GemBigUint::from(6u32));
        assert_eq!(token.withdrawable, GemBigUint::from(7u32));
    }

    #[test]
    fn test_an_inactive_update_deactivates_the_balance() {
        let inactive = GemBalanceUpdate {
            is_active: false,
            ..GemBalanceUpdate::mock(GemBalanceUpdateType::Token { available: GemBigUint::from(1u32) })
        };

        assert!(!GemAssetBalance::mock().applying(&inactive).is_active);
    }

    #[test]
    fn test_a_record_reads_each_value_at_the_asset_decimals() {
        let earned = GemAssetBalance::mock().applying(&GemBalanceUpdate::mock(GemBalanceUpdateType::Earn { balance: GemBigUint::from(2_500_000u32) }));

        let record = GemBalanceRecord::new(earned, 6);

        assert_eq!(record.earn.value, GemBigUint::from(2_500_000u32));
        assert_eq!(record.earn.amount, 2.5);
        assert_eq!(record.available.amount, 0.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemAssetConfiguration {
    pub is_enabled: Option<bool>,
    pub is_pinned: Option<bool>,
}
