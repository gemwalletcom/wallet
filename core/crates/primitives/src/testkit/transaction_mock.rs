use crate::transaction_metadata_types::{TransactionAssetTransfer, TransactionAssetTransfersMetadata};
use crate::{AssetId, Chain, PerpetualDirection, Transaction, TransactionDirection, TransactionId, TransactionPerpetualMetadata, TransactionState, TransactionSwapMetadata, TransactionType, TransactionUtxoInput};
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

    pub fn mock_with_state(transaction_type: TransactionType, state: TransactionState, direction: TransactionDirection) -> Self {
        Self {
            direction,
            ..Transaction::new(
                "hash".to_string(),
                AssetId::from_chain(Chain::Ethereum),
                "from".to_string(),
                "to".to_string(),
                None,
                transaction_type,
                state,
                BigUint::from(1u32),
                AssetId::from_chain(Chain::Ethereum),
                BigUint::from(1u32),
                None,
                None,
                Utc::now(),
            )
        }
    }

    pub fn mock_swap_with_provider(state: TransactionState, provider: Option<&str>) -> Self {
        let metadata = TransactionSwapMetadata {
            from_value: BigUint::from(5u32),
            to_value: BigUint::from(1u32),
            provider: provider.map(str::to_string),
            ..TransactionSwapMetadata::mock()
        };
        Self {
            transaction_type: TransactionType::Swap,
            state,
            metadata: Some(serde_json::to_value(metadata).unwrap()),
            ..Self::mock()
        }
    }

    pub fn mock_with_asset_transfers(asset_transfers: Vec<TransactionAssetTransfer>) -> Self {
        Self {
            metadata: Some(serde_json::to_value(TransactionAssetTransfersMetadata { asset_transfers }).unwrap()),
            ..Self::mock()
        }
    }

    pub fn mock_swap() -> Self {
        Transaction::new(
            "hash".to_string(),
            AssetId::from_chain(Chain::Ethereum),
            "from".to_string(),
            "to".to_string(),
            None,
            TransactionType::Swap,
            TransactionState::Pending,
            BigUint::from(1u32),
            AssetId::from_chain(Chain::Ethereum),
            BigUint::from(1_000_000_000_000_000_000u64),
            None,
            serde_json::to_value(TransactionSwapMetadata::mock()).ok(),
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

impl TransactionSwapMetadata {
    pub fn mock() -> Self {
        Self {
            from_asset: AssetId::from_chain(Chain::Ethereum),
            from_value: BigUint::from(1_000_000_000_000_000_000u64),
            to_asset: AssetId::from_chain(Chain::Bitcoin),
            to_value: BigUint::from(10_000_000_000_000_000_000u128),
            provider: Some("thorchain".to_string()),
        }
    }
}

impl TransactionPerpetualMetadata {
    pub fn mock() -> Self {
        Self {
            pnl: 0.0,
            price: 1.0,
            direction: PerpetualDirection::Long,
            is_liquidation: None,
            provider: None,
        }
    }
}

impl TransactionId {
    pub fn mock(hash: &str) -> Self {
        Self::new(Chain::Ethereum, hash.to_string())
    }
}
