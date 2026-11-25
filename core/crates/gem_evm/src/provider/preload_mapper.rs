use std::error::Error;

use alloy_primitives::hex;
use num_bigint::BigInt;
use num_traits::Num;
use primitives::{EVMChain, FeeRate, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, fee::FeePriority, fee::GasPriceType};

use crate::constants::TRANSFER_GAS_LIMIT;
use crate::fee_calculator::FeeCalculator;
use crate::models::fee::EthereumFeeHistory;

const GAS_LIMIT_PERCENT_INCREASE: u32 = 50;

pub fn bigint_to_hex_string(value: &BigInt) -> String {
    format!("0x{:x}", value)
}

pub fn bytes_to_hex_string(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

pub fn map_transaction_preload(nonce: u64, chain_id: String) -> Result<TransactionLoadMetadata, Box<dyn std::error::Error + Send + Sync>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{encode_erc721_transfer, encode_erc1155_transfer};
    use primitives::Asset;
    use primitives::swap::{ApprovalData, Permit2ApprovalData, SwapData, SwapQuoteData};

    #[test]
    fn test_map_transaction_preload() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let chain_id = "1".to_string();

        let result = map_transaction_preload(10, chain_id)?;

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
    fn test_map_transaction_preload_invalid_chain_id() {
        let chain_id_hex = "invalid".to_string();

        let result = map_transaction_preload(1, chain_id_hex);

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
