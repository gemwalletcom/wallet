use crate::{
    AddressName, AssetAddress, NFTAssetId, TransactionId, TransactionNFTTransferMetadata, TransactionSwapMetadata, asset_id::AssetId, transaction_direction::TransactionDirection,
    transaction_metadata_types::TransactionAssetTransfersMetadata, transaction_state::TransactionState, transaction_type::TransactionType, transaction_utxo::TransactionUtxoInput,
};

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, vec};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionsResponse {
    pub transactions: Vec<Transaction>,
    pub address_names: Vec<AddressName>,
}

impl TransactionsResponse {
    pub fn new(transactions: Vec<Transaction>, address_names: Vec<AddressName>) -> Self {
        Self { transactions, address_names }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable, Equatable, Hashable")]
pub struct Transaction {
    pub id: TransactionId,
    #[serde(rename = "assetId")]
    pub asset_id: AssetId,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub state: TransactionState,
    #[serde(rename = "blockNumber", skip_serializing_if = "Option::is_none")]
    pub block_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<String>,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub fee: BigUint,
    #[serde(rename = "feeAssetId")]
    pub fee_asset_id: AssetId,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub value: BigUint,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub direction: TransactionDirection,
    #[serde(rename = "utxoInputs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utxo_inputs: Option<Vec<TransactionUtxoInput>>,
    #[serde(rename = "utxoOutputs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utxo_outputs: Option<Vec<TransactionUtxoInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[typeshare(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    pub fn hash(&self) -> &str {
        &self.id.hash
    }

    pub fn new(
        hash: String,
        asset_id: AssetId,
        from_address: String,
        to_address: String,
        contract: Option<String>,
        transaction_type: TransactionType,
        state: TransactionState,
        fee: BigUint,
        fee_asset_id: AssetId,
        value: BigUint,
        memo: Option<String>,
        metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: TransactionId::new(asset_id.chain, hash),
            asset_id,
            from: from_address,
            to: to_address,
            contract,
            transaction_type,
            state,
            block_number: Some("".to_string()),
            sequence: Some("".to_string()),
            fee,
            fee_asset_id,
            value,
            memo,
            direction: TransactionDirection::SelfTransfer,
            utxo_inputs: vec![].into(),
            utxo_outputs: vec![].into(),
            metadata,
            data: None,
            created_at,
        }
    }

