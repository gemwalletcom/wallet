use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::Chain;
use std::sync::Arc;

use crate::YielderError;

pub fn create_eth_client(provider: Arc<dyn RpcProvider>, chain: Chain) -> Result<EthereumClient<RpcClient>, YielderError> {
    EthereumClient::<RpcClient>::for_provider(provider, chain).ok_or(YielderError::NotSupportedChain)
}
