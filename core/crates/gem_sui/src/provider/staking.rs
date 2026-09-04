use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainStaking;
use primitives::{DelegationBase, DelegationValidator};

use crate::provider::staking_mapper;
use crate::rpc::SuiProvider;

#[cfg(feature = "rpc")]
#[async_trait]
impl ChainStaking for SuiProvider {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let validators = self.get_validator_apys().await?;
        let apy = staking_mapper::map_staking_apy(validators)?;
        Ok(Some(apy))
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let validators = self.get_validator_apys().await?;
        let delegation_validators = staking_mapper::map_validators(validators);
        Ok(delegation_validators)
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let delegations = self.get_stake_delegations(address).await?;
        let system_state = self.get_system_state().await?;
        let delegation_bases = staking_mapper::map_delegations(delegations, system_state);
        Ok(delegation_bases)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::*;

    #[tokio::test]
    async fn test_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let apy = client.get_staking_apy().await?;
        assert!(apy.is_some());
        println!("Staking APY: {:?}", apy);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let validators = client.get_staking_validators(Some(5.0)).await?;
        assert!(!validators.is_empty());
        println!("Found {} validators", validators.len());
        Ok(())
    }
}