    pub fn new_with_utxo(
        hash: String,
        asset_id: AssetId,
        transaction_type: TransactionType,
        state: TransactionState,
        fee: BigUint,
        fee_asset_id: AssetId,
        value: BigUint,
        memo: Option<String>,
        utxo_inputs: Option<Vec<TransactionUtxoInput>>,
        utxo_outputs: Option<Vec<TransactionUtxoInput>>,
        metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: TransactionId::new(asset_id.chain, hash),
            asset_id,
            from: "".to_string(),
            to: "".to_string(),
            contract: None,
            transaction_type,
            state,
            block_number: Some("".to_string()),
            sequence: Some("".to_string()),
            fee,
            fee_asset_id,
            value,
            memo,
            direction: TransactionDirection::SelfTransfer,
            utxo_inputs: utxo_inputs.unwrap_or_default().into(),
            utxo_outputs: utxo_outputs.unwrap_or_default().into(),
            metadata,
            data: None,
            created_at,
        }
    }

    pub fn is_sent(&self, address: String) -> bool {
        self.input_addresses().contains(&address) || self.from == address
    }

    pub fn input_addresses(&self) -> Vec<String> {
        self.utxo_inputs.as_ref().map_or_else(Vec::new, |v| v.iter().map(|x| x.address.clone()).collect())
    }

    pub fn output_addresses(&self) -> Vec<String> {
        self.utxo_outputs.as_ref().map_or_else(Vec::new, |v| v.iter().map(|x| x.address.clone()).collect())
    }

    pub fn addresses(&self) -> Vec<String> {
        let mut addresses = vec![self.from.clone(), self.to.clone()];
        addresses.extend(self.input_addresses());
        addresses.extend(self.output_addresses());
        if let Some(metadata) = self.asset_transfers_metadata() {
            addresses.extend(metadata.asset_transfers.into_iter().flat_map(|transfer| [transfer.from, transfer.to]));
        }

        let mut unique = HashSet::new();
        addresses.retain(|address| !address.is_empty() && unique.insert(address.clone()));
        addresses
    }

    pub fn finalize(&self, addresses: Vec<String>) -> Self {
        if !self.asset_id.chain.is_utxo() {
            let Some(metadata) = self.asset_transfers_metadata() else {
                return self.clone();
            };
            return self.project_asset_transfer(metadata, &addresses).unwrap_or_else(|| self.clone());
        }

        let inputs_addresses = self.input_addresses();
        let outputs_addresses = self.output_addresses();

        if addresses.is_empty() || inputs_addresses.is_empty() || outputs_addresses.is_empty() {
            return self.clone();
        }

        let user_set: HashSet<String> = HashSet::from_iter(addresses);
        let input_set: HashSet<String> = HashSet::from_iter(inputs_addresses);
        let output_set: HashSet<String> = HashSet::from_iter(outputs_addresses.clone());

        if user_set.is_disjoint(&input_set) && user_set.is_disjoint(&output_set) {
            return self.clone();
        }

        let direction = if user_set.intersection(&input_set).next().is_some() {
            if user_set.is_superset(&output_set) {
                TransactionDirection::SelfTransfer
            } else {
                TransactionDirection::Outgoing
            }
        } else {
            TransactionDirection::Incoming
        };

        let utxo_inputs = self.utxo_inputs.as_ref().unwrap();
        let utxo_outputs = self.utxo_outputs.as_ref().unwrap();

        let from = utxo_inputs.first().unwrap().address.clone();
        let (to, value) = match direction {
            TransactionDirection::Incoming => {
                let to = outputs_addresses.iter().find(|x| user_set.contains(*x)).unwrap().clone();
                let value = Self::utxo_calculate_value(utxo_outputs, &user_set);
                (to, value)
            }
            TransactionDirection::Outgoing => {
                let to = outputs_addresses.iter().find(|x| !user_set.contains(*x)).unwrap().clone();
                let value = utxo_outputs.iter().find(|x| x.address == to).unwrap().value.clone();
                (to, value)
            }
            TransactionDirection::SelfTransfer => {
                let to = utxo_outputs.first().unwrap().address.clone();
                let value = Self::utxo_calculate_value(utxo_outputs, &user_set);
                (to, value)
            }
        };
        Self {
            from,
            to,
            value,
            direction,
            ..self.clone()
        }
    }

    fn project_asset_transfer(&self, metadata: TransactionAssetTransfersMetadata, addresses: &[String]) -> Option<Self> {
        let contains = |address: &str| addresses.iter().any(|candidate| candidate.eq_ignore_ascii_case(address));
        let mut transfers = metadata.asset_transfers.into_iter().filter(|transfer| contains(&transfer.from) || contains(&transfer.to));
        let transfer = transfers.next()?;
        if transfers.next().is_some() {
            return None;
        }

        let direction = if contains(&transfer.from) {
            if contains(&transfer.to) {
                TransactionDirection::SelfTransfer
            } else {
                TransactionDirection::Outgoing
            }
        } else {
            TransactionDirection::Incoming
        };

        Some(Self {
            asset_id: transfer.asset_id,
            from: transfer.from,
            to: transfer.to,
            transaction_type: TransactionType::Transfer,
            value: transfer.value.clone(),
            direction,
            metadata: None,
            ..self.clone()
        })
    }

    fn utxo_calculate_value(values: &[TransactionUtxoInput], addresses: &HashSet<String>) -> BigUint {
        values.iter().filter(|x| addresses.contains(&x.address)).map(|x| &x.value).sum()
    }

    pub fn swap_metadata(&self) -> Option<TransactionSwapMetadata> {
        self.metadata.as_ref().and_then(|value| TransactionSwapMetadata::deserialize(value).ok())
    }

    fn asset_transfers_metadata(&self) -> Option<TransactionAssetTransfersMetadata> {
        self.metadata.as_ref().and_then(|value| TransactionAssetTransfersMetadata::deserialize(value).ok())
    }

    pub fn nft_asset_id(&self) -> Option<NFTAssetId> {
        if self.transaction_type != TransactionType::TransferNFT {
            return None;
        }
        self.metadata
            .as_ref()
            .and_then(|value| TransactionNFTTransferMetadata::deserialize(value).ok())
            .map(|metadata| metadata.asset_id)
    }

    pub fn asset_ids(&self) -> Vec<AssetId> {
        let mut asset_ids = match self.transaction_type {
            TransactionType::Transfer
            | TransactionType::TokenApproval
            | TransactionType::StakeDelegate
            | TransactionType::StakeUndelegate
            | TransactionType::StakeRewards
            | TransactionType::StakeRedelegate
            | TransactionType::StakeWithdraw
            | TransactionType::StakeFreeze
            | TransactionType::StakeUnfreeze
            | TransactionType::AssetActivation
            | TransactionType::TransferNFT
            | TransactionType::SmartContractCall
            | TransactionType::PerpetualOpenPosition
            | TransactionType::PerpetualClosePosition
            | TransactionType::PerpetualModifyPosition
            | TransactionType::EarnDeposit
            | TransactionType::EarnWithdraw => vec![self.asset_id.clone()],
            TransactionType::Swap => self.swap_metadata().map(|metadata| vec![metadata.from_asset, metadata.to_asset]).unwrap_or_default(),
        };
        if let Some(metadata) = self.asset_transfers_metadata() {
            asset_ids.extend(metadata.asset_transfers.into_iter().map(|transfer| transfer.asset_id));
        }
        asset_ids.into_iter().collect::<HashSet<_>>().into_iter().collect()
    }

    pub fn associated_asset_ids(&self) -> Vec<AssetId> {
        self.asset_ids()
            .into_iter()
            .chain([self.asset_id.clone(), self.fee_asset_id.clone()])
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn assets_addresses(&self) -> Vec<AssetAddress> {
        if let Some(metadata) = self.asset_transfers_metadata() {
            return metadata
                .asset_transfers
                .into_iter()
                .flat_map(|transfer| {
                    [
                        AssetAddress::new(transfer.asset_id.clone(), transfer.from, None),
                        AssetAddress::new(transfer.asset_id, transfer.to, None),
                    ]
                })
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
        }

        match self.transaction_type {
            TransactionType::Transfer | TransactionType::TransferNFT => self
                .addresses()
                .into_iter()
                .map(|x| AssetAddress::new(self.asset_id.clone(), x, None))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            TransactionType::TokenApproval => vec![AssetAddress::new(self.asset_id.clone(), self.from.clone(), None)],
            TransactionType::StakeDelegate
            | TransactionType::StakeUndelegate
            | TransactionType::StakeRewards
            | TransactionType::StakeRedelegate
            | TransactionType::StakeWithdraw
            | TransactionType::StakeFreeze
            | TransactionType::StakeUnfreeze
            | TransactionType::AssetActivation
            | TransactionType::SmartContractCall
            | TransactionType::PerpetualOpenPosition
            | TransactionType::PerpetualClosePosition
            | TransactionType::PerpetualModifyPosition
            | TransactionType::EarnDeposit
            | TransactionType::EarnWithdraw => vec![AssetAddress::new(self.asset_id.clone(), self.to.clone(), None)],
            TransactionType::Swap => self
                .swap_metadata()
                .map(|metadata| {
                    vec![
                        AssetAddress::new(metadata.from_asset, self.from.clone(), None),
                        AssetAddress::new(metadata.to_asset, self.to.clone(), None),
                    ]
                })
                .unwrap_or_default(),
        }
    }

    pub fn assets_addresses_with_fee(&self) -> Vec<AssetAddress> {
        [self.assets_addresses(), vec![AssetAddress::new(self.fee_asset_id.clone(), self.from.clone(), None)]]
            .concat()
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn without_utxo(self) -> Self {
        Self {
            utxo_inputs: None,
            utxo_outputs: None,
            ..self
        }
    }

    pub fn with_data(self, data: Option<String>) -> Self {
        Self { data, ..self }
    }

    pub fn with_swap_state(self, state: TransactionState, metadata: Option<serde_json::Value>) -> Self {
        Self {
            state,
            transaction_type: TransactionType::Swap,
            metadata: metadata.or(self.metadata),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Asset, Chain, TransactionUtxoInput, transaction_metadata_types::TransactionAssetTransfer};

    #[test]
    fn test_asset_ids_transfer() {
        assert_eq!(Transaction::mock().asset_ids().len(), 1);

        let transaction = Transaction {
            asset_id: Asset::mock_ethereum_usdc().id,
            ..Transaction::mock()
        };
        assert_eq!(transaction.asset_ids().len(), 1);
    }

    #[test]
    fn test_asset_ids_swap() {
        let transaction = Transaction {
            transaction_type: TransactionType::Swap,
            metadata: Some(
                serde_json::to_value(TransactionSwapMetadata {
                    from_asset: Asset::mock_eth().id,
                    from_value: BigUint::from(1u64),
                    to_asset: Asset::mock_eth().id,
                    to_value: BigUint::from(1u64),
                    provider: None,
                })
                .unwrap(),
            ),
            ..Transaction::mock()
        };
        assert_eq!(transaction.asset_ids().len(), 1);

        let transaction = Transaction {
            transaction_type: TransactionType::Swap,
            metadata: Some(
                serde_json::to_value(TransactionSwapMetadata {
                    from_asset: Asset::mock_ethereum_usdc().id,
                    from_value: BigUint::from(1u64),
                    to_asset: Asset::mock_erc20().id,
                    to_value: BigUint::from(1u64),
                    provider: None,
                })
                .unwrap(),
            ),
            ..Transaction::mock()
        };
        assert_eq!(transaction.asset_ids().len(), 2);
    }

    #[test]
    fn test_assets_addresses_transfer() {
        // Without fee
        assert_eq!(Transaction::mock().assets_addresses().len(), 2);

        let transaction = Transaction {
            asset_id: Asset::mock_ethereum_usdc().id,
            ..Transaction::mock()
        };
        assert_eq!(transaction.assets_addresses().len(), 2);
        assert!(
            transaction
                .assets_addresses()
                .iter()
                .any(|a| a.asset_id == Asset::mock_ethereum_usdc().id && a.address == "0xfrom")
        );
        assert!(
            transaction
                .assets_addresses()
                .iter()
                .any(|a| a.asset_id == Asset::mock_ethereum_usdc().id && a.address == "0xto")
        );

        // With fee
        assert_eq!(Transaction::mock().assets_addresses_with_fee().len(), 2);
        assert_eq!(transaction.assets_addresses_with_fee().len(), 3);
        assert!(
            transaction
                .assets_addresses_with_fee()
                .iter()
                .any(|a| a.asset_id == Asset::mock_eth().id && a.address == "0xfrom")
        );
        assert!(
            transaction
                .assets_addresses_with_fee()
                .iter()
                .any(|a| a.asset_id == Asset::mock_ethereum_usdc().id && a.address == "0xfrom")
        );
        assert!(
            transaction
                .assets_addresses_with_fee()
                .iter()
                .any(|a| a.asset_id == Asset::mock_ethereum_usdc().id && a.address == "0xto")
        );
    }

    #[test]
    fn test_assets_addresses_swap() {
        let transaction = Transaction {
            transaction_type: TransactionType::Swap,
            from: "0xsame".to_string(),
            to: "0xsame".to_string(),
            metadata: Some(
                serde_json::to_value(TransactionSwapMetadata {
                    from_asset: Asset::mock_ethereum_usdc().id,
                    from_value: BigUint::from(1u64),
                    to_asset: Asset::mock_erc20().id,
                    to_value: BigUint::from(1u64),
                    provider: None,
                })
                .unwrap(),
            ),
            ..Transaction::mock()
        };
        // Without fee: 2 swap assets
        assert_eq!(transaction.assets_addresses().len(), 2);
        // With fee: 2 swap assets + 1 fee
        assert_eq!(transaction.assets_addresses_with_fee().len(), 3);
    }

    #[test]
    fn test_assets_addresses_token_approval_uses_owner() {
        let transaction = Transaction {
            asset_id: Asset::mock_ethereum_usdc().id,
            from: "0xowner".to_string(),
            to: "0xspender".to_string(),
            transaction_type: TransactionType::TokenApproval,
            ..Transaction::mock()
        };

        let addresses = transaction.assets_addresses();

        assert_eq!(addresses, vec![AssetAddress::new(Asset::mock_ethereum_usdc().id, "0xowner".to_string(), None)]);
    }

    fn utxo_input(address: &str, value: u64) -> TransactionUtxoInput {
        TransactionUtxoInput::new(address.to_string(), value.into())
    }

    #[test]
    fn test_finalize_incoming_utxo() {
        let transaction =
            Transaction::mock_utxo(vec![utxo_input("sender", 50_000)], vec![utxo_input("user", 40_000), utxo_input("change", 9_000)]).finalize(vec!["user".to_string()]);

        assert_eq!(
            (transaction.from.as_str(), transaction.to.as_str(), transaction.value.to_string().as_str()),
            ("sender", "user", "40000")
        );
        assert_eq!(transaction.direction, TransactionDirection::Incoming);
    }

    #[test]
    fn test_finalize_outgoing_utxo() {
        let transaction =
            Transaction::mock_utxo(vec![utxo_input("user", 50_000)], vec![utxo_input("recipient", 40_000), utxo_input("user", 9_000)]).finalize(vec!["user".to_string()]);

        assert_eq!(
            (transaction.from.as_str(), transaction.to.as_str(), transaction.value.to_string().as_str()),
            ("user", "recipient", "40000")
        );
        assert_eq!(transaction.direction, TransactionDirection::Outgoing);
    }

    #[test]
    fn test_finalize_self_transfer_utxo() {
        let transaction = Transaction::mock_utxo(vec![utxo_input("user", 50_000)], vec![utxo_input("user", 40_000), utxo_input("user", 9_000)]).finalize(vec!["user".to_string()]);

        assert_eq!(
            (transaction.from.as_str(), transaction.to.as_str(), transaction.value.to_string().as_str()),
            ("user", "user", "49000")
        );
        assert_eq!(transaction.direction, TransactionDirection::SelfTransfer);
    }

    #[test]
    fn test_finalize_non_utxo_unchanged() {
        let original = Transaction::mock();
        let transaction = original.clone().finalize(vec!["0xfrom".to_string()]);

        assert_eq!((transaction.from, transaction.to, transaction.value), (original.from, original.to, original.value));
    }

    #[test]
    fn test_asset_transfer_metadata() {
        let usdc = Asset::mock_ethereum_usdc().id;
        let original = Transaction {
            transaction_type: TransactionType::SmartContractCall,
            metadata: Some(
                serde_json::to_value(TransactionAssetTransfersMetadata {
                    asset_transfers: vec![
                        TransactionAssetTransfer {
                            asset_id: usdc.clone(),
                            from: "0xContract".to_string(),
                            to: "0xUser".to_string(),
                            value: BigUint::from(10u8),
                        },
                        TransactionAssetTransfer {
                            asset_id: usdc.clone(),
                            from: "0xContract".to_string(),
                            to: "0xOther".to_string(),
                            value: BigUint::from(20u8),
                        },
                    ],
                })
                .unwrap(),
            ),
            ..Transaction::mock()
        };

        assert_eq!(original.asset_ids().into_iter().collect::<HashSet<_>>(), HashSet::from([Asset::mock().id, usdc.clone()]));
        assert_eq!(original.assets_addresses().len(), 3);
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Transaction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.asset_transfers_metadata(), original.asset_transfers_metadata());

        let incoming = original.finalize(vec!["0xuser".to_string()]);
        assert_eq!(incoming.transaction_type, TransactionType::Transfer);
        assert_eq!(incoming.asset_id, usdc);
        assert_eq!(incoming.from, "0xContract");
        assert_eq!(incoming.to, "0xUser");
        assert_eq!(incoming.value, BigUint::from(10u32));
        assert_eq!(incoming.direction, TransactionDirection::Incoming);
        assert_eq!(incoming.metadata, None);

        let ambiguous = original.finalize(vec!["0xUser".to_string(), "0xOther".to_string()]);
        assert_eq!(ambiguous.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(ambiguous.metadata, original.metadata);
    }

    #[test]
    fn test_transaction_json_keeps_value_and_fee_as_decimal_strings() {
        let transaction = Transaction::new(
            "hash".to_string(),
            AssetId::from_chain(Chain::Ethereum),
            "from".to_string(),
            "to".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            BigUint::from(21_000u32),
            AssetId::from_chain(Chain::Ethereum),
            BigUint::from(1_000_000_000_000_000_000u64),
            None,
            None,
            Utc::now(),
        );
        let json = serde_json::to_value(&transaction).unwrap();

        assert_eq!(json["fee"], serde_json::json!("21000"));
        assert_eq!(json["value"], serde_json::json!("1000000000000000000"));
        assert_eq!(serde_json::from_value::<Transaction>(json).unwrap().value, transaction.value);
    }
}
