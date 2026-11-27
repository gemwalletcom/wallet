use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use crate::provider::balances_mapper::{map_balance_assets, map_balance_staking, map_coin_balance, map_token_accounts};
use crate::rpc::SolanaProvider;
use gem_client::Client;
use primitives::AssetBalance;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainBalances for SolanaProvider<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(&address).await?;
        Ok(map_coin_balance(&balance))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let results = self.get_token_accounts(&address, &token_ids).await?;
        let balances: Vec<AssetBalance> = results.iter().zip(&token_ids).flat_map(|(token_accounts, token_id)| map_token_accounts(token_accounts, token_id)).collect();

        Ok(balances)
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let accounts = self.get_staking_balance(&address).await?;
        Ok(map_balance_staking(accounts))
    }

    async fn get_balance_assets(&self, address: String) -> Result<Option<Vec<AssetBalance>>, Box<dyn Error + Send + Sync>> {
        let (token_accounts, token_2022_accounts) = futures::try_join!(self.get_token_accounts_by_owner(&address, crate::TOKEN_PROGRAM), self.get_token_accounts_by_owner(&address, crate::TOKEN_PROGRAM_2022))?;
        Ok(Some(map_balance_assets(token_accounts.value.into_iter().chain(token_2022_accounts.value).collect())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PYUSD_TOKEN_MINT, TOKEN_PROGRAM, TOKEN_PROGRAM_2022, rpc::SolanaClient};
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use num_bigint::BigUint;
    use primitives::{
        AssetId, Chain,
        testkit::{json_rpc::load_json_rpc_result, signer_mock::TEST_SOLANA_SENDER},
    };
    use serde_json::Value;

    #[tokio::test]
    async fn test_get_balance_assets_includes_token_2022_accounts() {
        let client = mock_jsonrpc_client(|_, params| {
            let response = match params[1]["programId"].as_str() {
                Some(TOKEN_PROGRAM) => include_str!("../../testdata/token_accounts_token_program.json"),
                Some(TOKEN_PROGRAM_2022) => include_str!("../../testdata/token_accounts_token_2022_program.json"),
                _ => unreachable!(),
            };
            Ok(load_json_rpc_result::<Value>(response))
        });
        let provider = SolanaProvider::new(SolanaClient::new(client));

        let balances = provider.get_balance_assets(TEST_SOLANA_SENDER.to_string()).await.unwrap().unwrap();

        assert_eq!(
            balances,
            vec![
                AssetBalance::new(AssetId::from_token(Chain::Solana, "9yMP3WUodc5RmuHSmNzFQagA58cjqWr8oQMXvPYnUHtJ"), BigUint::from(14_000_000_000_000u64)),
                AssetBalance::new(AssetId::from_token(Chain::Solana, PYUSD_TOKEN_MINT), BigUint::from(16_065_364_839u64)),
            ]
        );
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::{PYUSD_TOKEN_MINT, USDC_TOKEN_MINT, USDT_TOKEN_MINT, provider::testkit::create_solana_test_client};
    use primitives::{Chain, testkit::signer_mock::TEST_SOLANA_SENDER};

    #[tokio::test]
    async fn test_solana_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let balance = client.get_balance_coin(TEST_SOLANA_SENDER.to_string()).await?;

        assert_eq!(balance.asset_id.chain, Chain::Solana);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available >= num_bigint::BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let token_ids = vec![USDC_TOKEN_MINT.to_string(), USDT_TOKEN_MINT.to_string(), PYUSD_TOKEN_MINT.to_string()];

        let balances = client.get_balance_tokens(TEST_SOLANA_SENDER.to_string(), token_ids.clone()).await?;

        assert_eq!(balances.len(), token_ids.len());
        for (i, balance) in balances.iter().enumerate() {
            assert_eq!(balance.asset_id.chain, Chain::Solana);
            assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
            assert!(balance.balance.available >= num_bigint::BigUint::from(0u32));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let staking_balance = client.get_balance_staking(TEST_SOLANA_SENDER.to_string()).await?;

        if let Some(balance) = staking_balance {
            assert_eq!(balance.asset_id.chain, Chain::Solana);
            assert_eq!(balance.asset_id.token_id, None);
            assert!(balance.balance.staked > num_bigint::BigUint::from(0u32));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let address = TEST_SOLANA_SENDER.to_string();
        let assets = client.get_balance_assets(address).await?.unwrap();

        for asset in assets {
            assert_eq!(asset.asset_id.chain, Chain::Solana);
            assert!(asset.balance.available > num_bigint::BigUint::from(0u32));
        }
        Ok(())
    }
}
