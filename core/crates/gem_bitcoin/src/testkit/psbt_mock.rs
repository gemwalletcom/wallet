use std::str::FromStr;

use bitcoin::{Address, Amount, OutPoint, Psbt, Sequence, Transaction, TxIn, TxOut, Txid, absolute::LockTime, hashes::Hash, transaction::Version};
use gem_encoding::encode_base64;
use primitives::UTXO;

use crate::signer::psbt::{BitcoinPsbtRequest, BitcoinPsbtSignInput};
use crate::testkit::{address_mock::mock_p2wpkh_address, signer_mock::mock_utxo_with};

pub fn mock_psbt(inputs: &[(u64, &str)], outputs: &[(u64, &str)]) -> String {
    let script = |address: &str| Address::from_str(address).unwrap().assume_checked().script_pubkey();
    let transaction = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: (0..inputs.len())
            .map(|index| TxIn {
                previous_output: OutPoint { txid: Txid::all_zeros(), vout: index as u32 },
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                ..TxIn::default()
            })
            .collect(),
        output: outputs
            .iter()
            .map(|(value, address)| TxOut {
                value: Amount::from_sat(*value),
                script_pubkey: script(address),
            })
            .collect(),
    };
    let mut psbt = Psbt::from_unsigned_tx(transaction).unwrap();
    for (input, (value, address)) in psbt.inputs.iter_mut().zip(inputs) {
        input.witness_utxo = Some(TxOut {
            value: Amount::from_sat(*value),
            script_pubkey: script(address),
        });
    }
    encode_base64(&psbt.serialize())
}

pub fn mock_psbt_request(inputs: &[(u64, &str)], outputs: &[(u64, &str)], sign: &[(usize, Option<Vec<u32>>)]) -> BitcoinPsbtRequest {
    BitcoinPsbtRequest {
        account: mock_p2wpkh_address(),
        psbt: mock_psbt(inputs, outputs),
        sign_inputs: sign
            .iter()
            .map(|(index, sighash_types)| BitcoinPsbtSignInput {
                address: mock_p2wpkh_address(),
                index: *index,
                sighash_types: sighash_types.clone(),
            })
            .collect(),
        broadcast: false,
    }
}

pub fn mock_psbt_coins(inputs: &[(u64, &str)]) -> Vec<UTXO> {
    inputs
        .iter()
        .enumerate()
        .map(|(index, (value, address))| mock_utxo_with(&Txid::all_zeros().to_string(), index as i32, &value.to_string(), address))
        .collect()
}
