use std::error::Error;

use primitives::{TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use crate::models::{BitcoinNodeInfo, BitcoinUTXO};

pub fn map_transaction_preload(utxos: Vec<BitcoinUTXO>, input: TransactionPreloadInput) -> TransactionLoadMetadata {
    let utxos = map_utxos(utxos, input.sender_address);
    TransactionLoadMetadata::Bitcoin { utxos }
}

pub fn map_utxos(utxos: Vec<BitcoinUTXO>, address: String) -> Vec<UTXO> {
    utxos
        .into_iter()
        .filter(|utxo| !utxo.coinbase)
        .map(|utxo| UTXO {
            transaction_id: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
            address: address.clone(),
        })
        .collect()
}

pub fn map_transaction_preload_zcash(node_info: BitcoinNodeInfo, utxos: Vec<BitcoinUTXO>, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
    let utxos = map_utxos(utxos, input.sender_address);
    let branch_id = node_info.backend.consensus.ok_or("Branch ID not found")?.chaintip;
    Ok(TransactionLoadMetadata::Zcash { utxos, branch_id })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_utxos_skips_immature_coinbase() {
        let utxos: Vec<BitcoinUTXO> = serde_json::from_str(
            r#"[
                {"txid":"faa82c96d58602b7704eb609664ee800b6d9b2b27fa6c8c1dd4977d3264964cd","vout":0,"value":"307891765","height":974378,"confirmations":71,"coinbase":true},
                {"txid":"0000000000000000000000000000000000000000000000000000000000000001","vout":1,"value":"50000","height":974000,"confirmations":449}
            ]"#,
        )
        .unwrap();

        let result = map_utxos(utxos, "bitcoincash:qplzr8pyqknsgxeaevu20z3zrhcms3wp8g65lxrynf".to_string());

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].transaction_id, "0000000000000000000000000000000000000000000000000000000000000001");
        assert_eq!(result[0].vout, 1);
    }
}
