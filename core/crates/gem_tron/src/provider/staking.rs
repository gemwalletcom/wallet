use async_trait::async_trait;
use chain_traits::ChainStaking;
use chrono::Utc;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use super::staking_mapper::{map_staking_delegations, map_staking_validators};
use crate::rpc::TronProvider;
use crate::rpc::constants::{GET_WITNESS_127_PAY_PER_BLOCK, GET_WITNESS_PAY_PER_BLOCK};

#[async_trait]
impl<C: Client> ChainStaking for TronProvider<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let params = self.get_chain_parameters().await?;
        let witnesses = self.get_witnesses_list().await?;

        let block_reward = params.iter().find(|p| p.key == GET_WITNESS_PAY_PER_BLOCK).and_then(|p| p.value).unwrap_or(16_000_000) as f64 / 1_000_000.0;

        let voting_reward = params.iter().find(|p| p.key == GET_WITNESS_127_PAY_PER_BLOCK).and_then(|p| p.value).unwrap_or(160_000_000) as f64 / 1_000_000.0;

        let blocks_per_year = 365.25 * 24.0 * 60.0 * 60.0 / 3.0;
        let annual_rewards = (block_reward + voting_reward) * blocks_per_year;

        let total_votes: i64 = witnesses.witnesses.iter().map(|x| x.vote_count.unwrap_or(0)).sum();
        let total_staked_trx = total_votes as f64;

        if total_staked_trx == 0.0 {
            return Ok(Some(0.0));
        }

        let apy = (annual_rewards / total_staked_trx) * 100.0;

        Ok(Some(apy))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let witnesses = self.get_witnesses_list().await?;
        Ok(map_staking_validators(witnesses, apy))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let (account, reward, validators) = futures::try_join!(self.get_account(&address), self.get_reward(&address), self.get_staking_validators(None))?;

        Ok(map_staking_delegations(account, reward, &validators, Utc::now()))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_test_client};
    use num_bigint::BigUint;
    use primitives::Chain;

    #[tokio::test]
    async fn test_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let apy = client.get_staking_apy().await?;
        let apy_value = apy.expect("Tron staking APY should be present");

        assert!(apy_value > 0.0 && apy_value < 50.0);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let apy = client.get_staking_apy().await?;
        let validators = client.get_staking_validators(apy).await?;

        assert!(!validators.is_empty());
        assert!(validators.len() > 27);
        let system_validator = validators.iter().find(|v| v.id == "system").expect("system validator should exist");
        assert_eq!(system_validator.name, "Unstaking");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let delegations = client.get_staking_delegations(TEST_ADDRESS.to_string()).await?;
        for delegation in &delegations {
            assert_eq!(delegation.asset_id.chain, Chain::Tron);
            assert!(delegation.balance >= BigUint::from(0u32));
            assert!(delegation.rewards >= BigUint::from(0u32));
            assert!(delegation.shares >= BigUint::from(0u32));
        }
        Ok(())
    }
}
