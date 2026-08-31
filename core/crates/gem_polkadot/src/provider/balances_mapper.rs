use crate::models::account::PolkadotAccountBalance;
use primitives::{AssetBalance, Chain};

pub fn map_coin_balance(balance: PolkadotAccountBalance) -> AssetBalance {
    AssetBalance::new_balance(Chain::Polkadot.as_asset_id(), balance.balance())
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn test_map_coin_balance() {
        let result = map_coin_balance(PolkadotAccountBalance::mock());

        assert_eq!(result.asset_id, Chain::Polkadot.as_asset_id());
        assert_eq!(result.balance.available, BigUint::from(0u32));
        assert_eq!(result.balance.frozen, BigUint::from(31415926535_u64));
    }
}
