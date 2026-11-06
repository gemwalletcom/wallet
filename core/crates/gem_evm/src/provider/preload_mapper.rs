use std::error::Error;

use alloy_primitives::hex;
use num_bigint::BigInt;
use num_traits::Num;
use primitives::swap::{SwapQuoteData, SwapQuoteDataType};
use primitives::{Asset, AssetSubtype, ContractCallData, EVMChain, FeeRate, NFTType, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransferDataExtra, decode_hex, fee::FeePriority, fee::GasPriceType};

use crate::constants::TRANSFER_GAS_LIMIT;
use crate::encode::{encode_erc20_approve_max_value, encode_erc20_transfer, encode_erc721_transfer, encode_erc1155_transfer};
use crate::fee_calculator::FeeCalculator;
use crate::models::fee::EthereumFeeHistory;

const GAS_LIMIT_PERCENT_INCREASE: u32 = 50;

pub use crate::transaction_params::TransactionParams;

pub fn bigint_to_hex_string(value: &BigInt) -> String {
    format!("0x{:x}", value)
}

pub fn bytes_to_hex_string(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

pub fn map_transaction_preload(nonce_hex: String, chain_id: String) -> Result<TransactionLoadMetadata, Box<dyn std::error::Error + Send + Sync>> {
    let nonce = u64::from_str_radix(nonce_hex.trim_start_matches("0x"), 16)?;
    Ok(TransactionLoadMetadata::Evm {
        nonce,
        chain_id: chain_id.parse::<u64>()?,
        contract_call: None,
    })
}

pub fn map_transaction_fee_rates(chain: EVMChain, fee_history: &EthereumFeeHistory) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
    let base_fee = fee_history.base_fee_per_gas.last().ok_or("No base fee available")?;

    Ok(FeeCalculator::new()
        .calculate_priority_fees(chain, fee_history, &[FeePriority::Normal, FeePriority::Fast])?
        .into_iter()
        .map(|fee| FeeRate::new(fee.priority, GasPriceType::eip1559(base_fee.clone(), fee.value)))
        .collect())
}

pub fn get_transaction_params(input: &TransactionLoadInput) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    let value = input.value_as_bigint();

    match &input.input_type {
        TransactionInputType::Transfer { asset } | TransactionInputType::Withdrawal { asset } | TransactionInputType::Deposit { asset } => match asset.id.token_subtype() {
            AssetSubtype::NATIVE => Ok(TransactionParams::new(input.destination_address.clone(), vec![], value)),
            AssetSubtype::TOKEN => {
                let to = asset.token_id().ok_or("Missing token ID")?;
                let data = encode_erc20_transfer(&input.destination_address, &value)?;
                Ok(TransactionParams::new(to, data, BigInt::from(0)))
            }
        },
        TransactionInputType::TransferNft { nft_asset, .. } => {
            let contract_address = nft_asset.contract_address.as_ref().ok_or("Missing contract address")?;
            let data = match nft_asset.token_type {
                NFTType::ERC721 => encode_erc721_transfer(&input.sender_address, &input.destination_address, &nft_asset.token_id)?,
                NFTType::ERC1155 => encode_erc1155_transfer(&input.sender_address, &input.destination_address, &nft_asset.token_id)?,
                _ => return Err("Unsupported NFT type for EVM".into()),
            };
            Ok(TransactionParams::new(contract_address.clone(), data, BigInt::from(0)))
        }
        TransactionInputType::Swap { from_asset, swap_data, .. } => {
            if let Some(approval) = &swap_data.data.approval {
                Ok(TransactionParams::new_approval(approval.token.clone(), encode_erc20_approve_max_value(&approval.spender)?))
            } else {
                swap_transaction_params(input, from_asset, &swap_data.data)
            }
        }
        TransactionInputType::TokenApprove { approval_data: approval, .. }
        | TransactionInputType::Payment {
            extra: TransferDataExtra { approval: Some(approval), .. },
            ..
        } => Ok(TransactionParams::new_approval(approval.token.clone(), encode_erc20_approve_max_value(&approval.spender)?)),
        TransactionInputType::Generic { extra, .. } | TransactionInputType::Payment { extra, .. } => Ok(TransactionParams::new(extra.to.clone(), extra.data.clone().unwrap_or_default(), value)),
        TransactionInputType::Stake { .. } => Err("Unsupported chain for staking".into()),
        TransactionInputType::Earn { data: earn_data, .. } => {
            if let Some(approval) = &earn_data.approval {
                Ok(TransactionParams::new_approval(approval.token.clone(), encode_erc20_approve_max_value(&approval.spender)?))
            } else {
                earn_transaction_params(earn_data)
            }
        }
        _ => Err("Unsupported transfer type".into()),
    }
}

