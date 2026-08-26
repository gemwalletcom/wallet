use std::error::Error;

use alloy_primitives::hex;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::rpc::parsers::ProtocolParser;
use gem_evm::rpc::{EthereumClient, EvmFeeCalculator, EvmStakingClient};
use gem_evm::transaction_params::TransactionParams;
use num_bigint::BigInt;
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationValidator, StakeType};

use crate::constants::{HUB_READER_ADDRESS, STAKE_HUB_ADDRESS, STAKING_VALIDATORS_LIMIT};
use crate::encode::{decode_delegations, decode_undelegations, decode_validators, encode_delegations_call, encode_stake, encode_undelegations_call, encode_validators_call};
use crate::mapper::{map_delegations, map_staking_balance, map_validators};
use crate::model::{BscDelegation, BscUndelegation};
use crate::parser::BscParser;

pub struct BscStakingClient<C: Client + Clone> {
    client: EthereumClient<C>,
}

impl<C: Client + Clone> BscStakingClient<C> {
    pub fn new(client: EthereumClient<C>) -> Self {
        Self { client }
    }

    async fn fetch_staking_state(&self, address: &str) -> Result<(Vec<BscDelegation>, Vec<BscUndelegation>), Box<dyn Error + Sync + Send>> {
        let results = self
            .client
            .batch_eth_call(
                HUB_READER_ADDRESS,
                [
                    &hex::encode(encode_delegations_call(address, 0, STAKING_VALIDATORS_LIMIT)?),
                    &hex::encode(encode_undelegations_call(address, 0, STAKING_VALIDATORS_LIMIT)?),
                ],
            )
            .await?;

        Ok((decode_delegations(&hex::decode(&results[0])?)?, decode_undelegations(&hex::decode(&results[1])?)?))
    }

    async fn get_max_elected_validators(&self) -> Result<u16, Box<dyn Error + Sync + Send>> {
        let result_data = self.client.eth_call(STAKE_HUB_ADDRESS, &hex::decode("c473318f")?).await?;
        let bytes: [u8; 4] = result_data.get(28..32).ok_or("Invalid response format for maxElectedValidators")?.try_into()?;
        Ok(u32::from_be_bytes(bytes) as u16)
    }
}

#[async_trait]
impl<C: Client + Clone> EvmStakingClient for BscStakingClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(self
            .get_staking_validators(None)
            .await?
            .into_iter()
            .filter(|validator| validator.is_active && validator.apr.is_finite())
            .map(|validator| validator.apr)
            .reduce(f64::max))
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let limit = self.get_max_elected_validators().await?;
        let validators = decode_validators(&self.client.eth_call(HUB_READER_ADDRESS, &encode_validators_call(0, limit)).await?)?;
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
        encode_stake(stake_type, value)
    }

    fn protocol_parser(&self) -> Option<&'static dyn ProtocolParser> {
        Some(&BscParser)
    }
}

#[async_trait]
impl<C: Client + Clone> EvmFeeCalculator for BscStakingClient<C> {}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use gem_evm::rpc::EvmStakingClient;
    use num_bigint::BigUint;
    use primitives::Chain;

    use crate::testkit::{TEST_SMARTCHAIN_STAKING_ADDRESS, create_staking_client};

    #[tokio::test]
    async fn test_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_staking_client();
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
    async fn test_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_staking_client();
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
    async fn test_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_staking_client();
        let apy = client.get_staking_apy().await?.unwrap();

        println!("SmartChain APY: {}", apy);
        assert!(apy > 0.1, "Max APY should be greater than 0.1%, got: {}", apy);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_balance() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_staking_client();
        let balance = client.get_staking_balance(TEST_SMARTCHAIN_STAKING_ADDRESS).await?.unwrap();

        println!("Smartchain BNB Balance: {:?}", balance);

        assert!(balance.balance.staked > BigUint::from(1_000_000_000_000_000_000u64));

        Ok(())
    }
}
