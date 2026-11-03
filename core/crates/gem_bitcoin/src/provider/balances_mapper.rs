use crate::models::account::BitcoinAccount;
use num_traits::CheckedSub;
use primitives::{AssetBalance, Balance, BitcoinChain};

pub fn map_balance_coin(account: &BitcoinAccount, chain: BitcoinChain) -> AssetBalance {
    let available = account.balance.checked_sub(&account.immature_balance).unwrap_or_default();
    let pending_unconfirmed = account.unconfirmed_balance.to_biguint().unwrap_or_default() + &account.immature_balance;
    let balance = Balance::with_pending_unconfirmed(available, pending_unconfirmed);
    AssetBalance::new_balance(chain.get_chain().as_asset_id(), balance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::{BigInt, BigUint};

    #[test]
    fn test_map_balance_coin_positive_unconfirmed() {
        let account = BitcoinAccount {
            balance: BigUint::from(20998955_u64),
            unconfirmed_balance: BigInt::from(5100000_i64),
            immature_balance: BigUint::ZERO,
        };
        let result = map_balance_coin(&account, BitcoinChain::Bitcoin);

        assert_eq!(result.balance.available, BigUint::from(20998955_u64));
        assert_eq!(result.balance.pending_unconfirmed, BigUint::from(5100000_u64));
    }

    #[test]
    fn test_map_balance_coin_negative_unconfirmed() {
        let account = BitcoinAccount {
            balance: BigUint::from(20000000_u64),
            unconfirmed_balance: BigInt::from(-10001045_i64),
            immature_balance: BigUint::ZERO,
        };
        let result = map_balance_coin(&account, BitcoinChain::Bitcoin);

        assert_eq!(result.balance.available, BigUint::from(20000000_u64));
        assert_eq!(result.balance.pending_unconfirmed, BigUint::ZERO);
    }

    #[test]
    fn test_map_balance_coin_immature_coinbase_pending() {
        let account = BitcoinAccount {
            balance: BigUint::from(307941765_u64),
            unconfirmed_balance: BigInt::from(1000_i64),
            immature_balance: BigUint::from(307891765_u64),
        };
        let result = map_balance_coin(&account, BitcoinChain::BitcoinCash);

        assert_eq!(result.balance.available, BigUint::from(50000_u64));
        assert_eq!(result.balance.pending_unconfirmed, BigUint::from(307892765_u64));
    }
}
