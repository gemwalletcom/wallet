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
            GemBalanceUpdateType::Perpetual {
                available,
                reserved,
                withdrawable,
            } => {
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

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;

    fn balance() -> GemAssetBalance {
        GemAssetBalance::zero(Chain::Ethereum.as_asset_id())
    }

    fn metadata(votes: u32) -> BalanceMetadata {
        BalanceMetadata {
            votes,
            energy_available: 1,
            energy_total: 2,
            bandwidth_available: 3,
            bandwidth_total: 4,
        }
    }

    fn update(update_type: GemBalanceUpdateType) -> GemBalanceUpdate {
        GemBalanceUpdate {
            asset_id: Chain::Ethereum.as_asset_id(),
            update_type,
            is_active: true,
        }
    }

    #[test]
    fn test_a_requirement_has_no_shortfall_once_the_balance_covers_it() {
        let short = GemBalanceRequirement::new(GemBigInt::from(100), GemBigInt::from(40));
        let covered = GemBalanceRequirement::new(GemBigInt::from(100), GemBigInt::from(140));

        assert_eq!(short.shortfall, GemBigInt::from(60));
        assert_eq!(covered.shortfall, GemBigInt::ZERO, "a surplus is not a negative shortfall");
    }

    #[test]
    fn test_a_coin_update_leaves_the_staking_fields_alone() {
        let staked = balance().applying(&update(GemBalanceUpdateType::Stake {
            staked: GemBigUint::from(70u32),
            pending: GemBigUint::from(5u32),
            rewards: GemBigUint::from(3u32),
            locked: GemBigUint::from(2u32),
            frozen: GemBigUint::from(1u32),
            metadata: Some(metadata(9)),
        }));

        let after_coin = staked.applying(&update(GemBalanceUpdateType::Coin {
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
        let with_metadata = balance().applying(&update(GemBalanceUpdateType::Stake {
            staked: GemBigUint::from(70u32),
            pending: GemBigUint::ZERO,
            rewards: GemBigUint::ZERO,
            locked: GemBigUint::ZERO,
            frozen: GemBigUint::ZERO,
            metadata: Some(metadata(9)),
        }));

        let without = with_metadata.applying(&update(GemBalanceUpdateType::Stake {
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
        let perpetual = balance().applying(&update(GemBalanceUpdateType::Perpetual {
            available: GemBigUint::from(5u32),
            reserved: GemBigUint::from(6u32),
            withdrawable: GemBigUint::from(7u32),
        }));
        let token = perpetual.applying(&update(GemBalanceUpdateType::Token {
            available: GemBigUint::from(9u32),
        }));

        assert_eq!(token.available, GemBigUint::from(9u32));
        assert_eq!(token.reserved, GemBigUint::from(6u32));
        assert_eq!(token.withdrawable, GemBigUint::from(7u32));
    }

    #[test]
    fn test_an_inactive_update_deactivates_the_balance() {
        let inactive = GemBalanceUpdate {
            is_active: false,
            ..update(GemBalanceUpdateType::Token {
                available: GemBigUint::from(1u32),
            })
        };

        assert!(!balance().applying(&inactive).is_active);
    }

    #[test]
    fn test_a_record_reads_each_value_at_the_asset_decimals() {
        let earned = balance().applying(&update(GemBalanceUpdateType::Earn {
            balance: GemBigUint::from(2_500_000u32),
        }));

        let record = GemBalanceRecord::new(earned, 6);

        assert_eq!(record.earn.value, GemBigUint::from(2_500_000u32));
        assert_eq!(record.earn.amount, 2.5);
        assert_eq!(record.available.amount, 0.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemBalanceResource {
    Energy,
    Bandwidth,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemBalanceResourceRow {
    pub resource: GemBalanceResource,
    pub text: String,
}
