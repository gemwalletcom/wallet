use std::collections::HashMap;
use std::error::Error;

use num_bigint::BigInt;
use primitives::{
    AssetId, GasPriceType, StakeType, TransactionFee, TransactionInputType,
    chain_cosmos::CosmosChain,
    swap::{SwapData, SwapQuoteDataType},
};

use crate::constants::TRANSFER_GAS_LIMIT;

const CONTRACT_SWAP_MESSAGE_GAS_LIMIT: u64 = 2_000_000;
const STAKE_MESSAGE_GAS_LIMIT: u64 = 1_000_000;
const REDELEGATE_MESSAGE_GAS_LIMIT: u64 = 1_250_000;
const REWARDS_MESSAGE_GAS_LIMIT: u64 = 750_000;
const CLAIM_REWARDS_AND_STAKE_MESSAGES: u64 = 2;
const PROVIDER_GAS_LIMIT_BUFFER_NUMERATOR: u64 = 13;
const PROVIDER_GAS_LIMIT_BUFFER_DENOMINATOR: u64 = 10;

fn get_gas_limit(input_type: &TransactionInputType) -> Result<u64, Box<dyn Error + Send + Sync>> {
    Ok(match input_type {
        TransactionInputType::Transfer { .. }
        | TransactionInputType::Withdrawal { .. }
        | TransactionInputType::Deposit { .. }
        | TransactionInputType::TransferNft { .. }
        | TransactionInputType::Account { .. }
        | TransactionInputType::TokenApprove { .. }
        | TransactionInputType::Generic { .. }
        | TransactionInputType::Perpetual { .. }
        | TransactionInputType::Earn { .. } => TRANSFER_GAS_LIMIT,
        TransactionInputType::Swap { swap_data, .. } => get_swap_gas_limit(swap_data)?,
        TransactionInputType::Stake { stake_type, .. } => match stake_type {
            StakeType::Stake(_) => STAKE_MESSAGE_GAS_LIMIT,
            StakeType::Unstake(_) => STAKE_MESSAGE_GAS_LIMIT * CLAIM_REWARDS_AND_STAKE_MESSAGES,
            StakeType::Redelegate(_) => REDELEGATE_MESSAGE_GAS_LIMIT * CLAIM_REWARDS_AND_STAKE_MESSAGES,
            StakeType::Rewards(validators) => REWARDS_MESSAGE_GAS_LIMIT * u64::try_from(validators.len())?,
            StakeType::Withdraw(_) => REWARDS_MESSAGE_GAS_LIMIT,
            StakeType::Freeze(_) | StakeType::Unfreeze(_) => return Err("Cosmos freeze operations are not supported".into()),
        },
    })
}

fn get_swap_gas_limit(swap_data: &SwapData) -> Result<u64, Box<dyn Error + Send + Sync>> {
    match swap_data.data.data_type {
        SwapQuoteDataType::Transfer => Ok(TRANSFER_GAS_LIMIT),
        SwapQuoteDataType::Contract => {
            let provider_gas_limit = swap_data
                .data
                .gas_limit
                .as_deref()
                .and_then(|gas_limit| gas_limit.parse::<u64>().ok())
                .filter(|gas_limit| *gas_limit > 0);
            match provider_gas_limit {
                Some(gas_limit) => Ok(gas_limit.checked_mul(PROVIDER_GAS_LIMIT_BUFFER_NUMERATOR).ok_or("gas limit overflow")? / PROVIDER_GAS_LIMIT_BUFFER_DENOMINATOR),
                None => {
                    let messages: Vec<serde_json::Value> = serde_json::from_str(&swap_data.data.data)?;
                    Ok(CONTRACT_SWAP_MESSAGE_GAS_LIMIT * u64::try_from(messages.len())?)
                }
            }
        }
    }
}

