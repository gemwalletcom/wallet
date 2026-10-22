mod prefetch;
mod transaction;

use self::{prefetch::PrefetchedSuiData, transaction::build_mctp_transaction};
use crate::mayan::{
    client::MayanClient,
    model::{GetSwapSuiParams, MayanMctpQuote},
    tx_builder::{
        route::quote_destination_address,
        sui::{SuiClientSwapReplay, build_sui_quote_data, requires_client_swap, sui_error},
    },
    wormhole_chain::WormholeChain,
};
use crate::{Quote, RpcProvider, SwapperError, SwapperQuoteData, client_factory::create_sui_client, route_cache::Cache};
use futures::try_join;
use gem_sui::SuiClient;
use std::{fmt::Debug, sync::Arc};

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanMctpQuote, rpc_provider: Arc<dyn RpcProvider>, package_id_cache: &Cache<&'static str, String>) -> Result<SwapperQuoteData, SwapperError>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    let sender = quote.request.wallet_address.as_str();
    let sui_client = create_sui_client(rpc_provider)?;
    let destination_address = quote_destination_address(quote);
    let mctp_input_contract = route.mctp_input_contract.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let prefetched = PrefetchedSuiData::prefetch(&sui_client, sender, route, package_id_cache);
    let swap = get_swap_transaction(client, &sui_client, quote, route, mctp_input_contract);
    let (prefetched, swap) = try_join!(prefetched, swap)?;

    build_sui_quote_data(&sui_client, |gas_budget| build_mctp_transaction(quote, route, &prefetched, destination_address, swap.as_ref(), gas_budget)).await
}

async fn get_swap_transaction<C>(client: &MayanClient<C>, sui_client: &SuiClient, quote: &Quote, route: &MayanMctpQuote, mctp_input_contract: &str) -> Result<Option<SuiClientSwapReplay>, SwapperError>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    if !requires_client_swap(route, mctp_input_contract) {
        return Ok(None);
    }

    let with_wh_fee = route.has_auction == Some(true) || route.cheaper_chain.as_deref() != Some(WormholeChain::Sui.name());
    let params = GetSwapSuiParams::new(route, quote.request.wallet_address.clone(), mctp_input_contract.to_string(), with_wh_fee, route.destination_referrer_address()?);
    SuiClientSwapReplay::fetch(client, sui_client, params).await.map(Some)
}
