use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::types::JsonRpcError;
use primitives::AssetBalance;

use crate::{
    provider::balances_mapper::{map_balance_assets, map_balance_coin, map_balance_tokens},
    rpc::XrpClient,
};

const ACCOUNT_NOT_FOUND_ERROR_CODE: i32 = 19;

impl<C: Client + Clone> XrpClient<C> {
    fn default_if_account_not_found<T: Default>(result: Result<T, Box<dyn Error + Send + Sync>>) -> Result<T, Box<dyn Error + Send + Sync>> {
        match result {
            Ok(value) => Ok(value),
            Err(error) if Self::is_account_not_found(error.as_ref()) => Ok(T::default()),
            Err(error) => Err(error),
        }
    }

    fn is_account_not_found(error: &(dyn Error + Send + Sync + 'static)) -> bool {
        error.downcast_ref::<JsonRpcError>().is_some_and(|error| error.code == ACCOUNT_NOT_FOUND_ERROR_CODE)
    }
}

#[async_trait]
impl<C: Client + Clone> ChainBalances for XrpClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = Self::default_if_account_not_found(self.get_account_info(&address).await)?;
        let reserved_amount = self.get_chain().account_activation_fee().unwrap_or(0) as u64;

        map_balance_coin(account, self.get_chain().as_asset_id(), reserved_amount)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let objects = Self::default_if_account_not_found(self.get_account_objects(&address).await)?;
        Ok(map_balance_tokens(&objects, token_ids, self.get_chain()))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }

    async fn get_balance_assets(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let objects = Self::default_if_account_not_found(self.get_account_objects(&address).await)?;
        Ok(map_balance_assets(&objects, self.get_chain()))
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use num_bigint::BigUint;
    use primitives::{AssetId, Chain};
    use serde_json::json;

    use super::*;
    use crate::method;

    #[tokio::test]
    async fn test_account_not_found_balances() {
        let client = XrpClient::new(mock_jsonrpc_client(|rpc_method, _| {
            let error_message = match rpc_method {
                method::ACCOUNT_INFO => "Account not found.",
                method::ACCOUNT_OBJECTS => "accountNotFound",
                _ => panic!("unexpected method: {rpc_method}"),
            };
            Ok(json!({
                "error": "actNotFound",
                "error_code": ACCOUNT_NOT_FOUND_ERROR_CODE,
                "error_message": error_message,
                "status": "error"
            }))
        }));
        let token_id = "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De";

        let coin_balance = client.get_balance_coin("rMissing".to_string()).await.unwrap();
        let token_balances = client.get_balance_tokens("rMissing".to_string(), vec![token_id.to_string()]).await.unwrap();
        let asset_balances = client.get_balance_assets("rMissing".to_string()).await.unwrap();

        assert_eq!(coin_balance.balance.available, BigUint::ZERO);
        assert_eq!(token_balances.len(), 1);
        assert_eq!(token_balances[0].asset_id, AssetId::from_token(Chain::Xrp, token_id));
        assert_eq!(token_balances[0].balance.available, BigUint::ZERO);
        assert_eq!(asset_balances, vec![]);
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use primitives::Chain;

    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_ADDRESS_EMPTY, create_xrp_test_client};

    #[tokio::test]
    async fn test_xrp_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
        println!("Balance: {:?} {}", balance.balance.available, balance.asset_id);
        Ok(())
    }

    #[tokio::test]
    async fn test_xrp_get_balance_coin_empty_account() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS_EMPTY.to_string()).await?;
        assert!(balance.balance.available == num_bigint::BigUint::from(0u32));
        Ok(())
    }

    #[tokio::test]
    async fn test_xrp_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let token_ids = vec![
            "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De".to_string(), // RLUSD
        ];
        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids).await?;

        assert_eq!(balances.len(), 1);
        for balance in &balances {
            assert_eq!(balance.asset_id.chain, Chain::Xrp);
            assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_xrp_get_balance_tokens_empty_account() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let token_ids = vec![
            "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De".to_string(), // RLUSD
        ];
        let balances = client.get_balance_tokens(TEST_ADDRESS_EMPTY.to_string(), token_ids).await?;

        assert_eq!(balances.len(), 1);

        for balance in &balances {
            assert_eq!(balance.asset_id.chain, Chain::Xrp);
            assert!(balance.balance.available == num_bigint::BigUint::from(0u32));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_xrp_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        println!("Assets: {}", assets.len());

        assert!(!assets.is_empty());

        for asset in assets {
            assert_eq!(asset.asset_id.chain, Chain::Xrp);
        }
        Ok(())
    }
}
