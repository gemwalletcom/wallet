use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainTransactionLoad, TransactionFeeOperation};
use futures::try_join;
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{FeeRate, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::{
    constants::{FUNGIBLE_TOKEN_FUNCTION_CALL_GAS, TRANSFER_GAS_UNITS},
    models::StorageBalanceBounds,
    provider::{
        preload_mapper::{address_to_public_key, map_transaction_fee, map_transaction_preload},
        state_mapper::map_gas_price_to_priorities,
    },
    rpc::NearProvider,
};

#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for NearProvider<C> {
    fn transaction_fee_estimate_units(&self, operation: TransactionFeeOperation) -> Option<u64> {
        Some(match operation {
            TransactionFeeOperation::Transfer | TransactionFeeOperation::Swap => TRANSFER_GAS_UNITS,
            TransactionFeeOperation::TokenTransfer => FUNGIBLE_TOKEN_FUNCTION_CALL_GAS,
        })
    }

    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let public_key = address_to_public_key(&input.sender_address)?;
        let (access_key, block) = try_join!(self.get_account_access_key(&input.sender_address, &public_key), self.get_latest_block(),)?;
        Ok(map_transaction_preload(&access_key, &block))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let protocol_config = self.get_protocol_config().await?;
        let destination_address = input.input_type.swap_to_address().unwrap_or(&input.destination_address);
        let token_account_creation_deposit = if let Some(token_id) = input.input_type.get_asset().id.token_id.as_deref() {
            let args = serde_json::json!({ "account_id": destination_address });
            let balance: Option<serde_json::Value> = self.call_function(token_id, "storage_balance_of", &args).await?;
            if balance.is_some() {
                None
            } else {
                let bounds: StorageBalanceBounds = self.call_function(token_id, "storage_balance_bounds", &serde_json::json!({})).await?;
                Some(BigInt::from(bounds.min.parse::<u128>()?))
            }
        } else {
            None
        };
        Ok(TransactionLoadData {
            fee: map_transaction_fee(&input, destination_address, &protocol_config, token_account_creation_deposit),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        map_gas_price_to_priorities(&gas_price)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_near_test_client};
    use primitives::{Asset, AssetType, Chain, FeeOption, GasPriceType, SwapProvider, TransactionInputType, asset_constants::NEAR_USDT_ASSET_ID, swap::SwapData};

    #[tokio::test]
    async fn test_near_token_swap_load_includes_registration() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_near_test_client();
        let deposit_address = "gemwallet-near-token-registration-test-20260812.near";
        let from_asset = Asset::new(NEAR_USDT_ASSET_ID.clone(), "Tether".to_string(), "USDT".to_string(), 6, AssetType::TOKEN);
        let input_type = TransactionInputType::Swap(
            from_asset,
            Asset::from_chain(Chain::Near),
            SwapData::mock_transfer(SwapProvider::NearIntents, "1000000", "1", deposit_address),
        );
        let metadata = client
            .get_transaction_preload(TransactionPreloadInput {
                input_type: input_type.clone(),
                sender_address: TEST_ADDRESS.to_string(),
                destination_address: TEST_ADDRESS.to_string(),
            })
            .await?;
        let load = client
            .get_transaction_load(TransactionLoadInput {
                input_type,
                sender_address: TEST_ADDRESS.to_string(),
                destination_address: TEST_ADDRESS.to_string(),
                value: "1000000".to_string(),
                gas_price: GasPriceType::regular(100_000_000u64),
                memo: None,
                is_max_value: false,
                metadata,
            })
            .await?;

        assert!(load.fee.options.get(&FeeOption::TokenAccountCreation).is_some_and(|deposit| deposit > &BigInt::ZERO));
        Ok(())
    }
}
