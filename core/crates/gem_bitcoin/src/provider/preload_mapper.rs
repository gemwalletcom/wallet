use primitives::{BitcoinChain, SignerError, TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use crate::models::BitcoinUTXO;
use crate::signer::{SpendDestination, output_minimum};

pub fn map_transaction_preload(chain: BitcoinChain, utxos: Vec<BitcoinUTXO>, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, SignerError> {
    let destination = SpendDestination::script(chain, input.input_type.swap_to_address().unwrap_or(&input.destination_address))?;
    let output_minimum = output_minimum(chain, &input.sender_address, &destination)?;
    let utxos = map_utxos(utxos, input.sender_address);
    Ok(TransactionLoadMetadata::Bitcoin { utxos, output_minimum })
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
