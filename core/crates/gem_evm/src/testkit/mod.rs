#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_jsonrpc::JsonRpcClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use primitives::EVMChain;
use primitives::asset_constants::{ETHEREUM_DAI_TOKEN_ID, ETHEREUM_USDC_TOKEN_ID};

#[cfg(all(feature = "rpc", feature = "reqwest"))]
use crate::rpc::EthereumClient;

pub mod eip712_mock;
#[cfg(feature = "rpc")]
pub mod rpc_mock;
pub mod siwe_mock;
#[cfg(feature = "rpc")]
pub mod staking_mock;
pub mod trace_call_action_mock;
pub mod transaction_object_mock;

pub const TEST_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
pub const TEST_TRANSACTION_ID: &str = "0x98dd4d9a586620f84e8066f1b015d663f9c0c94c4e0e02377840c3e6d43e2ad3";
pub const TOKEN_USDC_ADDRESS: &str = ETHEREUM_USDC_TOKEN_ID;
pub const TOKEN_DAI_ADDRESS: &str = ETHEREUM_DAI_TOKEN_ID;

#[cfg(all(feature = "rpc", feature = "reqwest"))]
impl EthereumClient<ReqwestClient> {
    pub fn mock_with_url(chain: EVMChain, url: &str) -> Self {
        EthereumClient::new(JsonRpcClient::new(ReqwestClient::new_test_client(url.to_string())), chain)
    }
}
