use num_bigint::BigUint;
use primitives::{AssetBalance, Balance, Chain};

use crate::{constants::STORAGE_AMOUNT_PER_BYTE, models::account::Account};

pub fn map_native_balance(account: &Account) -> AssetBalance {
    let reserved = (BigUint::from(account.storage_usage) * STORAGE_AMOUNT_PER_BYTE).min(account.amount.clone());
    let available = &account.amount - &reserved;
    AssetBalance::new_balance(Chain::Near.as_asset_id(), Balance::with_reserved(available, reserved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Account;

    #[test]
    fn test_map_native_balance() {
        let account = Account {
            amount: BigUint::from(1000000000000000000000000_u128),
            storage_usage: 182,
        };
        let result = map_native_balance(&account);

        assert_eq!(result.asset_id, Chain::Near.as_asset_id());
        assert_eq!(result.balance.available, BigUint::from(998180000000000000000000u128));
        assert_eq!(result.balance.reserved, BigUint::from(1820000000000000000000u128));

        let underfunded = map_native_balance(&Account {
            amount: BigUint::from(1u8),
            storage_usage: 182,
        });
        assert_eq!(underfunded.balance.available, BigUint::ZERO);
        assert_eq!(underfunded.balance.reserved, BigUint::from(1u8));
    }
}
