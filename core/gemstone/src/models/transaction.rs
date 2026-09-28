use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::services::assets::model::GemFeeAmount;
use chain_primitives::checksum_address;
use primitives::contract_call_data::ContractCallData;
use primitives::{AssetId, EarnType, FeeOption, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransactionType};
use std::collections::HashMap;

pub type GemFeeOption = FeeOption;

#[uniffi::remote(Enum)]
pub enum FeeOption {
    TokenAccountCreation,
}

pub type GemContractCallData = ContractCallData;

pub type GemEarnType = EarnType;

#[derive(Debug, Clone)]
pub struct GemTransactionLoadInput {
    pub input_type: TransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub value: GemBigUint,
    pub gas_price: GasPriceType,
    pub memo: Option<String>,
    pub is_max_value: bool,
    pub metadata: GemTransactionLoadMetadata,
}

#[derive(Debug, Clone)]
pub struct GemSignerInput {
    pub input: GemTransactionLoadInput,
    pub fee: GemTransactionLoadFee,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GemSignedTransaction {
    pub data: String,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFeeOptionItem {
    pub option: GemFeeOption,
    pub value: GemBigInt,
    pub amount: GemFeeAmount,
}

#[derive(Debug, Default, Clone)]
pub struct GemFeeOptions {
    pub options: HashMap<GemFeeOption, GemBigInt>,
}

#[derive(Debug, Clone)]
pub struct GemTransactionLoadFee {
    pub fee: GemBigInt,
    pub gas_price_type: GasPriceType,
    pub gas_limit: GemBigInt,
    pub options: GemFeeOptions,
    pub fee_asset: AssetId,
}

#[derive(Debug, Clone)]
pub struct GemTransactionData {
    pub fee: GemTransactionLoadFee,
    pub metadata: GemTransactionLoadMetadata,
}

pub type GemTransactionLoadMetadata = TransactionLoadMetadata;

impl From<GemTransactionLoadInput> for TransactionLoadInput {
    fn from(value: GemTransactionLoadInput) -> Self {
        let input_type: TransactionInputType = value.input_type;
        let destination_address = checksum_address(&value.destination_address, input_type.get_asset().chain());
        TransactionLoadInput {
            input_type,
            sender_address: value.sender_address,
            destination_address,
            value: value.value,
            gas_price: value.gas_price,
            memo: value.memo,
            is_max_value: value.is_max_value,
            metadata: value.metadata,
        }
    }
}

impl From<GemSignerInput> for SignerInput {
    fn from(value: GemSignerInput) -> Self {
        SignerInput::new(value.input.into(), value.fee.into())
    }
}

impl From<TransactionLoadInput> for GemTransactionLoadInput {
    fn from(value: TransactionLoadInput) -> Self {
        GemTransactionLoadInput {
            input_type: value.input_type,
            sender_address: value.sender_address,
            destination_address: value.destination_address,
            value: value.value,
            gas_price: value.gas_price,
            memo: value.memo,
            is_max_value: value.is_max_value,
            metadata: value.metadata,
        }
    }
}

impl From<SignerInput> for GemSignerInput {
    fn from(value: SignerInput) -> Self {
        GemSignerInput {
            input: value.input.into(),
            fee: value.fee.into(),
        }
    }
}

pub fn transaction_metadata_block_number(metadata: &GemTransactionLoadMetadata) -> Option<String> {
    match metadata {
        GemTransactionLoadMetadata::Polkadot { block_number, .. } | GemTransactionLoadMetadata::Tron { block_number, .. } | GemTransactionLoadMetadata::Xrp { block_number, .. } | GemTransactionLoadMetadata::Cardano { block_number, .. } => {
            Some(block_number.to_string())
        }
        _ => None,
    }
}

pub fn transaction_metadata_sequence(metadata: &GemTransactionLoadMetadata) -> Option<String> {
    match metadata {
        GemTransactionLoadMetadata::Ton { sequence, .. }
        | GemTransactionLoadMetadata::Cosmos { sequence, .. }
        | GemTransactionLoadMetadata::Near { sequence, .. }
        | GemTransactionLoadMetadata::Stellar { sequence, .. }
        | GemTransactionLoadMetadata::Xrp { sequence, .. }
        | GemTransactionLoadMetadata::Algorand { sequence, .. }
        | GemTransactionLoadMetadata::Aptos { sequence, .. }
        | GemTransactionLoadMetadata::Polkadot { sequence, .. } => Some(sequence.to_string()),
        GemTransactionLoadMetadata::Evm { nonce, .. } => Some(nonce.to_string()),
        _ => None,
    }
}

impl GemFeeOptions {
    pub fn get(&self, option: &GemFeeOption) -> Option<&GemBigInt> {
        self.options.get(option)
    }

    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    pub fn total(&self) -> GemBigInt {
        self.options.values().sum()
    }

    pub fn items(&self, amount: impl Fn(&GemBigInt) -> GemFeeAmount) -> Vec<GemFeeOptionItem> {
        let mut items = self
            .options
            .iter()
            .map(|(option, value)| GemFeeOptionItem {
                option: option.clone(),
                value: value.clone(),
                amount: amount(value),
            })
            .collect::<Vec<_>>();
        items.sort_by(|left, right| left.option.as_ref().cmp(right.option.as_ref()));
        items
    }
}

impl From<GemTransactionLoadFee> for TransactionFee {
    fn from(value: GemTransactionLoadFee) -> Self {
        TransactionFee {
            fee: value.fee,
            gas_price_type: value.gas_price_type,
            gas_limit: value.gas_limit,
            options: value.options.options,
            fee_asset: value.fee_asset,
        }
    }
}

impl From<TransactionFee> for GemTransactionLoadFee {
    fn from(value: TransactionFee) -> Self {
        GemTransactionLoadFee {
            fee: value.fee,
            gas_price_type: value.gas_price_type,
            gas_limit: value.gas_limit,
            options: GemFeeOptions { options: value.options },
            fee_asset: value.fee_asset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_fee_items_preserve_total() {
        let fee = GemTransactionLoadFee {
            fee: 1_495_940.into(),
            gas_price_type: GasPriceType::solana(5_000u64, 2_500u64, 25_000u64),
            gas_limit: 100_000.into(),
            options: GemFeeOptions {
                options: HashMap::from([(FeeOption::TokenAccountCreation, 1_488_440.into())]),
            },
            fee_asset: AssetId::from_chain(Chain::Solana),
        };

        let amount = |value: &GemBigInt| crate::services::assets::rules::fee_amount(&primitives::Asset::from_chain(Chain::Solana), value, None, primitives::currency::Currency::USD);
        assert_eq!(
            fee.options.items(amount),
            vec![GemFeeOptionItem {
                option: FeeOption::TokenAccountCreation,
                value: 1_488_440.into(),
                amount: amount(&1_488_440.into()),
            }]
        );
        assert_eq!(fee.fee, 1_495_940.into());

        let fee = GemTransactionLoadFee {
            fee: 7_500.into(),
            options: GemFeeOptions::default(),
            ..fee
        };
        assert_eq!(fee.options.items(amount), vec![]);
    }

    #[test]
    fn test_metadata_without_a_block_or_sequence_stores_none() {
        let bitcoin = GemTransactionLoadMetadata::Bitcoin { utxos: vec![] };
        assert_eq!(transaction_metadata_block_number(&bitcoin), None);
        assert_eq!(transaction_metadata_sequence(&bitcoin), None);

        let xrp = GemTransactionLoadMetadata::Xrp {
            sequence: 7,
            block_number: 91,
            is_destination_address_exist: true,
        };
        assert_eq!(transaction_metadata_block_number(&xrp), Some("91".to_string()));
        assert_eq!(transaction_metadata_sequence(&xrp), Some("7".to_string()));
    }
}
