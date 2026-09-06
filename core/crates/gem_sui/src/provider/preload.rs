use std::{collections::HashMap, error::Error};

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::{ChainTransactionLoad, TransactionFeeOperation};
use num_bigint::BigInt;
use primitives::{
    AssetId, Chain, FeeRate, GasPriceType, StakeType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput, TransferDataExtra,
};

use crate::{
    constants::{ESTIMATION_GAS_BUDGET, SWAP_GAS_UNITS, TOKEN_TRANSFER_GAS_UNITS, TRANSFER_GAS_UNITS},
    gas_budget::GAS_BUDGET_MULTIPLIER,
    models::{Coin, OwnedCoins, SuiObject},
};
use crate::{
    provider::preload_mapper::{map_transaction_data, map_transaction_rate_rates},
    rpc::{SuiClient, SuiProvider},
    tx_builder::{finish_transaction_json, is_transaction_json},
};

#[cfg(feature = "rpc")]
#[async_trait]
impl ChainTransactionLoad for SuiProvider {
    fn transaction_fee_estimate_units(&self, operation: TransactionFeeOperation) -> Option<u64> {
        Some(match operation {
            TransactionFeeOperation::Transfer => TRANSFER_GAS_UNITS,
            TransactionFeeOperation::TokenTransfer => TOKEN_TRANSFER_GAS_UNITS,
            TransactionFeeOperation::Swap => SWAP_GAS_UNITS,
        })
    }

    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let input = self.finish_transaction_json_input(input).await?;
        let (sui_coins, token_coins, objects) = self.get_coins_for_input_type(input.sender_address.as_str(), input.input_type.clone()).await?;

        let estimate_bytes = map_transaction_data(input.clone(), sui_coins.clone(), token_coins.clone(), objects.clone(), ESTIMATION_GAS_BUDGET)?;
        let fee = self.estimate_fee(&estimate_bytes, &input.gas_price).await?;

        let message_bytes = match estimated_gas_budget(&input.input_type, &fee)? {
            Some(budget) => map_transaction_data(input, sui_coins, token_coins, objects, budget)?,
            None => estimate_bytes,
        };

        Ok(TransactionLoadData {
            fee,
            metadata: TransactionLoadMetadata::Sui { message_bytes },
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        Ok(map_transaction_rate_rates(gas_price))
    }
}

fn estimated_gas_budget(input_type: &TransactionInputType, fee: &TransactionFee) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
    match input_type {
        TransactionInputType::Swap { .. } | TransactionInputType::Generic { .. } => Ok(None),
        _ => Ok(Some(fee.gas_limit()?)),
    }
}

impl SuiClient {
    async fn finish_transaction_json_input(&self, input: TransactionLoadInput) -> Result<TransactionLoadInput, Box<dyn Error + Send + Sync>> {
        let TransactionLoadInput {
            sender_address,
            destination_address,
            value,
            input_type,
            gas_price,
            memo,
            is_max_value,
            metadata,
        } = input;

        let input_type = match input_type {
            TransactionInputType::Generic {
                asset,
                metadata: app_metadata,
                extra,
            } if extra.data.as_deref().is_some_and(is_transaction_json) => TransactionInputType::Generic {
                asset,
                metadata: app_metadata,
                extra: self.finish_transaction_json_extra(&sender_address, extra).await?,
            },
            other => other,
        };

        Ok(TransactionLoadInput {
            sender_address,
            destination_address,
            value,
            input_type,
            gas_price,
            memo,
            is_max_value,
            metadata,
        })
    }

    async fn finish_transaction_json_extra(&self, sender_address: &str, extra: TransferDataExtra) -> Result<TransferDataExtra, Box<dyn Error + Send + Sync>> {
        let transaction_json = String::from_utf8(extra.data.clone().unwrap_or_default()).map_err(|_| "Invalid UTF-8 data for Sui generic input")?;
        let output = finish_transaction_json(self, &transaction_json, sender_address).await?;
        Ok(TransferDataExtra {
            data: Some(output.base64_encoded().into_bytes()),
            ..extra
        })
    }

    async fn estimate_fee(&self, tx_data: &str, gas_price: &GasPriceType) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let tx_data_only = tx_data.split('_').next().unwrap_or(tx_data);
        let result = self.dry_run(tx_data_only.to_string()).await?;
        let fee = result.effects.gas_used.calculate_gas_budget()?;
        let gas_limit = fee * GAS_BUDGET_MULTIPLIER / 100;

