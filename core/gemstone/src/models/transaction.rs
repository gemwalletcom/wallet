use crate::models::custom_types::GemBigInt;
use crate::services::assets::model::GemFeeAmount;
use primitives::{FeeOption, TransactionLoadMetadata, TransactionType};
use std::collections::HashMap;

#[uniffi::remote(Enum)]
pub enum FeeOption {
    TokenAccountCreation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GemSignedTransaction {
    pub data: String,
    pub transaction_type: TransactionType,
    pub sequence: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFeeOptionItem {
    pub option: FeeOption,
    pub value: GemBigInt,
    pub amount: GemFeeAmount,
}

pub fn transaction_metadata_block_number(metadata: &TransactionLoadMetadata) -> Option<String> {
    match metadata {
        TransactionLoadMetadata::Polkadot { block_number, .. } | TransactionLoadMetadata::Tron { block_number, .. } | TransactionLoadMetadata::Xrp { block_number, .. } | TransactionLoadMetadata::Cardano { block_number, .. } => {
            Some(block_number.to_string())
        }
        _ => None,
    }
}

pub fn fee_option_items(options: &HashMap<FeeOption, GemBigInt>, amount: impl Fn(&GemBigInt) -> GemFeeAmount) -> Vec<GemFeeOptionItem> {
    let mut items = options
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_metadata_without_a_block_stores_none() {
        let bitcoin = TransactionLoadMetadata::Bitcoin { utxos: vec![] };
        assert_eq!(transaction_metadata_block_number(&bitcoin), None);

        let xrp = TransactionLoadMetadata::Xrp {
            sequence: 7,
            block_number: 91,
            is_destination_address_exist: true,
        };
        assert_eq!(transaction_metadata_block_number(&xrp), Some("91".to_string()));
    }
}
