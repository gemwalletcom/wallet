use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::try_join;
use std::error::Error;

use gem_client::Client;
use primitives::{Asset, AssetBalance};

use super::balances_mapper::{map_balance_assets, map_balance_staking, map_balance_token, map_balance_tokens};
use crate::rpc::client::HyperCoreClient;

const NATIVE_TOKEN_INDEX: u32 = 150;

#[async_trait]
impl<C: Client> ChainBalances for HyperCoreClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balances = self.get_spot_balances(&address).await?;
        let native_decimals = Asset::from_chain(self.chain).decimals;
        let Some(balance) = balances.balances.iter().find(|balance| balance.token == NATIVE_TOKEN_INDEX) else {
            return Ok(AssetBalance::new_zero_balance(self.chain.as_asset_id()));
        };

        map_balance_token(self.chain.as_asset_id(), balance, balances.available_after_maintenance(balance.token), native_decimals)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let (spot_balances, spot_meta) = try_join!(self.get_spot_balances(&address), self.get_spot_meta())?;
        Ok(map_balance_tokens(&spot_balances, &spot_meta.tokens, &token_ids, self.chain))
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balance = self.get_stake_balance(&address).await?;
        Ok(Some(map_balance_staking(&balance, self.chain)?))
    }

    async fn get_balance_assets(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let (spot_balances, spot_meta) = try_join!(self.get_spot_balances(&address), self.get_spot_meta())?;
        Ok(map_balance_assets(&spot_balances, &spot_meta.tokens, self.chain))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, USDC_TOKEN_ID, create_hypercore_test_client};
    use chain_traits::ChainBalances;
    use num_bigint::BigUint;

    #[tokio::test]
    async fn test_hypercore_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;

        println!("Hypercore coin balance: {:?} {}", balance.balance.available, balance.asset_id);

        assert!(balance.balance.available >= BigUint::from(0u64));
        assert_eq!(balance.asset_id.chain, primitives::Chain::HyperCore);
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let address = TEST_ADDRESS.to_string();
        let token_balances = client.get_balance_tokens(address, vec![USDC_TOKEN_ID.to_string()]).await?;

        println!("Hypercore token balances: {:?}", token_balances);

        assert!(!token_balances.is_empty());
        assert_eq!(token_balances[0].asset_id.chain, primitives::Chain::HyperCore);
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_staking(address).await?.ok_or("not found")?;

        println!("Hypercore staking balance: {:?}", balance.balance.staked);

        assert!(balance.balance.staked >= BigUint::from(0u64));
        assert_eq!(balance.asset_id.chain, primitives::Chain::HyperCore);
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        println!("Hypercore asset balances: {:?}", assets);

        for asset in &assets {
            assert_eq!(asset.asset_id.chain, primitives::Chain::HyperCore);
            assert!(asset.asset_id.token_id.is_some());
        }
        Ok(())
    }
}
