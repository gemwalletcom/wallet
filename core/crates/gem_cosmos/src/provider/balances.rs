use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::try_join;
use num_bigint::BigUint;
use std::error::Error;

use gem_client::Client;
use primitives::{AssetBalance, AssetId};

use crate::{provider::balances_mapper, rpc::client::CosmosClient};

#[async_trait]
impl<C: Client> ChainBalances for CosmosClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balances = self.get_balances(&address).await?;
        let chain = self.get_chain().as_chain();
        let denom = chain.as_denom().ok_or("Chain does not have a denom")?;

        if let Some(balance) = balances.iter().find(|balance| balance.denom == denom) {
            Ok(AssetBalance::new(chain.as_asset_id(), balance.amount.parse::<BigUint>().unwrap_or_default()))
        } else {
            Ok(AssetBalance::new_zero_balance(chain.as_asset_id()))
        }
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balances = self.get_balances(&address).await?;
        let token_balances = token_ids
            .iter()
            .map(|token_id| {
                let asset_id = AssetId {
                    chain: self.get_chain().as_chain(),
                    token_id: Some(token_id.clone()),
                };
                match balances.iter().find(|balance| balance.denom == *token_id) {
                    Some(balance) => AssetBalance::new(asset_id, balance.amount.parse::<BigUint>().unwrap_or_default()),
                    None => AssetBalance::new_zero_balance(asset_id),
                }
            })
            .collect();

        Ok(token_balances)
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let cosmos_chain = self.get_chain();
        let chain = cosmos_chain.as_chain();
        if !chain.is_stake_supported() {
            return Ok(None);
        }
        let denom = chain.as_denom().ok_or("Chain does not have a denom")?;

        let (delegations, unbonding, rewards) = try_join!(self.get_delegations(&address), self.get_unbonding_delegations(&address), self.get_delegation_rewards(&address))?;

        Ok(Some(balances_mapper::map_balance_staking(delegations, unbonding, rewards, chain, denom)))
    }

    async fn get_balance_assets(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;
    use primitives::chain_cosmos::CosmosChain;

    #[tokio::test]
    async fn test_a_denomination_that_disappeared_is_reported_as_zero() {
        let client = MockClient::new().with_get(|_| Ok(r#"{"balances":[{"denom":"still-here","amount":"7"}],"pagination":{"next_key":null}}"#.as_bytes().to_vec()));
        let client = CosmosClient::new(CosmosChain::Cosmos, client);

        let balances = client.get_balance_tokens("cosmos1".to_string(), vec!["still-here".to_string(), "spent".to_string()]).await.unwrap();

        assert_eq!(
            balances
                .iter()
                .map(|balance| (balance.asset_id.token_id.clone().unwrap_or_default(), balance.balance.available.to_string()))
                .collect::<Vec<_>>(),
            vec![("still-here".to_string(), "7".to_string()), ("spent".to_string(), "0".to_string())],
            "a requested denomination the node no longer lists is spent, not unchanged"
        );
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, TEST_EMPTY_ADDRESS, create_cosmos_test_client};
    use chain_traits::ChainBalances;
    use num_bigint::BigUint;

    #[tokio::test]
    async fn test_cosmos_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_cosmos_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;

        println!("Balance: {:?} {}", balance.balance.available, balance.asset_id);

        assert!(balance.balance.available > BigUint::from(0u64));
        Ok(())
    }

    #[tokio::test]
    async fn test_cosmos_get_balance_coin_empty_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_cosmos_test_client();
        let address = TEST_EMPTY_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;

        println!("Balance: {:?} {}", balance.balance.available, balance.asset_id);

        assert!(balance.balance.available == BigUint::from(0u64));
        Ok(())
    }

    #[tokio::test]
    async fn test_cosmos_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_cosmos_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        assert_eq!(assets.len(), 0);
        Ok(())
    }
}
