#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_jsonrpc::JsonRpcClient;
use primitives::Chain;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use primitives::EVMChain;
use primitives::asset_constants::{ETHEREUM_DAI_TOKEN_ID, ETHEREUM_USDC_TOKEN_ID};
use primitives::contract_constants::ZKSYNC_UNISWAP_PERMIT2_CONTRACT;

#[cfg(all(feature = "rpc", feature = "reqwest"))]
use crate::rpc::EthereumClient;

#[cfg(feature = "rpc")]
pub mod chain_provider_mock;
pub mod eip712_mock;
pub mod fee_history_mock;
#[cfg(feature = "rpc")]
pub mod rpc_mock;
pub mod siwe_mock;
pub mod trace_call_action_mock;
pub mod transaction_object_mock;

pub const TEST_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
pub const TEST_TRANSACTION_ID: &str = "0x98dd4d9a586620f84e8066f1b015d663f9c0c94c4e0e02377840c3e6d43e2ad3";
pub const TOKEN_USDC_ADDRESS: &str = ETHEREUM_USDC_TOKEN_ID;
pub const TOKEN_DAI_ADDRESS: &str = ETHEREUM_DAI_TOKEN_ID;
pub const DEPLOYED_PERMIT2_CONTRACTS: [(Chain, &str); 8] = [
    (Chain::ZkSync, ZKSYNC_UNISWAP_PERMIT2_CONTRACT),
    (Chain::Abstract, ZKSYNC_UNISWAP_PERMIT2_CONTRACT),
    (Chain::SmartChain, "0x31c2F6fcFf4F8759b3Bd5Bf0e1084A055615c768"),
    (Chain::ZkSync, "0x686FD50007EaA636F01154d660b96110B6bFe351"),
    (Chain::Monad, "0xDca6Dd86A5E305dB99A15eaEB2a6ecfc7F579778"),
    (Chain::Mantle, "0x5d6b0f5335ec95cD2aB7E52f2A0750dd86502435"),
    (Chain::Sonic, "0xB952578f3520EE8Ea45b7914994dcf4702cEe578"),
    (Chain::Sonic, "0x7Ac9E324c2a211a389fac64b773433A17dB22948"),
];

#[cfg(all(feature = "rpc", feature = "reqwest"))]
impl EthereumClient<ReqwestClient> {
    pub fn mock_with_url(chain: EVMChain, url: &str) -> Self {
        EthereumClient::new(JsonRpcClient::new(ReqwestClient::new_test_client(url.to_string())), chain)
    }
}
