use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use gem_client::Client;
use gem_evm::provider::preload_mapper::TransactionParams;
use gem_evm::rpc::{EthereumClient, EvmStakingClient};
use num_bigint::BigInt;
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationValidator, StakeType};

use crate::constants::STAKING_LENS_CONTRACT;
use crate::mapper::{
    decode_get_lens_apys, decode_get_lens_balance, decode_get_lens_delegations, decode_get_lens_validators, encode_get_lens_apys, encode_get_lens_balance,
    encode_get_lens_delegations, encode_get_lens_validators, encode_monad_staking, map_lens_delegations, map_lens_validator,
};

const MONAD_VALIDATOR_NAMES: &[(u64, &str)] = &[(16, "MonadVision"), (5, "Alchemy"), (10, "Stakin"), (9, "Everstake")];
const ETH_CALL_MONAD_DELEGATIONS_CHECK: &str = "eth_call_monad_delegations";

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
        let data = encode_get_lens_delegations(address)?;
        self.call_lens(data).await
    }
}

#[async_trait]
impl<C: Client + Clone> EvmStakingClient for MonadStakingClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let data = encode_get_lens_apys(&[]);
        let result = self.call_lens(data).await?;

        let apys = decode_get_lens_apys(&result)?;
        let apy_bps = apys.into_iter().max().unwrap_or(0);

        if apy_bps == 0 {
            return Ok(None);
        }

        Ok(Some(apy_bps as f64 / 100.0))
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let validator_names: HashMap<u64, &str> = MONAD_VALIDATOR_NAMES.iter().copied().collect();
        let validator_ids = MONAD_VALIDATOR_NAMES.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        let data = encode_get_lens_validators(&validator_ids);
        let result = self.call_lens(data).await?;

        let (validators, network_apy_bps) = decode_get_lens_validators(&result)?;
        let network_apy = network_apy_bps as f64 / 100.0;

        Ok(validators
            .into_iter()
            .map(|validator| map_lens_validator(&validator, &validator_names, network_apy))
            .collect())
    }

    async fn get_staking_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let positions = match self.call_delegations(address).await {
            Ok(bytes) => match decode_get_lens_delegations(&bytes) {
                Ok(position_list) => position_list,
                Err(_) => return Ok(Vec::new()),
            },
            Err(_) => return Ok(Vec::new()),
        };

        Ok(map_lens_delegations(address, positions))
    }

    async fn get_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let data = encode_get_lens_balance(address)?;
        let result = self.call_lens(data).await?;
        let balance = decode_get_lens_balance(&result)?;
        Ok(Some(AssetBalance::new_balance(
            AssetId::from_chain(Chain::Monad),
            primitives::Balance::stake_balance(balance.staked, balance.pending, Some(balance.rewards)),
        )))
    }

    fn encode_stake(&self, stake_type: &StakeType, value: &BigInt) -> Result<TransactionParams, Box<dyn Error + Sync + Send>> {
        let (to, data, stake_value) = encode_monad_staking(stake_type, value)?;
        Ok(TransactionParams::new(to.to_string(), data, stake_value))
    }

    fn node_check_method(&self) -> Option<&'static str> {
        Some(ETH_CALL_MONAD_DELEGATIONS_CHECK)
    }

    async fn node_check_probe(&self, address: &str) -> Result<(), Box<dyn Error + Sync + Send>> {
        self.call_delegations(address).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use serde_json::json;

    use super::*;
    use crate::testkit::TEST_MONAD_ADDRESS;

    #[tokio::test]
    async fn test_call_delegations() {
        let rpc_client = mock_jsonrpc_client(|request_method, params| {
            assert_eq!(request_method, gem_evm::method::ETH_CALL);
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
        let client = MonadStakingClient::new(EthereumClient::new(rpc_client, primitives::EVMChain::Monad));

        assert_eq!(client.call_delegations(TEST_MONAD_ADDRESS).await.unwrap(), Vec::<u8>::new());
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use gem_evm::rpc::EvmStakingClient;

    use crate::testkit::{TEST_MONAD_ADDRESS, create_monad_staking_client};

    #[tokio::test]
    async fn test_monad_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_monad_staking_client();
        let delegations = client.get_staking_delegations(TEST_MONAD_ADDRESS).await?;

        assert!(!delegations.is_empty());

        println!("Monad Delegations count: {}", delegations.len());
        println!("Monad Delegations: {:?}", delegations);

        Ok(())
    }

    #[tokio::test]
    async fn test_monad_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_monad_staking_client();
        let apy = client.get_staking_apy().await?.unwrap();

        println!("Monad APY: {}", apy);
        assert!(apy > 0.0);
        Ok(())
    }
}
