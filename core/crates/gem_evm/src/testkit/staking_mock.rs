use std::error::Error;

use async_trait::async_trait;
use num_bigint::BigInt;
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationValidator, StakeType};

use crate::provider::preload_mapper::TransactionParams;
use crate::rpc::EvmStakingClient;

pub struct MockStakingClient;

#[async_trait]
impl EvmStakingClient for MockStakingClient {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(Some(42.0))
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_staking_delegations(&self, _address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_staking_balance(&self, _address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(Some(AssetBalance::new(AssetId::from_chain(Chain::SmartChain), 123u32.into())))
    }

    fn encode_stake(&self, _stake_type: &StakeType, _value: &BigInt) -> Result<TransactionParams, Box<dyn Error + Sync + Send>> {
        Err("mock staking client does not encode stake calls".into())
    }
}
