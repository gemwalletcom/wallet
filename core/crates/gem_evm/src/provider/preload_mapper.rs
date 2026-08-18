use std::error::Error;

use alloy_primitives::hex;
use num_bigint::BigInt;
use num_traits::Num;
use primitives::swap::SwapQuoteDataType;
use primitives::{AssetSubtype, EVMChain, FeeRate, NFTType, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, decode_hex, fee::FeePriority, fee::GasPriceType};

use crate::constants::TRANSFER_GAS_LIMIT;
use crate::encode::{encode_erc20_approve_max_value, encode_erc20_transfer, encode_erc721_transfer, encode_erc1155_transfer};
use crate::fee_calculator::FeeCalculator;
use crate::models::fee::EthereumFeeHistory;

const GAS_LIMIT_PERCENT_INCREASE: u32 = 50;

pub struct TransactionParams {
    pub to: String,
    pub data: Vec<u8>,
    pub value: BigInt,
}

impl TransactionParams {
    pub fn new(to: String, data: Vec<u8>, value: BigInt) -> Self {
        Self { to, data, value }
    }

    pub fn new_approval(to: String, data: Vec<u8>) -> Self {
        Self { to, data, value: BigInt::from(0) }
    }
}

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
    map_eip1559_fee_rates(chain, fee_history)
}

pub fn map_eip1559_fee_rates(chain: EVMChain, fee_history: &EthereumFeeHistory) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
    let base_fee = fee_history.base_fee_per_gas.last().ok_or("No base fee available")?;
    let min_priority_fee = BigInt::from(chain.min_priority_fee());

    Ok(FeeCalculator::new()
        .calculate_priority_fees(fee_history, &[FeePriority::Normal, FeePriority::Fast], min_priority_fee.clone())?
        .into_iter()
        .map(|x| {
            let priority_fee = BigInt::max(min_priority_fee.clone(), x.value);
            FeeRate::new(x.priority, GasPriceType::eip1559(base_fee.clone(), priority_fee))
        })
        .collect())
}

pub fn get_transaction_params(chain: EVMChain, input: &TransactionLoadInput) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    map_evm_transaction_params(chain, input)
}

pub fn map_evm_transaction_params(_chain: EVMChain, input: &TransactionLoadInput) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    let value = BigInt::from_str_radix(&input.value, 10)?;

    match &input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => match asset.id.token_subtype() {
            AssetSubtype::NATIVE => Ok(TransactionParams::new(input.destination_address.clone(), vec![], value)),
            AssetSubtype::TOKEN => {
                let to = asset.token_id.as_ref().ok_or("Missing token ID")?.clone();
                let value = BigInt::from_str_radix(&input.value, 10)?;
                let data = encode_erc20_transfer(&input.destination_address, &value)?;
                Ok(TransactionParams::new(to, data, BigInt::from(0)))
            }
        },
        TransactionInputType::TransferNft(_, nft_asset) => {
            let contract_address = nft_asset.contract_address.as_ref().ok_or("Missing contract address")?;
            let data = match nft_asset.token_type {
                NFTType::ERC721 => encode_erc721_transfer(&input.sender_address, &input.destination_address, &nft_asset.token_id)?,
                NFTType::ERC1155 => encode_erc1155_transfer(&input.sender_address, &input.destination_address, &nft_asset.token_id)?,
                _ => return Err("Unsupported NFT type for EVM".into()),
            };
            Ok(TransactionParams::new(contract_address.clone(), data, BigInt::from(0)))
        }
        TransactionInputType::Swap(from_asset, _, swap_data) => {
            if let Some(approval) = &swap_data.data.approval {
                Ok(TransactionParams::new(
                    approval.token.clone(),
                    encode_erc20_approve_max_value(&approval.spender)?,
                    BigInt::from(0),
                ))
            } else {
                match from_asset.id.token_subtype() {
                    AssetSubtype::NATIVE => {
                        let value = match swap_data.data.data_type {
                            SwapQuoteDataType::Transfer if input.is_max_value => BigInt::ZERO,
                            _ => BigInt::from_str_radix(&swap_data.data.value, 10)?,
                        };
                        Ok(TransactionParams::new(swap_data.data.to.clone(), hex::decode(swap_data.data.data.clone())?, value))
                    }
                    AssetSubtype::TOKEN => match swap_data.data.data_type {
                        SwapQuoteDataType::Contract => Ok(TransactionParams::new(swap_data.data.to.clone(), hex::decode(swap_data.data.data.clone())?, BigInt::ZERO)),
                        SwapQuoteDataType::Transfer => {
                            let to = from_asset.token_id.clone().ok_or("Missing token ID")?.clone();
                            let data = encode_erc20_transfer(&swap_data.data.to.clone(), &BigInt::from_str_radix(&input.value, 10)?)?;
                            Ok(TransactionParams::new(to, data, BigInt::ZERO))
                        }
                    },
                }
            }
        }
        TransactionInputType::TokenApprove(_, approval) => Ok(TransactionParams::new(
            approval.token.clone(),
            encode_erc20_approve_max_value(&approval.spender)?,
            BigInt::from(0),
        )),
        TransactionInputType::Generic(_, _, extra) => Ok(TransactionParams::new(
            extra.to.clone(),
            extra.data.clone().unwrap_or_default(),
            BigInt::from_str_radix(&input.value, 10)?,
        )),
        TransactionInputType::Earn(_, _, earn_data) => {
            if let Some(approval) = &earn_data.approval {
                Ok(TransactionParams::new_approval(approval.token.clone(), encode_erc20_approve_max_value(&approval.spender)?))
            } else {
                Ok(TransactionParams::new(
                    earn_data.contract_address.clone(),
                    decode_hex(&earn_data.call_data)?,
                    BigInt::from(0),
                ))
            }
        }
        _ => Err("Unsupported transfer type".into()),
    }
}

