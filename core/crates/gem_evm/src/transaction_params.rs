use std::error::Error;

use num_bigint::BigInt;
use primitives::swap::{ApprovalData, SwapQuoteData, SwapQuoteDataType};
use primitives::{Asset, AssetSubtype, ContractCallData, NFTType, TransactionInputType, TransactionLoadInput, TransferDataExtra, decode_hex};

use crate::encode::{encode_erc20_approve_max_value, encode_erc20_transfer, encode_erc721_transfer, encode_erc1155_transfer};

#[derive(Debug, PartialEq)]
pub struct TransactionParams {
    pub to: String,
    pub data: Vec<u8>,
    pub value: BigInt,
}

impl TransactionParams {
    pub fn new(to: impl Into<String>, data: Vec<u8>, value: BigInt) -> Self {
        Self { to: to.into(), data, value }
    }

    pub fn new_approval(approval: &ApprovalData) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self::new(approval.token.clone(), encode_erc20_approve_max_value(&approval.spender)?, BigInt::ZERO))
    }
}

#[derive(Debug, PartialEq)]
pub enum TransactionCalls {
    Call(TransactionParams),
    ApproveAndCall { approval: TransactionParams, call: TransactionParams, call_gas_limit: u64 },
}

impl TransactionCalls {
    pub fn first(&self) -> &TransactionParams {
        match self {
            Self::Call(call) => call,
            Self::ApproveAndCall { approval, .. } => approval,
        }
    }

    pub fn approved_call_gas_limit(&self) -> u64 {
        match self {
            Self::Call(_) => 0,
            Self::ApproveAndCall { call_gas_limit, .. } => *call_gas_limit,
        }
    }
}

pub fn transaction_calls(input: &TransactionLoadInput) -> Result<TransactionCalls, Box<dyn Error + Send + Sync>> {
    match &input.input_type {
        TransactionInputType::Swap { from_asset, swap_data, .. } => with_approval(swap_data.data.approval.as_ref(), swap_data.data.gas_limit.as_deref(), swap_call(input, from_asset, &swap_data.data)?),
        TransactionInputType::Earn { data, .. } => with_approval(data.approval.as_ref(), data.gas_limit.as_deref(), earn_call(data)?),
        _ => Ok(TransactionCalls::Call(transaction_call(input)?)),
    }
}

pub fn transaction_call(input: &TransactionLoadInput) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    let value = input.value_as_bigint();
    match &input.input_type {
        TransactionInputType::Transfer { asset } | TransactionInputType::Withdrawal { asset } | TransactionInputType::Deposit { asset } => match asset.id.token_subtype() {
            AssetSubtype::NATIVE => Ok(TransactionParams::new(input.destination_address.clone(), vec![], value)),
            AssetSubtype::TOKEN => {
                let to = asset.token_id().ok_or("Missing token ID")?;
                Ok(TransactionParams::new(to, encode_erc20_transfer(&input.destination_address, &value)?, BigInt::ZERO))
            }
        },
        TransactionInputType::TransferNft { nft_asset, .. } => {
            let contract_address = nft_asset.contract_address.as_ref().ok_or("Missing contract address")?;
            let data = match nft_asset.token_type {
                NFTType::ERC721 => encode_erc721_transfer(&input.sender_address, &input.destination_address, &nft_asset.token_id)?,
                NFTType::ERC1155 => encode_erc1155_transfer(&input.sender_address, &input.destination_address, &nft_asset.token_id)?,
                _ => return Err("Unsupported NFT type for EVM".into()),
            };
            Ok(TransactionParams::new(contract_address.clone(), data, BigInt::ZERO))
        }
        TransactionInputType::TokenApprove { approval_data: approval, .. }
        | TransactionInputType::Payment {
            extra: TransferDataExtra { approval: Some(approval), .. },
            ..
        } => TransactionParams::new_approval(approval),
        TransactionInputType::Generic { extra, .. } | TransactionInputType::Payment { extra, .. } => Ok(TransactionParams::new(extra.to.clone(), extra.data.clone().unwrap_or_default(), value)),
        TransactionInputType::Stake { .. } => Err("Unsupported chain for staking".into()),
        _ => Err("Unsupported transfer type".into()),
    }
}

fn with_approval(approval: Option<&ApprovalData>, call_gas_limit: Option<&str>, call: TransactionParams) -> Result<TransactionCalls, Box<dyn Error + Send + Sync>> {
    let Some(approval) = approval else {
        return Ok(TransactionCalls::Call(call));
    };
    Ok(TransactionCalls::ApproveAndCall {
        approval: TransactionParams::new_approval(approval)?,
        call,
        call_gas_limit: call_gas_limit.ok_or("missing gas limit for the call after approval")?.parse()?,
    })
}

