use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::future::join_all;
use std::error::Error;

use gem_client::Client;
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Chain, asset_balance::BalanceMetadata};

use crate::{
    address::TronAddress,
    models::TronRpcError,
    provider::balances_mapper::{map_balance_staking, map_coin_balance, map_token_balance},
    rpc::{TronProvider, trongrid::mapper::TronGridMapper},
};

#[async_trait]
impl<C: Client> ChainBalances for TronProvider<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        map_coin_balance(&account)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let parameter = TronAddress::parse(&address)?.abi_address_parameter();

        let futures: Vec<_> = token_ids
            .into_iter()
            .map(|token_id| {
                let parameter = parameter.clone();
                async move {
                    let balance_hex = match self.trigger_constant_contract(&token_id, "balanceOf(address)", &parameter).await {
                        Ok(balance_hex) => balance_hex,
                        Err(error) if error.downcast_ref::<TronRpcError>().is_some() => return Ok(None),
                        Err(error) => return Err(error),
                    };
                    let asset_id = AssetId::from(self.get_chain(), Some(token_id));
                    map_token_balance(&balance_hex, asset_id).map(Some)
                }
            })
            .collect();
        Ok(join_all(futures).await.into_iter().collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect())
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        if let Some(address) = &account.address {
            let (reward, usage) = futures::try_join!(self.get_reward(address), self.get_account_usage(address))?;
            Ok(Some(map_balance_staking(&account, &reward, &usage)?))
        } else {
            Ok(Some(AssetBalance::new_staking_with_metadata(
                AssetId::from_chain(Chain::Tron),
                BigUint::from(0u32),
                BigUint::from(0u32),
                BigUint::from(0u32),
                BalanceMetadata::default(),
            )))
        }
    }

    async fn get_balance_assets(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_indexer_accounts(&address).await?.into_iter().next().map(TronGridMapper::map_asset_balances).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::TronClient;
    use gem_client::testkit::MockClient;
    use primitives::asset_constants::TRON_USDT_TOKEN_ID;

    #[tokio::test]
    async fn test_get_balance_tokens_skips_reverted_contract() {
        let reverted = "TKzxdSv2FZKQrEqkKVgp5DcwEXBEKMg2Ax";
        let client = MockClient::new().with_post(move |_, body| match String::from_utf8_lossy(body).contains(reverted) {
            true => Ok(include_bytes!("../../testdata/trigger_constant_contract_reverted.json").to_vec()),
            false => Ok(include_bytes!("../../testdata/balance_token.json").to_vec()),
        });
        let provider = TronProvider::new_rpc_only(TronClient::new(client));

        let balances = provider
            .get_balance_tokens("TFdTEn9dJuqh351y8fyJ3eMmghFsZNwakb".to_string(), vec![TRON_USDT_TOKEN_ID.to_string(), reverted.to_string()])
            .await
            .unwrap();

        assert_eq!(balances, vec![AssetBalance::new(AssetId::from(Chain::Tron, Some(TRON_USDT_TOKEN_ID.to_string())), BigUint::from(136389002_u64))]);
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_USDT_TOKEN_ID, create_test_client};
    use num_bigint::BigUint;
    use primitives::Chain;

    #[tokio::test]
    async fn test_tron_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;

        assert_eq!(balance.asset_id.chain, Chain::Tron);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available > BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let token_ids = vec![TEST_USDT_TOKEN_ID.to_string()];

        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids.clone()).await?;

        assert_eq!(balances.len(), token_ids.len());
        for (i, balance) in balances.iter().enumerate() {
            assert_eq!(balance.asset_id.chain, Chain::Tron);
            assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
            assert!(balance.balance.available > BigUint::from(0u32));
        }

        assert!(balances.first().unwrap().balance.available > BigUint::from(0u32), "USDT balance should be greater than 0");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let balance = client.get_balance_staking(TEST_ADDRESS.to_string()).await?;

        let balance = balance.ok_or("Staking balance not found")?;

        assert_eq!(balance.asset_id.chain, Chain::Tron);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.staked > BigUint::from(0u32));

        let metadata = balance.balance.metadata.as_ref().ok_or("Metadata not found")?;

        assert!(metadata.bandwidth_available > 0);
        assert!(metadata.bandwidth_total >= 600);

        assert!(metadata.bandwidth_available <= metadata.bandwidth_total);
        assert!(metadata.energy_available <= metadata.energy_total);

        Ok(())
    }

    #[tokio::test]
    async fn test_tron_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        assert!(!assets.is_empty(), "TRON test address should have TRC20 tokens");

        for asset in &assets {
            assert_eq!(asset.asset_id.chain, Chain::Tron);
            assert!(asset.balance.available > BigUint::from(0u32));
            assert!(asset.asset_id.token_id.is_some());
        }
        Ok(())
    }
}
