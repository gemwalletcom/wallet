use std::error::Error;

use async_trait::async_trait;
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{AssetBalance, DelegationBase, DelegationValidator, StakeType, TransactionFee, TransactionLoadInput};

use super::EthereumClient;
use super::parsers::ProtocolParser;
use crate::provider::preload::calculate_fee;
use crate::transaction_params::TransactionParams;

#[async_trait]
pub trait EvmFeeCalculator: Send + Sync {
    async fn calculate_fee(&self, input: &TransactionLoadInput, _params: &TransactionParams, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        calculate_fee(input, gas_limit)
    }
}

#[async_trait]
pub trait EvmStakingClient: Send + Sync {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(Vec::new())
    }

    async fn get_staking_delegations(&self, _address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        Ok(Vec::new())
    }

    async fn get_staking_balance(&self, _address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }

    fn encode_stake(&self, _stake_type: &StakeType, _value: &BigInt) -> Result<TransactionParams, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support staking".into())
    }

    fn protocol_parser(&self) -> Option<&'static ProtocolParser> {
        None
    }

    fn node_check_method(&self) -> Option<&'static str> {
        None
    }

    async fn node_check_probe(&self, _address: &str) -> Result<(), Box<dyn Error + Sync + Send>> {
        Ok(())
    }
}

pub trait EvmChainProvider: EvmStakingClient + EvmFeeCalculator {}

impl<T: EvmStakingClient + EvmFeeCalculator> EvmChainProvider for T {}

#[async_trait]
impl<C: Client + Clone> EvmStakingClient for EthereumClient<C> {}

#[async_trait]
impl<C: Client + Clone> EvmFeeCalculator for EthereumClient<C> {}
