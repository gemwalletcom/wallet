use std::str::FromStr;

use bitcoin::{Address, Amount, OutPoint, Psbt, Sequence, Transaction, TxIn, TxOut, Txid, absolute::LockTime, hashes::Hash, transaction::Version};

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
    psbt.to_string()
}
