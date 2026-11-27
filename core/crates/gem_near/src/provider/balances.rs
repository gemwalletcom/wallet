use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::future::try_join_all;
use gem_client::Client;
use primitives::{AssetBalance, Chain};

use super::balances_mapper::map_native_balance;
use super::token_mapper::map_token_balance;
use crate::rpc::NearProvider;

#[async_trait]
impl<C: Client + Clone> ChainBalances for NearProvider<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = match self.get_account(&address).await {
            Ok(account) => account,
            Err(error) if error.payload.is_unknown_account() => return Ok(AssetBalance::new_zero_balance(Chain::Near.as_asset_id())),
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
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use gem_jsonrpc::client::JsonRpcClient;

    use super::*;
    use crate::provider::testkit::TEST_ADDRESS;
    use crate::rpc::NearClient;

    #[tokio::test]
    async fn test_get_balance_coin_reads_zero_only_for_unknown_account() {
        for (response, expected) in [
            (include_str!("../../testdata/account_unknown.json"), Ok("0".to_string())),
            (include_str!("../../testdata/account_unavailable_shard.json"), Err("Server error (-32000)".to_string())),
        ] {
            let client = MockClient::new().with_post(move |_, _| Ok(response.as_bytes().to_vec()));
            let provider = NearProvider::new_rpc_only(NearClient::new(JsonRpcClient::new(client)));
            let balance = provider.get_balance_coin(TEST_ADDRESS.to_string()).await;

            assert_eq!(balance.map(|balance| balance.balance.available.to_string()).map_err(|error| error.to_string()), expected);
        }
    }
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

        assert!(assets.is_none());
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
