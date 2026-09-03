use std::error::Error;

use async_trait::async_trait;
use gem_client::Client;
use gem_evm::rpc::parsers::ProtocolParser;
use gem_evm::rpc::{EthereumClient, EvmFeeCalculator, EvmStakingClient};
use gem_evm::transaction_params::TransactionParams;
use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationState, DelegationValidator, StakeType};

use crate::client::{EverstakeClient, account_state};
use crate::constants::EVERSTAKE_POOL_ADDRESS;
use crate::encode::encode_stake;
use crate::mapper::{map_balance_to_delegation, map_staking_balance, map_withdraw_request_to_delegations};
use crate::parser::EverstakeParser;

pub struct EverstakeStakingClient<C: Client + Clone> {
    client: EthereumClient<C>,
    stats: Option<EverstakeClient<C>>,
}

impl<C: Client + Clone> EverstakeStakingClient<C> {
    pub fn new(client: EthereumClient<C>, stats: Option<EverstakeClient<C>>) -> Self {
        Self { client, stats }
    }
}

#[async_trait]
impl<C: Client + Clone> EvmStakingClient for EverstakeStakingClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        match &self.stats {
            Some(stats) => Ok(Some(stats.get_staking_apy().await?)),
            None => Ok(None),
        }
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(vec![DelegationValidator::stake(
            Chain::Ethereum,
            EVERSTAKE_POOL_ADDRESS.to_string(),
            "Everstake".to_string(),
            true,
            0.1,
            apy.unwrap_or(0.0),
        )])
    }

    async fn get_staking_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let state = account_state(&self.client, address).await?;

        let mut delegations = Vec::new();

        let active_balance = state.deposited_balance;
        if active_balance > BigUint::zero() {
            delegations.push(map_balance_to_delegation(&active_balance, &state.restaked_reward, DelegationState::Active));
        }

        let pending_balance = state.pending_balance + state.pending_deposited_balance;
        if pending_balance > BigUint::zero() {
            delegations.push(map_balance_to_delegation(&pending_balance, &BigUint::zero(), DelegationState::Activating));
        }

        delegations.extend(map_withdraw_request_to_delegations(&state.withdraw_request));

        Ok(delegations)
    }

    async fn get_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let delegations = self.get_staking_delegations(address).await?;
        Ok(Some(AssetBalance::new_balance(AssetId::from_chain(Chain::Ethereum), map_staking_balance(&delegations))))
    }

    fn encode_stake(&self, stake_type: &StakeType, value: &BigInt) -> Result<TransactionParams, Box<dyn Error + Sync + Send>> {
        encode_stake(stake_type, value)
    }

    fn protocol_parser(&self) -> Option<&'static dyn ProtocolParser> {
        Some(&EverstakeParser)
    }
}

#[async_trait]
impl<C: Client + Clone> EvmFeeCalculator for EverstakeStakingClient<C> {}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use gem_evm::rpc::EvmStakingClient;
    use num_bigint::BigUint;
    use primitives::{Chain, DelegationState};

    use crate::testkit::{TEST_ETHEREUM_STAKING_ADDRESS, create_everstake_staking_client};

    #[tokio::test]
    async fn test_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_everstake_staking_client();
        let apy = client.get_staking_apy().await?.unwrap();

        assert!(apy > 2.0 && apy < 6.0, "APY should be between 2% and 6%, got: {}", apy);
        println!("Ethereum APY: {}", apy);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_everstake_staking_client();
        let validators = client.get_staking_validators(Some(4.2)).await?;

        println!("Ethereum Validators count: {}", validators.len());
        assert_eq!(validators.len(), 1);

        if let Some(validator) = validators.first() {
            assert_eq!(validator.chain, Chain::Ethereum);
            assert_eq!(validator.name, "Everstake");
            assert!(validator.is_active);
            assert_eq!(validator.apr, 4.2);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_everstake_staking_client();
        let delegations = client.get_staking_delegations("0xF3A43C831D4462019635C5E08F4c0920218f3b93").await?;

        println!("Ethereum Delegations count: {}", delegations.len());
        println!("Ethereum Delegations: {:?}", delegations);

        for delegation in &delegations {
            assert_eq!(delegation.asset_id.chain, Chain::Ethereum);
            assert!(
                delegation.state == DelegationState::Active
                    || delegation.state == DelegationState::Activating
                    || delegation.state == DelegationState::Deactivating
                    || delegation.state == DelegationState::AwaitingWithdrawal
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_balance() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_everstake_staking_client();
        let balance = client.get_staking_balance(TEST_ETHEREUM_STAKING_ADDRESS).await?;

        println!("Ethereum staking balance: {:?}", balance);

        assert!(balance.unwrap().balance.staked > BigUint::from(0u32));

        Ok(())
    }
}
