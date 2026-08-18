use std::error::Error;

use alloy_primitives::hex;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::provider::preload_mapper::TransactionParams;
use gem_evm::rpc::{EthereumClient, EvmStakingClient};
use num_bigint::BigInt;
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationValidator, StakeType};

use crate::mapper::{encode_stake_hub, map_delegations, map_staking_balance, map_validators};
use crate::stake_hub::{
    BscDelegation, BscUndelegation, HUB_READER_ADDRESS, STAKE_HUB_ADDRESS, decode_delegations_return, decode_undelegations_return, decode_validators_return,
    encode_delegations_call, encode_undelegations_call, encode_validators_call,
};

const STAKING_VALIDATORS_LIMIT: u16 = 128;

pub struct BscStakingClient<C: Client + Clone> {
    client: EthereumClient<C>,
}

impl<C: Client + Clone> BscStakingClient<C> {
    pub fn new(client: EthereumClient<C>) -> Self {
        Self { client }
    }

    async fn fetch_staking_state(&self, address: &str) -> Result<(Vec<BscDelegation>, Vec<BscUndelegation>), Box<dyn Error + Sync + Send>> {
        let delegations_call_data = encode_delegations_call(address, 0, STAKING_VALIDATORS_LIMIT)?;
        let undelegations_call_data = encode_undelegations_call(address, 0, STAKING_VALIDATORS_LIMIT)?;

        let results = self
            .client
            .batch_call_data(vec![(HUB_READER_ADDRESS, delegations_call_data), (HUB_READER_ADDRESS, undelegations_call_data)])
            .await?;

        let delegations = decode_delegations_return(&hex::decode(&results[0])?)?;
        let undelegations = decode_undelegations_return(&hex::decode(&results[1])?)?;

        Ok((delegations, undelegations))
    }

    async fn get_max_elected_validators(&self) -> Result<u16, Box<dyn Error + Sync + Send>> {
        let result_data = self.client.eth_call(STAKE_HUB_ADDRESS, &hex::decode("c473318f")?).await?;

        if result_data.len() >= 32 {
            let value = u32::from_be_bytes([result_data[28], result_data[29], result_data[30], result_data[31]]) as u16;
            Ok(value)
        } else {
            Err("Invalid response format for maxElectedValidators".into())
        }
    }
}

#[async_trait]
impl<C: Client + Clone> EvmStakingClient for BscStakingClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let validators = self.get_staking_validators(None).await?;
        let max_apr = validators
            .into_iter()
            .filter(|validator| validator.is_active)
            .filter_map(|validator| if validator.apr.is_finite() { Some(validator.apr) } else { None })
            .fold(None, |acc: Option<f64>, apr| match acc {
                Some(current) if current >= apr => Some(current),
                _ => Some(apr),
            });
        Ok(max_apr)
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let limit = self.get_max_elected_validators().await?;
        let call_data = encode_validators_call(0, limit);

        let validators = decode_validators_return(&self.client.eth_call(HUB_READER_ADDRESS, &call_data).await?)?;

        Ok(map_validators(validators))
    }

    async fn get_staking_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let (delegations, undelegations) = self.fetch_staking_state(address).await?;

        Ok(map_delegations(delegations, undelegations))
    }

    async fn get_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let (delegations, undelegations) = self.fetch_staking_state(address).await?;

        Ok(Some(AssetBalance::new_balance(
            AssetId::from_chain(Chain::SmartChain),
            map_staking_balance(&delegations, &undelegations),
        )))
    }

    fn encode_stake(&self, stake_type: &StakeType, value: &BigInt) -> Result<TransactionParams, Box<dyn Error + Sync + Send>> {
        let (to, data, stake_value) = encode_stake_hub(stake_type, value)?;
        Ok(TransactionParams::new(to.to_string(), data, stake_value))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use gem_evm::rpc::EvmStakingClient;
    use num_bigint::BigUint;
    use primitives::Chain;

    use crate::testkit::{TEST_SMARTCHAIN_STAKING_ADDRESS, create_bsc_staking_client};

    #[tokio::test]
    async fn test_smartchain_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_bsc_staking_client();
        let validators = client.get_staking_validators(Some(0.0)).await?;

        println!("SmartChain Validators count: {}", validators.len());
        assert!(validators.len() > 24);

        if let Some(validator) = validators.first() {
            assert_eq!(validator.chain, Chain::SmartChain);
            assert!(!validator.id.is_empty());
            assert!(!validator.name.is_empty());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_bsc_staking_client();
        let delegations = client.get_staking_delegations(TEST_SMARTCHAIN_STAKING_ADDRESS).await?;

        println!("SmartChain Delegations: {:?}", delegations);

        assert!(!delegations.is_empty());

        for delegation in &delegations {
            println!(
                "Delegation - Validator: {}, Balance: {}, State: {:?}",
                delegation.validator_id, delegation.balance, delegation.state
            );
            assert_eq!(delegation.asset_id.chain, Chain::SmartChain);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_bsc_staking_client();
        let apy = client.get_staking_apy().await?.unwrap();

        println!("SmartChain APY: {}", apy);
        assert!(apy > 0.1, "Max APY should be greater than 0.1%, got: {}", apy);

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_staking_balance() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_bsc_staking_client();
        let balance = client.get_staking_balance(TEST_SMARTCHAIN_STAKING_ADDRESS).await?.unwrap();

        println!("Smartchain BNB Balance: {:?}", balance);

        assert!(balance.balance.staked > BigUint::from(1_000_000_000_000_000_000u64));

        Ok(())
    }
}