fn swap_call(input: &TransactionLoadInput, from_asset: &Asset, swap_data: &SwapQuoteData) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    match swap_data.data_type {
        SwapQuoteDataType::Contract => Ok(TransactionParams::new(swap_data.to.clone(), decode_hex(&swap_data.data)?, BigInt::from(swap_data.value.clone()))),
        SwapQuoteDataType::Transfer => match from_asset.id.token_subtype() {
            AssetSubtype::NATIVE => {
                let value = if input.is_max_value { BigInt::ZERO } else { BigInt::from(swap_data.value.clone()) };
                Ok(TransactionParams::new(swap_data.to.clone(), decode_hex(&swap_data.data)?, value))
            }
            AssetSubtype::TOKEN => {
                let to = from_asset.token_id().ok_or("Missing token ID")?;
                Ok(TransactionParams::new(to, encode_erc20_transfer(&swap_data.to, &input.value_as_bigint())?, BigInt::ZERO))
            }
        },
    }
}

fn earn_call(data: &ContractCallData) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    Ok(TransactionParams::new(data.contract_address.clone(), decode_hex(&data.call_data)?, BigInt::ZERO))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::testkit::signer_mock::TEST_EVM_RECIPIENT;

    #[test]
    fn test_transaction_calls() {
        let approval = ApprovalData::mock();
        let swap_data = SwapQuoteData::mock_contract_call(TEST_EVM_RECIPIENT, "500", "0xabcd", None);

        assert_eq!(
            transaction_calls(&TransactionLoadInput::mock_evm(TransactionInputType::mock_swap(Asset::mock_erc20(), swap_data.clone()), "1000")).unwrap(),
            TransactionCalls::Call(TransactionParams::new(TEST_EVM_RECIPIENT, vec![0xab, 0xcd], BigInt::from(500))),
            "a token swap sends the quote's value"
        );

        let approved_swap = SwapQuoteData {
            approval: Some(approval.clone()),
            gas_limit: Some("200000".to_string()),
            ..swap_data.clone()
        };
        assert_eq!(
            transaction_calls(&TransactionLoadInput::mock_evm(TransactionInputType::mock_swap(Asset::mock_erc20(), approved_swap.clone()), "1000")).unwrap(),
            TransactionCalls::ApproveAndCall {
                approval: TransactionParams::new_approval(&approval).unwrap(),
                call: TransactionParams::new(TEST_EVM_RECIPIENT, vec![0xab, 0xcd], BigInt::from(500)),
                call_gas_limit: 200_000,
            }
        );
        assert_eq!(
            transaction_calls(&TransactionLoadInput::mock_evm(
                TransactionInputType::mock_swap(Asset::mock_erc20(), SwapQuoteData { gas_limit: None, ..approved_swap }),
                "1000"
            ))
            .unwrap_err()
            .to_string(),
            "missing gas limit for the call after approval"
        );

        let earn = ContractCallData {
            approval: Some(approval.clone()),
            gas_limit: Some("300000".to_string()),
            ..ContractCallData::mock()
        };
        assert_eq!(
            transaction_calls(&TransactionLoadInput::mock_evm(TransactionInputType::mock_earn(earn), "0")).unwrap(),
            TransactionCalls::ApproveAndCall {
                approval: TransactionParams::new_approval(&approval).unwrap(),
                call: TransactionParams::new(TEST_EVM_RECIPIENT, vec![0xab, 0xcd], BigInt::ZERO),
                call_gas_limit: 300_000,
            }
        );

        let payment = TransactionInputType::mock_payment(
            Asset::mock_erc20(),
            TransferDataExtra {
                approval: Some(approval.clone()),
                ..TransferDataExtra::mock()
            },
        );
        assert_eq!(
            transaction_calls(&TransactionLoadInput::mock_evm(payment, "1000")).unwrap(),
            TransactionCalls::Call(TransactionParams::new_approval(&approval).unwrap()),
            "a payment that needs an approval sends only the approval"
        );
        assert_eq!(
            transaction_calls(&TransactionLoadInput::mock_evm(TransactionInputType::mock_payment(Asset::mock_erc20(), TransferDataExtra::mock()), "1000")).unwrap(),
            TransactionCalls::Call(TransactionParams::new(TEST_EVM_RECIPIENT, vec![], BigInt::from(1000)))
        );
    }
}