pub fn calculate_transaction_fee(input_type: &TransactionInputType, chain: CosmosChain, gas_price_type: &GasPriceType) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
    let gas_limit = get_gas_limit(input_type)?;
    let fee = match chain {
        CosmosChain::Thorchain | CosmosChain::Mayachain => gas_price_type.gas_price(),
        CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Injective | CosmosChain::Sei | CosmosChain::Noble => {
            let transfer_gas_limit = BigInt::from(TRANSFER_GAS_LIMIT);
            (gas_price_type.gas_price() * gas_limit + &transfer_gas_limit - 1) / transfer_gas_limit
        }
    };

    Ok(TransactionFee {
        fee,
        gas_price_type: gas_price_type.clone(),
        gas_limit: BigInt::from(gas_limit),
        options: HashMap::new(),
        fee_asset: AssetId::from_chain(chain.as_chain()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, Chain, DelegationValidator, Resource, SwapProvider, swap::SwapQuoteData};

    #[test]
    fn test_calculate_transaction_fee() {
        let atom = Asset::from_chain(Chain::Cosmos);
        let transaction_fee = |chain: CosmosChain, input_type: TransactionInputType, base_fee: u64| {
            let fee = calculate_transaction_fee(&input_type, chain, &GasPriceType::regular(base_fee)).unwrap();
            (fee.gas_limit, fee.fee)
        };

        assert_eq!(
            transaction_fee(CosmosChain::Cosmos, TransactionInputType::Transfer { asset: atom.clone() }, 1_000),
            (BigInt::from(200_000), BigInt::from(1_000))
        );
        assert_eq!(
            transaction_fee(
                CosmosChain::Cosmos,
                TransactionInputType::Swap {
                    from_asset: atom.clone(),
                    to_asset: atom.clone(),
                    swap_data: SwapData {
                        data: SwapQuoteData {
                            data_type: SwapQuoteDataType::Transfer,
                            gas_limit: None,
                            ..SwapQuoteData::mock()
                        },
                        ..SwapData::mock_with_provider(SwapProvider::SwapsXyz)
                    },
                },
                1_000,
            ),
            (BigInt::from(200_000), BigInt::from(1_000))
        );
        assert_eq!(
            transaction_fee(
                CosmosChain::Cosmos,
                TransactionInputType::Swap {
                    from_asset: atom.clone(),
                    to_asset: atom.clone(),
                    swap_data: SwapData::mock_with_provider_data(SwapProvider::Squid, "[{}, {}]", None),
                },
                1_000,
            ),
            (BigInt::from(4_000_000), BigInt::from(20_000))
        );
        assert_eq!(
            transaction_fee(
                CosmosChain::Cosmos,
                TransactionInputType::Swap {
                    from_asset: atom.clone(),
                    to_asset: atom.clone(),
                    swap_data: SwapData::mock_with_provider_data(SwapProvider::Squid, "[{}]", Some("250001")),
                },
                1_000,
            ),
            (BigInt::from(325_001), BigInt::from(1_626))
        );
        assert_eq!(
            transaction_fee(
                CosmosChain::Cosmos,
                TransactionInputType::Stake {
                    asset: atom.clone(),
                    stake_type: StakeType::Rewards(vec![DelegationValidator::mock_osmosis("cosmosvaloper1"); 7]),
                },
                1_000,
            ),
            (BigInt::from(5_250_000), BigInt::from(26_250))
        );
        assert_eq!(
            transaction_fee(
                CosmosChain::Thorchain,
                TransactionInputType::Transfer {
                    asset: Asset::from_chain(Chain::Thorchain),
                },
                2_000_000,
            ),
            (BigInt::from(200_000), BigInt::from(2_000_000))
        );
    }

    #[test]
    fn calculate_transaction_fee_rejects_freeze_without_panicking() {
        let input_type = TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::Cosmos),
            stake_type: StakeType::Freeze(Resource::Bandwidth),
        };

        assert_eq!(
            calculate_transaction_fee(&input_type, CosmosChain::Cosmos, &GasPriceType::regular(1u64))
                .unwrap_err()
                .to_string(),
            "Cosmos freeze operations are not supported"
        );
    }
}
