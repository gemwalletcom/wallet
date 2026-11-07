use std::error::Error;

use primitives::{TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use crate::models::{BitcoinNodeInfo, BitcoinUTXO};

pub fn map_transaction_preload(utxos: Vec<BitcoinUTXO>, input: TransactionPreloadInput) -> TransactionLoadMetadata {
    let utxos = map_confirmed_utxos(utxos, input.sender_address);
    TransactionLoadMetadata::Bitcoin { utxos }
}

pub fn map_confirmed_utxos(utxos: Vec<BitcoinUTXO>, address: String) -> Vec<UTXO> {
    utxos
        .into_iter()
        .filter(BitcoinUTXO::is_confirmed)
        .map(|utxo| UTXO {
            transaction_id: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
            address: address.clone(),
        })
        .collect()
}

pub fn map_transaction_preload_zcash(node_info: BitcoinNodeInfo, utxos: Vec<BitcoinUTXO>, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
    let utxos = map_confirmed_utxos(utxos, input.sender_address);
    let branch_id = node_info.backend.consensus.ok_or("Branch ID not found")?.chaintip;
    Ok(TransactionLoadMetadata::Zcash { utxos, branch_id })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_confirmed_utxos_leaves_out_a_pending_deposit() {
        let response: Vec<BitcoinUTXO> = serde_json::from_str(include_str!("../../testdata/utxo_pending_deposit.json")).unwrap();

        let utxos = map_confirmed_utxos(response, "address".to_string());

        assert_eq!(utxos.iter().map(|utxo| (utxo.vout, utxo.value.to_string())).collect::<Vec<_>>(), vec![(1, "100000".to_string())]);
    }
}
