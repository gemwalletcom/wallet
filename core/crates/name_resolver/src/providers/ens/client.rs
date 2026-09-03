use std::error::Error;

use alloy_ens::namehash;
use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::JsonRpcClient;
use primitives::EVMChain;

use super::contract::{ENSRegistry, ENSResolver};

const REGISTRY_ADDRESS: &str = "0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e";

pub struct EnsClient {
    client: EthereumClient<ReqwestClient>,
}

impl EnsClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: EthereumClient::new(JsonRpcClient::new(client), EVMChain::Ethereum),
        }
    }

    pub async fn get_resolver(&self, name: &str) -> Result<Address, Box<dyn Error + Send + Sync>> {
        let call = ENSRegistry::resolverCall { node: namehash(name) }.abi_encode();
        let result = self.client.eth_call(REGISTRY_ADDRESS, &call).await?;
        Ok(ENSRegistry::resolverCall::abi_decode_returns(&result)?)
    }

    pub async fn get_address(&self, resolver: &Address, name: &str) -> Result<Address, Box<dyn Error + Send + Sync>> {
        let call = ENSResolver::addrCall { node: namehash(name) }.abi_encode();
        let result = self.client.eth_call(&resolver.to_string(), &call).await?;
        Ok(ENSResolver::addrCall::abi_decode_returns(&result)?)
    }
}
