use std::cmp::Ordering;

use gem_bitcoin::{
    models::Transaction,
    provider::transactions_mapper::{map_utxo_inputs, map_utxo_outputs, map_utxo_transaction},
};
use num_bigint::BigUint;
use primitives::{Chain, transaction_utxo::TransactionUtxoInput};

use super::value_balance::ValueBalanceReader;

pub(crate) fn map_transactions(transactions: Vec<Transaction>) -> Vec<primitives::Transaction> {
    transactions.iter().filter_map(map_transaction).collect()
}

pub(crate) fn map_transaction(transaction: &Transaction) -> Option<primitives::Transaction> {
    let mut inputs = map_utxo_inputs(Chain::Zcash, transaction)?;
    let mut outputs = map_utxo_outputs(Chain::Zcash, transaction)?;
    let input_value = transaction.value_in.parse::<BigUint>().ok()?;
    let output_value = transaction.value.clone();
    let value_balance = transaction.hex.as_deref().and_then(ValueBalanceReader::read).unwrap_or_default();
    let shielded_value = BigUint::from(value_balance.unsigned_abs());
    let (input_value, output_value) = match value_balance.cmp(&0) {
        Ordering::Greater => {
            inputs.push(shielded_utxo(&shielded_value));
            (input_value + &shielded_value, output_value)
        }
        Ordering::Less => {
            outputs.push(shielded_utxo(&shielded_value));
            (input_value, output_value + &shielded_value)
        }
        Ordering::Equal => (input_value, output_value),
    };
    let fee = if input_value > output_value { &input_value - &output_value } else { BigUint::ZERO };
    map_utxo_transaction(Chain::Zcash, transaction, inputs, outputs, output_value, fee)
}

fn shielded_utxo(value: &BigUint) -> TransactionUtxoInput {
    TransactionUtxoInput {
        address: String::new(),
        value: value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::models::transaction::{Input, Output};
    use primitives::TransactionDirection;

    use super::*;

    const SENDER: &str = "t1SdZhkr3TmiHttYNL6y4qsK5N7nMDoPHm8";
    const RECIPIENT: &str = "t1SXEuGoJQco68tbdW6guj5w2CjGunQD347";
    const CHANGE_SENDER: &str = "t1dTLYuecTCt8irWJsyLkDaJm2EAvVgdxgM";

    fn finalize(transaction: &Transaction, address: &str) -> primitives::Transaction {
        map_transaction(transaction).unwrap().finalize(vec![address.to_string()])
    }

    #[test]
    fn test_map_shielded_send() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_shielded_send.json")).unwrap();
        let mapped = finalize(&transaction, SENDER);

        assert_eq!(transaction.fees, BigUint::from(35_090_290u64));
        assert_eq!((mapped.direction, mapped.from.as_str(), mapped.to.as_str()), (TransactionDirection::Outgoing, SENDER, ""));
        assert_eq!((mapped.value, mapped.fee), (BigUint::from(35_065_290u64), BigUint::from(25_000u64)));
    }

    #[test]
    fn test_map_shielded_send_with_change() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_shielded_send_change.json")).unwrap();
        let mapped = finalize(&transaction, CHANGE_SENDER);

        assert_eq!(transaction.fees, BigUint::from(215_000u64));
        assert_eq!((mapped.direction, mapped.from.as_str(), mapped.to.as_str()), (TransactionDirection::Outgoing, CHANGE_SENDER, ""));
        assert_eq!((mapped.value, mapped.fee), (BigUint::from(200_000u64), BigUint::from(15_000u64)));
    }

    #[test]
    fn test_map_shielded_receive() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_shielded_receive.json")).unwrap();
        let mapped = finalize(&transaction, RECIPIENT);

        assert_eq!((mapped.direction, mapped.from.as_str(), mapped.to.as_str()), (TransactionDirection::Incoming, "", RECIPIENT));
        assert_eq!((mapped.value, mapped.fee), (BigUint::from(2_000_000u64), BigUint::from(15_000u64)));
    }

    #[test]
    fn test_map_transparent_transaction() {
        let transaction = Transaction {
            vin: vec![Input::mock()],
            vout: vec![Output::mock()],
            ..Transaction::mock()
        };
        let mapped = map_transaction(&transaction).unwrap();

        assert_eq!((mapped.value, mapped.fee), (BigUint::from(100_000u64), BigUint::from(5_000u64)));
        assert_eq!(mapped.utxo_outputs.unwrap().len(), 1);
    }
}
