use primitives::{TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use crate::models::BitcoinUTXO;

pub fn map_transaction_preload(utxos: Vec<BitcoinUTXO>, input: TransactionPreloadInput) -> TransactionLoadMetadata {
    let utxos = map_utxos(utxos, input.sender_address);
    TransactionLoadMetadata::Bitcoin { utxos }
}

pub fn map_utxos(utxos: Vec<BitcoinUTXO>, address: String) -> Vec<UTXO> {
    utxos
        .into_iter()
        .map(|utxo| UTXO {
            transaction_id: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
            address: address.clone(),
        })
        .collect()
}
