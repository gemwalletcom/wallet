use crate::{AssetId, Chain, Transaction, TransactionState, TransactionType, TransactionUtxoInput};
use chrono::Utc;
use num_bigint::BigUint;

impl Transaction {
    pub fn mock() -> Self {
        Transaction::new(
            "0x1234567890abcdef".to_string(),
            AssetId::from_chain(Chain::Ethereum),
            "0xfrom".to_string(),
            "0xto".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(21_000u32),
            AssetId::from_chain(Chain::Ethereum),
            BigUint::from(1_000_000u32),
            None,
            None,
            Utc::now(),
        )
    }

    pub fn mock_with_params(asset_id: AssetId, transaction_type: TransactionType, value: BigUint) -> Self {
        Transaction::new(
            "0x1234567890abcdef".to_string(),
            asset_id.clone(),
            "0xfrom".to_string(),
            "0xto".to_string(),
            None,
            transaction_type,
            TransactionState::Confirmed,
            BigUint::from(21_000u32),
            asset_id,
            value,
            None,
            None,
            Utc::now(),
        )
    }

    pub fn mock_utxo(utxo_inputs: Vec<TransactionUtxoInput>, utxo_outputs: Vec<TransactionUtxoInput>) -> Self {
        Transaction::new_with_utxo(
            "btc_tx_hash".to_string(),
            AssetId::from_chain(Chain::Bitcoin),
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(1_000u32),
            AssetId::from_chain(Chain::Bitcoin),
            BigUint::from(0u32),
            None,
            Some(utxo_inputs),
            Some(utxo_outputs),
            None,
            Utc::now(),
        )
    }
}
