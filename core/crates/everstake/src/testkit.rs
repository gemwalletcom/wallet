use primitives::{AssetId, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator};

#[cfg(feature = "chain_integration_tests")]
use gem_evm::rpc::EthereumClient;
#[cfg(feature = "chain_integration_tests")]
use gem_jsonrpc::JsonRpcClient;
#[cfg(feature = "chain_integration_tests")]
use gem_reqwest::ReqwestClient;
#[cfg(feature = "chain_integration_tests")]
use primitives::EVMChain;
#[cfg(feature = "chain_integration_tests")]
use settings::testkit::get_test_settings;

#[cfg(feature = "chain_integration_tests")]
use crate::client::EverstakeClient;
use crate::constants::EVERSTAKE_POOL_ADDRESS;
#[cfg(feature = "chain_integration_tests")]
use crate::staking::EverstakeStakingClient;

#[cfg(feature = "chain_integration_tests")]
pub const TEST_ETHEREUM_STAKING_ADDRESS: &str = gem_evm::testkit::TEST_ADDRESS;

pub fn mock_delegation(state: DelegationState) -> Delegation {
    Delegation {
        base: DelegationBase {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            state,
            balance: 2_000_000_000_000_000_000u64.into(),
            shares: 0u32.into(),
            rewards: 0u32.into(),
            completion_date: None,
            delegation_id: "eth-delegation".to_string(),
            validator_id: EVERSTAKE_POOL_ADDRESS.to_string(),
        },
        validator: DelegationValidator::stake(Chain::Ethereum, EVERSTAKE_POOL_ADDRESS.to_string(), "Everstake".to_string(), true, 10.0, 4.2),
        price: None,
    }
}

#[cfg(feature = "chain_integration_tests")]
pub fn create_everstake_staking_client() -> EverstakeStakingClient<ReqwestClient> {
    let settings = get_test_settings();
    EverstakeStakingClient::new(
        EthereumClient::new(JsonRpcClient::new(ReqwestClient::new_test_client(settings.chains.ethereum.url)), EVMChain::Ethereum),
        Some(EverstakeClient::new(ReqwestClient::new(settings.everstake.url, gem_reqwest::reqwest_client()))),
    )
}
