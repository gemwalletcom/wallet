use crate::models::transaction::BitcoinUTXO;
use num_bigint::BigUint;
use primitives::{AssetBalance, Balance, BitcoinChain};

pub fn map_balance_coin(utxos: &[BitcoinUTXO], chain: BitcoinChain) -> AssetBalance {
    let (available, pending): (Vec<&BitcoinUTXO>, Vec<&BitcoinUTXO>) = utxos.iter().partition(|utxo| utxo.is_spendable());
    let sum = |utxos: Vec<&BitcoinUTXO>| -> BigUint { utxos.into_iter().map(|utxo| &utxo.value).sum() };
    let balance = Balance::with_pending_unconfirmed(sum(available), sum(pending));
    AssetBalance::new_balance(chain.get_chain().as_asset_id(), balance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_balance_coin() {
        let utxos: Vec<BitcoinUTXO> = serde_json::from_str(include_str!("../../testdata/utxos_immature_coinbase.json")).unwrap();
        let result = map_balance_coin(&utxos, BitcoinChain::BitcoinCash);

        assert_eq!(result.balance.available, BigUint::from(51000_u64));
        assert_eq!(result.balance.pending_unconfirmed, BigUint::from(307891765_u64));
    }

    #[test]
    fn test_map_balance_coin_empty() {
        let result = map_balance_coin(&[], BitcoinChain::Bitcoin);

        assert_eq!(result.balance.available, BigUint::ZERO);
        assert_eq!(result.balance.pending_unconfirmed, BigUint::ZERO);
    }
}
