use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use gem_client::Client;
use gem_evm::rpc::parsers::ProtocolParser;
use gem_evm::rpc::{EthereumClient, EvmFeeCalculator, EvmStakingClient};
use gem_evm::transaction_params::TransactionParams;
use num_bigint::BigInt;
use num_traits::Zero;
use primitives::{AssetBalance, AssetId, Balance, Chain, DelegationBase, DelegationValidator, StakeType};

use crate::constants::{STAKING_LENS_CONTRACT, VALIDATOR_NAMES};
use crate::encode::{decode_apys, decode_balance, decode_delegations, decode_validators, encode_apys, encode_balance, encode_delegations, encode_stake, encode_validators};
use crate::mapper::{map_delegation, map_validator};
use crate::parser::MonadParser;

pub struct MonadStakingClient<C: Client + Clone> {
    client: EthereumClient<C>,
}

impl<C: Client + Clone> MonadStakingClient<C> {
    pub fn new(client: EthereumClient<C>) -> Self {
        Self { client }
    }

    async fn call_lens(&self, data: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
        self.client.eth_call(STAKING_LENS_CONTRACT, &data).await
    }

    async fn call_delegations(&self, address: &str) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
        self.call_lens(encode_delegations(address)?).await
    }
}

#[async_trait]
impl<C: Client + Clone> EvmStakingClient for MonadStakingClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let result = self.call_lens(encode_apys(&[])).await?;
        Ok(decode_apys(&result)?.into_iter().max().filter(|apy_bps| *apy_bps > 0).map(|apy_bps| apy_bps as f64 / 100.0))
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let validator_names: HashMap<u64, &str> = VALIDATOR_NAMES.iter().copied().collect();
        let validator_ids = VALIDATOR_NAMES.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        let result = self.call_lens(encode_validators(&validator_ids)).await?;
        let (validators, network_apy_bps) = decode_validators(&result)?;
        let network_apy = network_apy_bps as f64 / 100.0;

        Ok(validators.into_iter().map(|validator| map_validator(&validator, &validator_names, network_apy)).collect())
    }

    async fn get_staking_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let positions = self.call_delegations(address).await.and_then(|bytes| decode_delegations(&bytes)).unwrap_or_default();

        Ok(positions
            .into_iter()
            .filter(|position| !position.amount.is_zero() || !position.rewards.is_zero())
            .map(|position| map_delegation(address, position))
            .collect())
    }

    async fn get_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let result = self.call_lens(encode_balance(address)?).await?;
        let balance = decode_balance(&result)?;
        Ok(Some(AssetBalance::new_balance(
            AssetId::from_chain(Chain::Monad),
            Balance::stake_balance(balance.staked, balance.pending, Some(balance.rewards)),
        )))
    }

    fn encode_stake(&self, stake_type: &StakeType, value: &BigInt) -> Result<TransactionParams, Box<dyn Error + Sync + Send>> {
        encode_stake(stake_type, value)
    }

    fn protocol_parser(&self) -> Option<&'static ProtocolParser> {
        Some(&MonadParser)
    }

    fn node_check_method(&self) -> Option<&'static str> {
        Some("eth_call_monad_delegations")
    }

    async fn node_check_probe(&self, address: &str) -> Result<(), Box<dyn Error + Sync + Send>> {
        self.call_delegations(address).await.map(|_| ())
    }
}

#[async_trait]
impl<C: Client + Clone> EvmFeeCalculator for MonadStakingClient<C> {}

#[cfg(test)]
mod tests {
    use gem_evm::method;
    use gem_evm::rpc::{EthereumClient, EvmStakingClient};
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::EVMChain;
    use serde_json::json;

    use super::*;
    use crate::testkit::TEST_ADDRESS;

    #[tokio::test]
    async fn test_call_delegations() {
        let rpc_client = mock_jsonrpc_client(|request_method, params| {
            assert_eq!(request_method, method::ETH_CALL);
            assert_eq!(
                params,
                &json!([
                    {
                        "data": "0x31cc13ba000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb7",
                        "to": STAKING_LENS_CONTRACT
                    },
                    "latest"
                ])
            );
            Ok(json!("0x"))
        });
        let client = MonadStakingClient::new(EthereumClient::new(rpc_client, EVMChain::Monad));

        assert_eq!(client.call_delegations(TEST_ADDRESS).await.unwrap(), Vec::<u8>::new());
    }

    #[tokio::test]
    async fn test_node_check_probe_calls_lens() {
        let rpc_client = mock_jsonrpc_client(|request_method, _params| {
            assert_eq!(request_method, method::ETH_CALL);
            Ok(json!("0x"))
        });
        let client = MonadStakingClient::new(EthereumClient::new(rpc_client, EVMChain::Monad));

        assert_eq!(client.node_check_method(), Some("eth_call_monad_delegations"));
        client.node_check_probe(TEST_ADDRESS).await.unwrap();
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use gem_evm::rpc::EvmStakingClient;

    use crate::testkit::{TEST_ADDRESS, create_staking_client};

    #[tokio::test]
    async fn test_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_staking_client();
        let delegations = client.get_staking_delegations(TEST_ADDRESS).await?;

        assert!(!delegations.is_empty());

        println!("Monad Delegations count: {}", delegations.len());
        println!("Monad Delegations: {:?}", delegations);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_staking_client();
        let apy = client.get_staking_apy().await?.unwrap();

        println!("Monad APY: {}", apy);
        assert!(apy > 0.0);
        Ok(())
    }
}