pub fn get_approved_transaction_params(input: &TransactionLoadInput) -> Result<Option<TransactionParams>, Box<dyn Error + Send + Sync>> {
    match &input.input_type {
        TransactionInputType::Swap { from_asset, swap_data, .. } if swap_data.data.approval.is_some() => Ok(Some(swap_transaction_params(input, from_asset, &swap_data.data)?)),
        TransactionInputType::Earn { data: earn_data, .. } if earn_data.approval.is_some() => Ok(Some(earn_transaction_params(earn_data)?)),
        _ => Ok(None),
    }
}

fn swap_transaction_params(input: &TransactionLoadInput, from_asset: &Asset, swap_data: &SwapQuoteData) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    match swap_data.data_type {
        SwapQuoteDataType::Contract => Ok(TransactionParams::new(swap_data.to.clone(), decode_hex(&swap_data.data)?, BigInt::from(swap_data.value.clone()))),
        SwapQuoteDataType::Transfer => match from_asset.id.token_subtype() {
            AssetSubtype::NATIVE => {
                let value = if input.is_max_value { BigInt::ZERO } else { BigInt::from(swap_data.value.clone()) };
                Ok(TransactionParams::new(swap_data.to.clone(), decode_hex(&swap_data.data)?, value))
            }
            AssetSubtype::TOKEN => {
                let to = from_asset.token_id().ok_or("Missing token ID")?;
                let data = encode_erc20_transfer(&swap_data.to, &input.value_as_bigint())?;
                Ok(TransactionParams::new(to, data, BigInt::ZERO))
            }
        },
    }
}

fn earn_transaction_params(earn_data: &ContractCallData) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    Ok(TransactionParams::new(earn_data.contract_address.clone(), decode_hex(&earn_data.call_data)?, BigInt::ZERO))
}

pub fn calculate_gas_limit_with_increase(gas_limit: BigInt) -> BigInt {
    if gas_limit == BigInt::from(TRANSFER_GAS_LIMIT) {
        gas_limit
    } else {
        gas_limit * BigInt::from(100 + GAS_LIMIT_PERCENT_INCREASE) / BigInt::from(100)
    }
}

pub fn pending_permit_gas_limit(input: &TransactionLoadInput) -> Result<Option<BigInt>, Box<dyn Error + Send + Sync>> {
    let TransactionInputType::Swap { swap_data, .. } = &input.input_type else {
        return Ok(None);
    };
    if swap_data.data.approval.is_some() || swap_data.data.permit2.is_none() {
        return Ok(None);
    }
    let gas_limit = swap_data.data.gas_limit.as_deref().ok_or("swap gas limit is required while its permit is unsigned")?;
    Ok(Some(BigInt::from_str_radix(gas_limit, 10)?))
}

pub fn requested_gas_limit(input: &TransactionLoadInput) -> Option<BigInt> {
    let TransactionInputType::Generic { extra, .. } = &input.input_type else {
        return None;
    };
    extra.gas_limit.clone()
}

