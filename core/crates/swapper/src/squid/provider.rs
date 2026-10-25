use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use gem_client::Client;

use super::client::SquidClient;
use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, RpcProvider, SwapAmountMode, SwapResult, SwapResultRequest, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData, config::get_swap_proxy_url,
};

#[derive(Debug)]
pub struct Squid<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: SquidClient<C>,
}

impl Squid<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Squid),
            client: SquidClient::new(RpcClient::new(get_swap_proxy_url("squid"), rpc_provider)),
        }
    }
}

#[async_trait]
impl<C> Swapper for Squid<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, _request: &QuoteRequest) -> Result<Quote, SwapperError> {
        Err(SwapperError::NotSupportedChain)
    }

    async fn get_quote_data(&self, _quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        Err(SwapperError::NotSupportedChain)
    }

    async fn get_swap_result(&self, request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        let result = self.client.get_status(&request.transaction_hash, request.chain.network_id()).await?;
        Ok(SwapResult::from(result.squid_transaction_status.swap_status()))
    }
}
