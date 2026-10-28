use primitives::{BitcoinChain, OutputMinimum, SignerError, TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use crate::models::BitcoinUTXO;
use crate::signer::{SpendDestination, output_minimum};

pub fn map_transaction_preload(chain: BitcoinChain, utxos: Vec<BitcoinUTXO>, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, SignerError> {
    let destination = SpendDestination::script(chain, input.input_type.swap_to_address().unwrap_or(&input.destination_address))?;
    let output_minimum = preload_output_minimum(chain, &input, &destination)?;
    let utxos = map_utxos(utxos, input.sender_address);
    Ok(TransactionLoadMetadata::Bitcoin { utxos, output_minimum })
}

pub fn preload_output_minimum(chain: BitcoinChain, input: &TransactionPreloadInput, destination: &SpendDestination) -> Result<OutputMinimum, SignerError> {
    let minimum = output_minimum(chain, &input.sender_address, destination)?;
    Ok(match input.input_type.swap_to_address() {
        Some(_) => OutputMinimum { change: 0, ..minimum },
        None => minimum,
    })
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

#[cfg(test)]
mod tests {
    use primitives::{Asset, Chain, TransactionInputType};

    use super::*;
    use crate::testkit::signer_mock::mock_transfer_swap_input;

    #[test]
    fn test_swap_dust_change_needs_no_minimum() {
        let swap = mock_transfer_swap_input(BitcoinChain::Bitcoin, "memo");
        let minimum = |input_type: TransactionInputType| {
            let input = TransactionPreloadInput {
                input_type,
                sender_address: swap.sender_address.clone(),
                destination_address: swap.destination_address.clone(),
                references: vec![],
            };
            map_transaction_preload(BitcoinChain::Bitcoin, vec![], input).unwrap().get_output_minimum().unwrap()
        };
        let transfer = output_minimum(BitcoinChain::Bitcoin, &swap.sender_address, &SpendDestination::script(BitcoinChain::Bitcoin, &swap.destination_address).unwrap()).unwrap();

        assert_eq!(minimum(TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Bitcoin) }), transfer);
        assert_eq!(minimum(swap.input.input_type.clone()), OutputMinimum { change: 0, ..transfer }, "a swap's dust change goes to the fee");
    }
}
