use crate::AssetId;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressBalances {
    pub coin: AssetBalance,
    pub staking: Option<AssetBalance>,
    pub assets: Vec<AssetBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetBalance {
    pub asset_id: AssetId,
    pub balance: Balance,
    pub is_active: bool,
}

impl AssetBalance {
    pub fn new(asset_id: AssetId, balance: BigUint) -> Self {
        Self {
            asset_id,
            balance: Balance::coin_balance(balance),
            is_active: true,
        }
    }

    pub fn new_zero_balance(asset_id: AssetId) -> Self {
        Self::new(asset_id, BigUint::from(0u32))
    }

    pub fn new_balance(asset_id: AssetId, balance: Balance) -> Self {
        Self {
            asset_id,
            balance,
            is_active: true,
        }
    }

    pub fn new_with_active(asset_id: AssetId, balance: Balance, is_active: bool) -> Self {
        Self { asset_id, balance, is_active }
    }

    pub fn new_staking(asset_id: AssetId, staked: BigUint, pending: BigUint, rewards: BigUint) -> Self {
        Self {
            asset_id,
            balance: Balance::stake_balance(staked, pending, Some(rewards)),
            is_active: true,
        }
    }
    pub fn new_earn(asset_id: AssetId, earn: BigUint) -> Self {
        Self {
            asset_id,
            balance: Balance {
                earn,
                ..Balance::coin_balance(BigUint::from(0u32))
            },
            is_active: true,
        }
    }

    pub fn new_staking_with_metadata(asset_id: AssetId, staked: BigUint, pending: BigUint, rewards: BigUint, metadata: BalanceMetadata) -> Self {
        Self {
            asset_id,
            balance: Balance::stake_balance_with_metadata(staked, pending, Some(rewards), Some(metadata)),
            is_active: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub available: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub frozen: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub locked: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub staked: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub pending: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub pending_unconfirmed: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub rewards: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub reserved: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub earn: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint")]
    pub withdrawable: BigUint,
    pub metadata: Option<BalanceMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct BalanceMetadata {
    pub votes: u32,
    pub energy_available: u32,
    pub energy_total: u32,
    pub bandwidth_available: u32,
    pub bandwidth_total: u32,
}

impl Balance {
    pub fn coin_balance(available: BigUint) -> Self {
        Self {
            available,
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked: BigUint::from(0u32),
            pending: BigUint::from(0u32),
            pending_unconfirmed: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            reserved: BigUint::from(0u32),
            earn: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
            metadata: None,
        }
    }

    pub fn with_pending_unconfirmed(available: BigUint, pending_unconfirmed: BigUint) -> Self {
        Self {
            available,
            pending_unconfirmed,
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked: BigUint::from(0u32),
            pending: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            reserved: BigUint::from(0u32),
            earn: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
            metadata: None,
        }
    }

    pub fn with_reserved(available: BigUint, reserved: BigUint) -> Self {
        Self {
            available,
            reserved,
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked: BigUint::from(0u32),
            pending: BigUint::from(0u32),
            pending_unconfirmed: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            earn: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
            metadata: None,
        }
    }

    pub fn stake_balance(staked: BigUint, pending: BigUint, rewards: Option<BigUint>) -> Self {
        Self::stake_balance_with_metadata(staked, pending, rewards, None)
    }

    pub fn stake_balance_with_metadata(staked: BigUint, pending: BigUint, rewards: Option<BigUint>, metadata: Option<BalanceMetadata>) -> Self {
        Self {
            available: BigUint::from(0u32),
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked,
            pending,
            pending_unconfirmed: BigUint::from(0u32),
            rewards: rewards.unwrap_or(BigUint::from(0u32)),
            reserved: BigUint::from(0u32),
            earn: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_balance_json_uses_decimal_strings() {
        let balance = Balance::with_pending_unconfirmed(BigUint::from(4_386_329_430u64), BigUint::from(12u32));
        let value = serde_json::to_value(&balance).unwrap();

        assert_eq!(
            value,
            json!({
                "available": "4386329430",
                "frozen": "0",
                "locked": "0",
                "staked": "0",
                "pending": "0",
                "pendingUnconfirmed": "12",
                "rewards": "0",
                "reserved": "0",
                "earn": "0",
                "withdrawable": "0",
                "metadata": null
            })
        );
    }
}
