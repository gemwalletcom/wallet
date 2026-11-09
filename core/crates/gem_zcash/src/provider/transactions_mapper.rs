use std::cmp::Ordering;

use gem_bitcoin::{
    models::Transaction as BitcoinTransaction,
    provider::transactions_mapper::{map_transaction as map_transparent_transaction, map_utxo_inputs, map_utxo_outputs, map_utxo_transaction},
};
use num_bigint::BigUint;
use primitives::{Chain, Transaction, transaction_utxo::TransactionUtxoInput};

use super::value_balance::ValueBalanceReader;

pub(crate) fn map_transactions(transactions: Vec<BitcoinTransaction>) -> Vec<Transaction> {
    transactions.iter().filter_map(map_transaction).collect()
}

pub(crate) fn map_transaction(transaction: &BitcoinTransaction) -> Option<Transaction> {
    let Some(value_balance) = transaction.hex.as_deref().and_then(ValueBalanceReader::read) else {
        return map_transparent_transaction(Chain::Zcash, transaction);
    };
    let mut inputs = map_utxo_inputs(Chain::Zcash, transaction)?;
    let mut outputs = map_utxo_outputs(Chain::Zcash, transaction)?;
    let input_value = transaction.value_in.parse::<BigUint>().ok()?;
    let output_value = transaction.value.clone();
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
    use primitives::TransactionDirection;

    use super::*;

    #[test]
    fn test_map_transaction() {
        let transaction = |json: &str| serde_json::from_str::<BitcoinTransaction>(json).unwrap();
        let send = transaction(include_str!("../../testdata/transaction_shielded_send.json"));
        let mapped = map_transaction(&send).unwrap().finalize(vec!["t1SdZhkr3TmiHttYNL6y4qsK5N7nMDoPHm8".to_string()]);
        assert_eq!(send.fees, BigUint::from(35_090_290u64));
        assert_eq!((mapped.direction, mapped.from.as_str(), mapped.to.as_str()), (TransactionDirection::Outgoing, "t1SdZhkr3TmiHttYNL6y4qsK5N7nMDoPHm8", ""));
        assert_eq!((mapped.value, mapped.fee), (BigUint::from(35_065_290u64), BigUint::from(25_000u64)));

        let send_with_change = transaction(include_str!("../../testdata/transaction_shielded_send_change.json"));
        let mapped = map_transaction(&send_with_change).unwrap().finalize(vec!["t1dTLYuecTCt8irWJsyLkDaJm2EAvVgdxgM".to_string()]);
        assert_eq!(send_with_change.fees, BigUint::from(215_000u64));
        assert_eq!((mapped.direction, mapped.from.as_str(), mapped.to.as_str()), (TransactionDirection::Outgoing, "t1dTLYuecTCt8irWJsyLkDaJm2EAvVgdxgM", ""));
        assert_eq!((mapped.value, mapped.fee), (BigUint::from(200_000u64), BigUint::from(15_000u64)));

        let receive = transaction(include_str!("../../testdata/transaction_shielded_receive.json"));
        let mapped = map_transaction(&receive).unwrap().finalize(vec!["t1SXEuGoJQco68tbdW6guj5w2CjGunQD347".to_string()]);
        assert_eq!((mapped.direction, mapped.from.as_str(), mapped.to.as_str()), (TransactionDirection::Incoming, "", "t1SXEuGoJQco68tbdW6guj5w2CjGunQD347"));
        assert_eq!((mapped.value, mapped.fee), (BigUint::from(2_000_000u64), BigUint::from(15_000u64)));

        let joinsplit = transaction(include_str!("../../testdata/transaction_joinsplit.json"));
        assert_eq!(map_transaction(&joinsplit).unwrap().fee, BigUint::from(10_000u64));

        for hex in [None, Some("not hex".to_string())] {
            let transparent = BitcoinTransaction {
                vin: joinsplit.vin.clone(),
                hex,
                ..receive.clone()
            };
            assert!(map_transparent_transaction(Chain::Zcash, &transparent).is_some());
            assert_eq!(
                map_transaction(&transparent),
                map_transparent_transaction(Chain::Zcash, &transparent),
                "without a readable raw transaction only the transparent side is shown"
            );
        }
    }
}
