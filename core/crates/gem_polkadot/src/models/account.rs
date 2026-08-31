use num_bigint::{BigInt, BigUint};
use primitives::Balance;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_bigint_from_str, deserialize_u64_from_str, serialize_bigint};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotAccountBalance {
    #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
    pub free: BigInt,
    #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
    pub reserved: BigInt,
    #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
    pub frozen: BigInt,
    #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
    pub transferable: BigInt,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub nonce: u64,
}

impl PolkadotAccountBalance {
    pub fn balance(&self) -> Balance {
        let zero = BigInt::from(0);
        let free = self.free.clone().max(zero.clone());
        let available = self.transferable.clone().clamp(zero, free.clone());

        Balance {
            available: BigUint::try_from(available.clone()).unwrap_or_default(),
            frozen: BigUint::try_from(free - available).unwrap_or_default(),
            reserved: BigUint::try_from(self.reserved.clone()).unwrap_or_default(),
            ..Balance::coin_balance(BigUint::from(0u32))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_balance_info() {
        let balance: PolkadotAccountBalance = serde_json::from_str(r#"{"nonce":"48","free":"31415926535","reserved":"0","frozen":"31415926535","transferable":"0"}"#).unwrap();

        assert_eq!(balance.free, BigInt::from(31415926535_u64));
        assert_eq!(balance.reserved, BigInt::from(0));
        assert_eq!(balance.frozen, BigInt::from(31415926535_u64));
        assert_eq!(balance.transferable, BigInt::from(0));
        assert_eq!(balance.nonce, 48);

        let old_sidecar = serde_json::from_str::<PolkadotAccountBalance>(r#"{"nonce":"1","free":"1000","reserved":"100","frozen":"frozen does not exist for this runtime"}"#);

        assert!(old_sidecar.is_err());
    }

    #[test]
    fn test_balance() {
        let vesting_lock = PolkadotAccountBalance::mock_with_balances(31415926535, 0, 31415926535, 0).balance();
        assert_eq!(vesting_lock.available, BigUint::from(0u32));
        assert_eq!(vesting_lock.frozen, BigUint::from(31415926535_u64));
        assert_eq!(vesting_lock.reserved, BigUint::from(0u32));

        let reserved_exceeds_free = PolkadotAccountBalance::mock_with_balances(2501175677207, 84509609960724902, 84500000000000000, 2501075677207).balance();
        assert_eq!(reserved_exceeds_free.available, BigUint::from(2501075677207_u64));
        assert_eq!(reserved_exceeds_free.frozen, BigUint::from(100000000_u64));
        assert_eq!(reserved_exceeds_free.reserved, BigUint::from(84509609960724902_u64));

        let transferable_above_free = PolkadotAccountBalance::mock_with_balances(1000, 0, 0, 2000).balance();
        assert_eq!(transferable_above_free.available, BigUint::from(1000u32));
        assert_eq!(transferable_above_free.frozen, BigUint::from(0u32));
    }
}
