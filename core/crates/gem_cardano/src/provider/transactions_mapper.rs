use crate::models::rpc::{AddressHistory, Transaction};
use num_bigint::BigUint;
use primitives::{DateTime, TransactionState, TransactionType, chain::Chain, transaction_utxo::TransactionUtxoInput};

pub fn map_transaction(chain: Chain, created_at: DateTime, transaction: &Transaction) -> Option<primitives::Transaction> {
    let inputs: Vec<TransactionUtxoInput> = transaction
        .inputs
        .iter()
        .map(|x| TransactionUtxoInput {
            address: x.address.clone(),
            value: x.value.clone(),
        })
        .collect();

    let outputs: Vec<TransactionUtxoInput> = transaction
        .outputs
        .iter()
        .map(|x| TransactionUtxoInput {
            address: x.address.clone(),
            value: x.value.clone(),
        })
        .collect();

    if inputs.is_empty() || outputs.is_empty() {
        return None;
    }
    let transaction = primitives::Transaction::new_with_utxo(
        transaction.hash.clone(),
        chain.as_asset_id(),
        TransactionType::Transfer,
        TransactionState::Confirmed,
        transaction.fee.clone(),
        chain.as_asset_id(),
        BigUint::from(0u32),
        None,
        inputs.into(),
        outputs.into(),
        None,
        created_at,
    );

    Some(transaction)
}

pub fn map_transactions_by_address(chain: Chain, history: AddressHistory, limit: usize) -> Vec<primitives::Transaction> {
    let mut transactions = history.received.into_iter().chain(history.sent).collect::<Vec<_>>();
    transactions.sort_by(|a, b| b.included_at.cmp(&a.included_at).then_with(|| a.transaction.hash.cmp(&b.transaction.hash)));
    transactions.dedup_by(|a, b| a.transaction.hash == b.transaction.hash);
    transactions.truncate(limit);
    transactions.iter().filter_map(|transaction| map_transaction(chain, transaction.included_at, &transaction.transaction)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rpc::{Data, Input, Output};

    #[test]
    fn test_map_transaction() {
        let transaction = Transaction {
            hash: "tx_hash".to_string(),
            inputs: vec![Input {
                address: "addr1".to_string(),
                value: 1000u32.into(),
            }],
            outputs: vec![Output {
                address: "addr2".to_string(),
                value: 900u32.into(),
            }],
            fee: 100u32.into(),
        };

        let result = map_transaction(Chain::Cardano, DateTime::parse_rfc3339("2023-01-01T00:00:00Z").unwrap(), &transaction);
        assert!(result.is_some());

        let mapped_tx = result.unwrap();
        assert_eq!(mapped_tx.id.to_string(), "cardano_tx_hash");
        assert_eq!(mapped_tx.fee, BigUint::from(100u64));
    }

    #[test]
    fn test_map_transactions_by_address() {
        let response: Data<AddressHistory> = serde_json::from_str(include_str!("../../testdata/address_transactions.json")).unwrap();

        let transactions = map_transactions_by_address(Chain::Cardano, response.data, 3);

        assert_eq!(transactions.iter().map(|transaction| transaction.hash()).collect::<Vec<_>>(), vec!["receive", "send_with_change", "send_max"]);
    }
}