pub fn get_extra_fee_gas_limit(input: &TransactionLoadInput) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    match &input.input_type {
        TransactionInputType::Swap { swap_data, .. } => {
            if swap_data.data.approval.is_some() {
                if let Some(ref gas_limit) = swap_data.data.gas_limit {
                    Ok(BigInt::from_str_radix(gas_limit, 10)?)
                } else {
                    Ok(BigInt::from(0))
                }
            } else {
                Ok(BigInt::from(0))
            }
        }
        TransactionInputType::Earn { data: earn_data, .. } => {
            if earn_data.approval.is_some()
                && let Some(gas_limit) = &earn_data.gas_limit
            {
                return Ok(BigInt::from_str_radix(gas_limit, 10)?);
            }
            Ok(BigInt::from(0))
        }
        _ => Ok(BigInt::from(0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::swap::{ApprovalData, Permit2ApprovalData, SwapData};
    use primitives::testkit::signer_mock::TEST_EVM_RECIPIENT;
    use primitives::{DelegationValidator, EarnType};

    #[test]
    fn test_get_transaction_params_contract_swap() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let input = TransactionLoadInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock_erc20(),
                to_asset: Asset::mock_eth(),
                swap_data: SwapData {
                    data: SwapQuoteData::mock_contract_call(TEST_EVM_RECIPIENT, "500", "0xabcd", None),
                    ..SwapData::mock()
                },
            },
            "1000",
        );

        assert_eq!(get_transaction_params(&input)?, TransactionParams::new(TEST_EVM_RECIPIENT, vec![0xab, 0xcd], BigInt::from(500)));
        Ok(())
    }

    #[test]
    fn test_get_approved_transaction_params() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let approved_swap = TransactionLoadInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock_erc20(),
                to_asset: Asset::mock_eth(),
                swap_data: SwapData {
                    data: SwapQuoteData {
                        approval: Some(ApprovalData::mock()),
                        ..SwapQuoteData::mock_contract_call(TEST_EVM_RECIPIENT, "500", "0xabcd", None)
                    },
                    ..SwapData::mock()
                },
            },
            "1000",
        );
        let approved_earn = TransactionLoadInput::mock_evm(
            TransactionInputType::Earn {
                asset: Asset::mock_erc20(),
                earn_type: EarnType::Deposit(DelegationValidator::mock()),
                data: ContractCallData {
                    approval: Some(ApprovalData::mock()),
                    ..ContractCallData::mock()
                },
            },
            "1000",
        );
        let swap_without_approval = TransactionLoadInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock_erc20(),
                to_asset: Asset::mock_eth(),
                swap_data: SwapData::mock(),
            },
            "1000",
        );

        assert_eq!(get_approved_transaction_params(&approved_swap)?, Some(TransactionParams::new(TEST_EVM_RECIPIENT, vec![0xab, 0xcd], BigInt::from(500))));
        assert_eq!(get_approved_transaction_params(&approved_earn)?, Some(TransactionParams::new(TEST_EVM_RECIPIENT, vec![0xab, 0xcd], BigInt::ZERO)));
        assert_eq!(get_approved_transaction_params(&swap_without_approval)?, None);
        assert_eq!(get_approved_transaction_params(&TransactionLoadInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock_eth() }, "1000"))?, None);
        Ok(())
    }

    #[test]
    fn test_get_transaction_params_payment() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let approval = ApprovalData::mock();
        let payment = |approval| TransactionInputType::mock_payment(Asset::mock_erc20(), TransferDataExtra { approval, ..TransferDataExtra::mock() });

        assert_eq!(
            get_transaction_params(&TransactionLoadInput::mock_evm(payment(Some(approval.clone())), "1000"))?,
            TransactionParams::new_approval(approval.token.clone(), encode_erc20_approve_max_value(&approval.spender)?),
            "a payment that still needs an approval sends the approval"
        );
        assert_eq!(
            get_transaction_params(&TransactionLoadInput::mock_evm(payment(None), "1000"))?,
            TransactionParams::new(TEST_EVM_RECIPIENT, vec![], BigInt::from(1000)),
            "a payment paid by a transaction sends what the gateway built"
        );
        Ok(())
    }

    #[test]
    fn test_map_transaction_preload_with_hex_prefix() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let nonce_hex = "0xa".to_string();
        let chain_id = "1".to_string();

        let result = map_transaction_preload(nonce_hex, chain_id)?;

        match result {
            TransactionLoadMetadata::Evm { nonce, chain_id, contract_call } => {
                assert_eq!(nonce, 10);
                assert_eq!(chain_id, 1);
                assert!(contract_call.is_none());
            }
            _ => panic!("Expected Evm variant"),
        }

        Ok(())
    }

    #[test]
    fn test_map_transaction_preload_invalid_nonce() {
        let nonce_hex = "invalid".to_string();
        let chain_id_hex = "0x1".to_string();

        let result = map_transaction_preload(nonce_hex, chain_id_hex);

        assert!(result.is_err());
    }

    #[test]
    fn test_map_transaction_preload_invalid_chain_id() {
        let nonce_hex = "0x1".to_string();
        let chain_id_hex = "invalid".to_string();

        let result = map_transaction_preload(nonce_hex, chain_id_hex);

        assert!(result.is_err());
    }

    #[test]
    fn test_map_transaction_fee_rates_normal_case() -> Result<(), Box<dyn Error + Sync + Send>> {
        let fee_history = EthereumFeeHistory::mock();

        let result = map_transaction_fee_rates(EVMChain::Ethereum, &fee_history)?;

        assert_eq!(
            result.into_iter().map(|rate| (rate.priority, rate.gas_price_type)).collect::<Vec<_>>(),
            vec![
                (FeePriority::Normal, GasPriceType::eip1559(20_000_000_000u64, 200_000_000u64)),
                (FeePriority::Fast, GasPriceType::eip1559(20_000_000_000u64, 2_220_000_000u64)),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_map_transaction_fee_rates_next_block_base_fee() {
        let history = EthereumFeeHistory {
            base_fee_per_gas: vec![50, 10, 40, 30].into_iter().map(BigInt::from).collect(),
            ..EthereumFeeHistory::mock()
        };
        for chain in EVMChain::all() {
            let rates = map_transaction_fee_rates(chain, &history).unwrap();

            assert_eq!(rates.into_iter().map(|rate| rate.gas_price_type.gas_price()).collect::<Vec<_>>(), vec![BigInt::from(30), BigInt::from(30)], "{chain:?}");
        }
    }

    #[test]
    fn test_map_transaction_fee_rates_empty_base_fees() {
        let history = EthereumFeeHistory {
            base_fee_per_gas: vec![],
            ..EthereumFeeHistory::mock()
        };

        assert_eq!(map_transaction_fee_rates(EVMChain::Ethereum, &history).unwrap_err().to_string(), "No base fee available");
    }

    #[test]
    fn test_map_transaction_fee_rates_zero_base_fee() -> Result<(), Box<dyn Error + Sync + Send>> {
        let fee_history = EthereumFeeHistory {
            base_fee_per_gas: vec![BigInt::from(0u64)],
            ..EthereumFeeHistory::mock()
        };

        let result = map_transaction_fee_rates(EVMChain::SmartChain, &fee_history)?;

        assert_eq!(result.len(), 2);

        assert_eq!(result[0].gas_price_type.gas_price(), BigInt::ZERO);
        assert!(result[0].gas_price_type.priority_fee() != BigInt::ZERO);

        Ok(())
    }

    #[test]
    fn test_map_transaction_fee_rates_invalid_hex() {
        let fee_history = EthereumFeeHistory {
            reward: vec![vec!["invalid_hex".to_string()]],
            ..EthereumFeeHistory::mock()
        };

        let result = map_transaction_fee_rates(EVMChain::Ethereum, &fee_history);
        assert!(result.is_err());
    }

    #[test]
    fn test_pending_permit_gas_limit() {
        let swap = |permit2: Option<Permit2ApprovalData>, approval: Option<ApprovalData>, gas_limit: Option<&str>| {
            TransactionLoadInput::mock_evm(
                TransactionInputType::Swap {
                    from_asset: Asset::mock_erc20(),
                    to_asset: Asset::mock_eth(),
                    swap_data: SwapData {
                        data: SwapQuoteData {
                            permit2,
                            approval,
                            gas_limit: gas_limit.map(String::from),
                            ..SwapQuoteData::mock()
                        },
                        ..SwapData::mock()
                    },
                },
                "1000",
            )
        };

        assert_eq!(
            pending_permit_gas_limit(&swap(Some(Permit2ApprovalData::mock()), None, Some("300000"))).unwrap(),
            Some(BigInt::from(300_000)),
            "an unsigned permit cannot be estimated, so the provider's limit is used"
        );
        assert_eq!(
            pending_permit_gas_limit(&swap(Some(Permit2ApprovalData::mock()), Some(ApprovalData::mock()), Some("300000"))).unwrap(),
            None,
            "a pending approval is the transaction that gets estimated"
        );
        assert_eq!(pending_permit_gas_limit(&swap(None, None, Some("300000"))).unwrap(), None, "signed calldata is estimated");
        assert!(pending_permit_gas_limit(&swap(Some(Permit2ApprovalData::mock()), None, None)).is_err());
    }

    #[test]
    fn test_calculate_gas_limit_with_increase() {
        let gas_21000 = BigInt::from(21000);
        let result = calculate_gas_limit_with_increase(gas_21000.clone());
        assert_eq!(result, gas_21000);

        let gas_100000 = BigInt::from(100000);
        let result = calculate_gas_limit_with_increase(gas_100000);
        assert_eq!(result, BigInt::from(150000));
    }

    #[test]
    fn test_bigint_to_string_conversion() {
        let value = BigInt::from(100_000_000u64);
        assert_eq!(value.to_string(), "100000000");
    }

    #[test]
    fn test_encode_erc721_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let from = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
        let to = "0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199";
        let token_id = "1234";

        let result = encode_erc721_transfer(from, to, token_id)?;

        assert!(!result.is_empty());
        let selector = &result[0..4];
        assert_eq!(hex::encode(selector), "42842e0e");

        Ok(())
    }

    #[test]
    fn test_encode_erc1155_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let from = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
        let to = "0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199";
        let token_id = "5678";

        let result = encode_erc1155_transfer(from, to, token_id)?;

        assert!(!result.is_empty());
        let selector = &result[0..4];
        assert_eq!(hex::encode(selector), "f242432a");

        Ok(())
    }
}
