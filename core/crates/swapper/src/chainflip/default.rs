use super::{broker::BrokerClient, client::ChainflipClient, provider::ChainflipProvider};
use crate::{
    SwapperProvider,
    alien::{RpcClient, RpcProvider},
    config::{get_swap_provider_url, get_swap_proxy_url},
};
use std::sync::Arc;

impl ChainflipProvider<RpcClient, RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let chainflip_client = ChainflipClient::new(RpcClient::new(get_swap_proxy_url("chainflip-swap"), rpc_provider.clone()));

        let broker_client = BrokerClient::new(
            RpcClient::new(get_swap_proxy_url("chainflip-broker"), rpc_provider.clone()),
            RpcClient::new(get_swap_provider_url(SwapperProvider::Chainflip), rpc_provider.clone()),
        );

        Self::with_clients(chainflip_client, broker_client, rpc_provider)
    }
}