        Ok(TransactionFee {
            fee: BigInt::from(fee),
            gas_price_type: gas_price.clone(),
            gas_limit: BigInt::from(gas_limit),
            options: HashMap::new(),
            fee_asset: AssetId::from_chain(Chain::Sui),
        })
    }

    async fn get_coins_for_input_type(
        &self,
        address: &str,
        input_type: TransactionInputType,
    ) -> Result<(OwnedCoins<Coin>, Option<OwnedCoins<Coin>>, Vec<SuiObject>), Box<dyn Error + Send + Sync>> {
        match input_type {
            TransactionInputType::Transfer { asset } | TransactionInputType::Withdrawal { asset } => match asset.id.token_id {
                None => Ok((self.get_gas_coins(address).await?, None, Vec::new())),
                Some(token_id) => {
                    let (gas_coins, token_coins) = futures::try_join!(self.get_gas_coins(address), self.get_coins(address, &token_id))?;
                    Ok((gas_coins, Some(token_coins), Vec::new()))
                }
            },
            TransactionInputType::Stake { stake_type, .. } => match stake_type {
                StakeType::Stake(_) => Ok((self.get_gas_coins(address).await?, None, Vec::new())),
                StakeType::Unstake(delegation) => {
                    let (gas_coins, staked_object) = futures::try_join!(self.get_gas_coins(address), self.get_object(delegation.base.delegation_id.clone()))?;
                    Ok((gas_coins, None, vec![staked_object]))
                }
                StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => {
                    Err("Unsupported stake type for Sui".into())
                }
            },
            TransactionInputType::Swap { .. } => Ok((OwnedCoins::default(), None, Vec::new())),
            TransactionInputType::Generic { .. } => Ok((OwnedCoins::default(), None, Vec::new())),
            TransactionInputType::TransferNft { .. } | TransactionInputType::Account { .. } => Err("Unsupported transaction type for Sui".into()),
            _ => Err("Unsupported transaction type for Sui".into()),
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::models::SuiStakeStatus;
    use crate::provider::testkit::*;
    use chain_traits::ChainTransactionLoad;
    use gem_encoding::decode_base64;
    use num_bigint::BigUint;
    use primitives::{Asset, Chain, Delegation, FeePriority, StakeType, TransactionLoadInput};

    #[tokio::test]
    async fn test_sui_get_transaction_fee_rates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let rates = client
            .get_transaction_fee_rates(TransactionInputType::Transfer {
                asset: Asset::from_chain(Chain::Sui),
            })
            .await?;

        println!("Sui transaction fee rates: {:?}", rates);

        assert_eq!(rates.len(), 2);
        assert_eq!(rates[0].priority, FeePriority::Normal);
        assert_eq!(rates[1].priority, FeePriority::Fast);

        Ok(())
    }

    #[tokio::test]
    async fn test_sui_get_transaction_preload() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let input = TransactionPreloadInput {
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
            input_type: TransactionInputType::Transfer {
                asset: Asset::from_chain(Chain::Sui),
            },
            references: vec![],
        };

        let _metadata = client.get_transaction_preload(input).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_sui_get_transaction_preload_unstake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();

        let user_address = TEST_ADDRESS;
        let delegation_id = client
            .get_stake_delegations(user_address.to_string())
            .await?
            .into_iter()
            .flat_map(|delegation| delegation.stakes.into_iter())
            .find(|stake| stake.status == SuiStakeStatus::Active)
            .ok_or("No active Sui stake found for test address")?
            .staked_sui_id;

        let delegation = Delegation::mock_with_id(delegation_id);
        let stake_type = StakeType::Unstake(delegation);

        let input = TransactionLoadInput {
            sender_address: user_address.to_string(),
            destination_address: user_address.to_string(),
            value: BigUint::from(1000000000u64),
            input_type: TransactionInputType::Stake {
                asset: Asset::from_chain(Chain::Sui),
                stake_type,
            },
            gas_price: primitives::GasPriceType::regular(BigInt::from(1000)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        };

        let result = client.get_transaction_load(input).await?;

        match result.metadata {
            TransactionLoadMetadata::Sui { message_bytes } => {
                assert!(!message_bytes.is_empty());
                println!("Sui unstake transaction data: {} chars", message_bytes.len());

                assert!(message_bytes.contains('_'));
                let parts: Vec<&str> = message_bytes.split('_').collect();
                assert_eq!(parts.len(), 2);

                assert!(decode_base64(parts[0]).is_ok());
                assert!(hex::decode(parts[1]).is_ok());
            }
            _ => panic!("Expected Sui metadata for unstake transaction"),
        }

        assert!(result.fee.fee > BigInt::from(0));
        assert!(result.fee.gas_limit > BigInt::from(0));
        assert!(result.fee.gas_limit >= result.fee.fee);

        println!("Unstake transaction fee: {}", result.fee.fee);

        Ok(())
    }
}
