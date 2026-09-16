use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::alien::{self, RpcClient, RpcProvider};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::Chain;
use std::sync::Arc;

use crate::YielderError;

pub fn create_client(provider: Arc<dyn RpcProvider>, chain: Chain) -> Result<JsonRpcClient<RpcClient>, YielderError> {
    alien::create_client(provider, chain).map_err(|_| YielderError::NotSupportedChain)
}

pub fn create_eth_client(provider: Arc<dyn RpcProvider>, chain: Chain) -> Result<EthereumClient<RpcClient>, YielderError> {
    EthereumClient::for_chain(create_client(provider, chain)?, chain).ok_or(YielderError::NotSupportedChain)
}