pub fn calculate_gas_limit_with_increase(gas_limit: BigInt) -> BigInt {
    if gas_limit == BigInt::from(TRANSFER_GAS_LIMIT) {
        gas_limit
    } else {
        gas_limit * BigInt::from(100 + GAS_LIMIT_PERCENT_INCREASE) / BigInt::from(100)
    }
}

pub fn get_extra_fee_gas_limit(input: &TransactionLoadInput) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    match &input.input_type {
        TransactionInputType::Swap(_, _, swap_data) => {
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
        TransactionInputType::Earn(_, _, earn_data) => {
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

    fn create_test_fee_history_for_mapper() -> EthereumFeeHistory {
        EthereumFeeHistory {
            reward: vec![vec!["0xbebc200".to_string(), "0x11e1a300".to_string()]],
            base_fee_per_gas: vec![BigInt::from(20_000_000_000u64)],
            gas_used_ratio: vec![0.5],
            oldest_block: 0x1234,
        }
    }

    #[test]
    fn test_map_transaction_fee_rates_normal_case() -> Result<(), Box<dyn Error + Sync + Send>> {
        let fee_history = create_test_fee_history_for_mapper();

        let result = map_transaction_fee_rates(EVMChain::Ethereum, &fee_history)?;

        assert_eq!(result.len(), 2);

        let min_priority_fee = BigInt::from(EVMChain::Ethereum.min_priority_fee());
        for fee_rate in &result {
            match &fee_rate.gas_price_type {
                GasPriceType::Eip1559 { gas_price, priority_fee } => {
                    assert!(*gas_price >= min_priority_fee);
                    assert!(*priority_fee >= min_priority_fee);
                }
                _ => panic!("Expected EIP-1559 gas price type"),
            }
        }

        Ok(())
    }

    #[test]
    fn test_map_transaction_fee_rates_zero_base_fee() -> Result<(), Box<dyn Error + Sync + Send>> {
        let fee_history = EthereumFeeHistory {
            reward: vec![vec!["0xbebc200".to_string(), "0x11e1a300".to_string()]],
            base_fee_per_gas: vec![BigInt::from(0u64)], // Zero base fee
            gas_used_ratio: vec![0.5],
            oldest_block: 0x1234,
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
            base_fee_per_gas: vec![BigInt::from(20_000_000_000u64)],
            gas_used_ratio: vec![0.5],
            oldest_block: 0x1234,
        };

        let result = map_transaction_fee_rates(EVMChain::Ethereum, &fee_history);
        assert!(result.is_err());
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

        let min_priority = BigInt::from(primitives::EVMChain::Ethereum.min_priority_fee());
        assert_eq!(min_priority.to_string(), "100000000");
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
