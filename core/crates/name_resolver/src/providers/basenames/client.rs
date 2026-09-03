use std::error::Error;

use alloy_ens::namehash;
use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::JsonRpcClient;
use primitives::EVMChain;

use super::contract::L2Resolver;

const L2_RESOLVER_ADDRESS: &str = "0xC6d566A56A1aFf6508b41f6c90ff131615583BCD";

pub struct BasenamesClient {
    client: EthereumClient<ReqwestClient>,
}

impl BasenamesClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: EthereumClient::new(JsonRpcClient::new(client), EVMChain::Base),
        }
    }

    pub async fn get_address(&self, name: &str) -> Result<Address, Box<dyn Error + Send + Sync>> {
        let call = L2Resolver::addrCall { node: namehash(name) }.abi_encode();
        let result = self.client.eth_call(L2_RESOLVER_ADDRESS, &call).await?;
        Ok(L2Resolver::addrCall::abi_decode_returns(&result)?)
    }
}
