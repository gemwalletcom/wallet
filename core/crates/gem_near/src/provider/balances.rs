use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::future::try_join_all;
use gem_client::Client;
use gem_jsonrpc::types::JsonRpcError;
use primitives::{AssetBalance, Chain};

use super::balances_mapper::map_native_balance;
use super::token_mapper::map_token_balance;
use crate::rpc::NearProvider;

const ACCOUNT_NOT_FOUND_ERROR_CODE: i32 = -32000;

#[async_trait]
impl<C: Client + Clone> ChainBalances for NearProvider<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = match self.get_account(&address).await {
            Ok(account) => account,
            Err(error) if is_account_missing(&error) => return Ok(AssetBalance::new_zero_balance(Chain::Near.as_asset_id())),
            Err(error) => return Err(error.into()),
        };
        Ok(map_native_balance(&account))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balances = try_join_all(token_ids.iter().map(|token_id| async {
            let value: String = self.call_function(token_id, "ft_balance_of", &serde_json::json!({ "account_id": address })).await?;
            map_token_balance(token_id, &value)
        }))
        .await?;
        Ok(balances)
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }

    async fn get_balance_assets(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

fn is_account_missing(error: &JsonRpcError) -> bool {
    error.code == ACCOUNT_NOT_FOUND_ERROR_CODE
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, create_near_test_client};
    use chain_traits::ChainBalances;

    #[tokio::test]
    async fn test_near_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_near_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
        println!("Balance: {} {}", balance.balance.available, balance.asset_id);
        Ok(())
    }

    #[tokio::test]
    async fn test_near_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_near_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        assert_eq!(assets.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_near_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use primitives::asset_constants::NEAR_USDT_TOKEN_ID;

        let client = create_near_test_client();
        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), vec![NEAR_USDT_TOKEN_ID.to_string()]).await?;

        assert_eq!(balances.len(), 1);
        assert_eq!(balances[0].asset_id.token_id.as_deref(), Some(NEAR_USDT_TOKEN_ID));
        Ok(())
    }
}
