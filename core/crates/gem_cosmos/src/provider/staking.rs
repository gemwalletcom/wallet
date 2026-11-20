use async_trait::async_trait;
use chain_traits::{ChainBalances, ChainStaking};
use futures::try_join;
use number_formatter::BigNumberFormatter;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator, chain_cosmos::CosmosChain};

use crate::{
    constants::OSMOSIS_TAKER_FEE_STAKING_REWARDS_BUFFER,
    provider::staking_mapper::{calculate_network_apy, map_staking_delegations, map_staking_validators},
    rpc::client::CosmosClient,
};

#[async_trait]
impl<C: Client> ChainStaking for CosmosClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain();
        match chain {
            CosmosChain::Noble | CosmosChain::Thorchain | CosmosChain::Mayachain => Ok(None),
            CosmosChain::Cosmos | CosmosChain::Injective => {
                let denom = chain.denom();
                let (inflation, supply, staking_pool) = try_join!(self.get_inflation(), self.get_supply_by_denom(denom.as_ref()), self.get_staking_pool())?;
                let annual_rewards = inflation.inflation * supply.amount.amount;

                Ok(Some(calculate_network_apy(annual_rewards, staking_pool.pool.bonded_tokens)))
            }
            CosmosChain::Osmosis => {
                let (pool_manager_params, staking_rewards_buffer, staking_pool) = try_join!(self.get_osmosis_pool_manager_params(), self.get_balance_coin(OSMOSIS_TAKER_FEE_STAKING_REWARDS_BUFFER.to_string()), self.get_staking_pool())?;
                let staking_rewards_buffer = BigNumberFormatter::value_as_f64(&staking_rewards_buffer.balance.available, 0)?;
                let annual_rewards = pool_manager_params.params.taker_fee_params.annual_staking_rewards(staking_rewards_buffer);

                Ok(Some(calculate_network_apy(annual_rewards, staking_pool.pool.bonded_tokens)))
            }
            CosmosChain::Celestia => {
                let (annual_provisions, staking_pool) = try_join!(self.get_celestia_annual_provisions(), self.get_staking_pool())?;
                Ok(Some(calculate_network_apy(annual_provisions.annual_provisions, staking_pool.pool.bonded_tokens)))
            }
            CosmosChain::Sei => Ok(Some(5.62)),
        }
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain();
        if !chain.as_chain().is_stake_supported() {
            return Ok(vec![]);
        }
        let validators = self.get_validators().await?;
        Ok(map_staking_validators(validators.validators, chain, apy))
    }

    async fn get_staking_delegation_validators(&self, address: String) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain();
        if !chain.as_chain().is_stake_supported() {
            return Ok(vec![]);
        }
        let validators = self.get_delegations_validators(&address).await?;
        Ok(map_staking_validators(validators.validators, chain, None))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain().as_chain();
        if !chain.is_stake_supported() {
            return Ok(vec![]);
        }
        let denom = chain.as_denom().unwrap_or_default();
        let (active_delegations, unbonding, rewards) = try_join!(self.get_delegations(&address), self.get_unbonding_delegations(&address), self.get_delegation_rewards(&address),)?;

        Ok(map_staking_delegations(active_delegations, unbonding, rewards, self.get_chain(), denom))
    }
}

#[cfg(test)]
mod tests {
    use chain_traits::ChainStaking;
    use gem_client::testkit::MockClient;
    use primitives::chain_cosmos::CosmosChain;

    use crate::rpc::client::CosmosClient;

    #[tokio::test]
    async fn test_get_staking_apy() {
        let client = MockClient::new().with_get(|path| {
            let body = match path {
                "/cosmos/staking/v1beta1/pool" => r#"{"pool":{"not_bonded_tokens":"65093274631553","bonded_tokens":"156561640535903"}}"#,
                "/osmosis/poolmanager/v1beta1/Params" => r#"{"params":{"taker_fee_params":{"daily_staking_rewards_smoothing_factor":"28"}}}"#,
                "/cosmos/bank/v1beta1/balances/osmo1wd8rxw3j07jcj40en6q5dhcxlsr7c8ulrn29a3" => r#"{"balances":[{"denom":"uosmo","amount":"661477421390"}]}"#,
                _ => panic!("unexpected request {path}"),
            };
            Ok(body.as_bytes().to_vec())
        });

        let apy = CosmosClient::new(CosmosChain::Osmosis, client).get_staking_apy().await.unwrap();

        assert_eq!(apy, Some(5.507626671626291));
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{create_celestia_test_client, create_cosmos_test_client, create_osmosis_test_client};
    use chain_traits::ChainStaking;

    #[tokio::test]
    async fn test_get_celestia_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_celestia_test_client();
        let apy = client.get_staking_apy().await?.unwrap();

        assert!(apy > 0.0 && apy < 100.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_osmosis_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_osmosis_test_client();
        let apy = client.get_staking_apy().await?.unwrap();

        assert!(apy > 0.0 && apy < 100.0);

        println!("Osmosis staking APY: {}%", apy);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cosmos_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_cosmos_test_client();
        let apy = client.get_staking_apy().await?;

        assert!(apy.is_some());
        let apy_value = apy.unwrap();

        assert!(apy_value > 5.0 && apy_value < 25.0);

        println!("Cosmos staking APY: {}%", apy_value);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cosmos_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_cosmos_test_client();
        let apy = client.get_staking_apy().await?;
        let validators = client.get_staking_validators(apy).await?;

        assert!(!validators.is_empty());
        assert!(validators.len() <= 200);

        for validator in validators.iter().take(5) {
            assert!(!validator.id.is_empty());
            assert!(!validator.name.is_empty());
            assert!(validator.commission >= 0.0 && validator.commission <= 100.0);
            if validator.is_active {
                assert!(validator.apr >= 0.0);
            }
        }

        Ok(())
    }
}
