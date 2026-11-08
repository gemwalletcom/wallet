use std::collections::HashMap;

use num_bigint::BigInt;
use primitives::{AssetId, Chain, GasPriceType, TransactionFee, TransactionLoadMetadata, UTXO};

use crate::models::utxo::UTXO as CardanoUTXO;

pub(crate) fn map_transaction_preload(utxos: Vec<CardanoUTXO>, block_number: u64) -> TransactionLoadMetadata {
    TransactionLoadMetadata::Cardano {
        utxos: utxos.into_iter().filter(|utxo| utxo.tokens.is_empty()).map(UTXO::from).collect(),
        block_number,
    }
}

pub(crate) fn map_transaction_fee(fee: u64) -> TransactionFee {
    TransactionFee::new_gas_price_type(GasPriceType::regular(BigInt::from(1u64)), BigInt::from(fee), BigInt::from(1u64), HashMap::new(), AssetId::from_chain(Chain::Cardano))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{rpc::Data, utxo::UTXOS};
    use num_bigint::BigUint;

    #[test]
    fn test_map_transaction_preload() {
        let response: Data<UTXOS<Vec<CardanoUTXO>>> = serde_json::from_str(include_str!("../../testdata/utxos_with_token.json")).unwrap();
        let metadata = map_transaction_preload(response.data.utxos, 1);

        assert_eq!(
            metadata.get_utxos().unwrap().into_iter().map(|utxo| utxo.transaction_id).collect::<Vec<_>>(),
            vec!["fc1b1eb70fc853e0da24129d5585810599d4b9ec45198817d0058c9055c4a6ad".to_string()],
            "a UTXO that also holds a token can't be spent by an ADA-only transaction"
        );
        assert_eq!(metadata.get_utxo_value(), Some(BigUint::from(77_825_303u64)));
    }
}
