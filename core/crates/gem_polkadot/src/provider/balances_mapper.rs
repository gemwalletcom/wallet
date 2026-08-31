use crate::models::account::PolkadotAccountBalance;
use num_bigint::{BigInt, BigUint};
use primitives::{AssetBalance, Balance, Chain};

pub fn map_coin_balance(balance: PolkadotAccountBalance) -> AssetBalance {
    let zero = BigInt::from(0);
    let transferable = balance
        .transferable
        .unwrap_or_else(|| {
            let untouchable = std::cmp::max(balance.frozen.unwrap_or_default() - &balance.reserved, zero.clone());
            &balance.free - untouchable
        })
        .clamp(zero.clone(), balance.free.clone());
    let frozen = std::cmp::max(&balance.free - &transferable, zero);

    AssetBalance::new_balance(
        Chain::Polkadot.as_asset_id(),
        Balance {
            available: BigUint::try_from(transferable).unwrap_or_default(),
            frozen: BigUint::try_from(frozen).unwrap_or_default(),
            reserved: BigUint::try_from(balance.reserved).unwrap_or_default(),
            ..Balance::coin_balance(BigUint::from(0u32))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account_balance(free: u64, reserved: u64, frozen: Option<u64>, transferable: Option<u64>) -> PolkadotAccountBalance {
        PolkadotAccountBalance {
            free: BigInt::from(free),
            reserved: BigInt::from(reserved),
            frozen: frozen.map(BigInt::from),
            transferable: transferable.map(BigInt::from),
            nonce: 0,
        }
    }

    #[test]
    fn test_map_coin_balance() {
        let vesting_lock = map_coin_balance(account_balance(31415926535, 0, Some(31415926535), Some(0)));
        assert_eq!(vesting_lock.asset_id, Chain::Polkadot.as_asset_id());
        assert_eq!(vesting_lock.balance.available, BigUint::from(0u32));
        assert_eq!(vesting_lock.balance.frozen, BigUint::from(31415926535_u64));
        assert_eq!(vesting_lock.balance.reserved, BigUint::from(0u32));

        let reserved_exceeds_free = map_coin_balance(account_balance(2501175677207, 84509609960724902, Some(84500000000000000), Some(2501075677207)));
        assert_eq!(reserved_exceeds_free.balance.available, BigUint::from(2501075677207_u64));
        assert_eq!(reserved_exceeds_free.balance.frozen, BigUint::from(100000000_u64));
        assert_eq!(reserved_exceeds_free.balance.reserved, BigUint::from(84509609960724902_u64));

        let fallback_lock_not_covered_by_reserved = map_coin_balance(account_balance(1000000000000, 100000000000, Some(400000000000), None));
        assert_eq!(fallback_lock_not_covered_by_reserved.balance.available, BigUint::from(700000000000_u64));
        assert_eq!(fallback_lock_not_covered_by_reserved.balance.frozen, BigUint::from(300000000000_u64));
        assert_eq!(fallback_lock_not_covered_by_reserved.balance.reserved, BigUint::from(100000000000_u64));

        let fallback_without_lock_fields = map_coin_balance(account_balance(1000000000000, 100000000000, None, None));
        assert_eq!(fallback_without_lock_fields.balance.available, BigUint::from(1000000000000_u64));
        assert_eq!(fallback_without_lock_fields.balance.frozen, BigUint::from(0u32));
        assert_eq!(fallback_without_lock_fields.balance.reserved, BigUint::from(100000000000_u64));

        let transferable_above_free = map_coin_balance(account_balance(1000, 0, None, Some(2000)));
        assert_eq!(transferable_above_free.balance.available, BigUint::from(1000u32));
        assert_eq!(transferable_above_free.balance.frozen, BigUint::from(0u32));
    }
}
