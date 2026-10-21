mod prefetch;
mod transaction;

use self::{prefetch::PrefetchedSuiData, transaction::build_mctp_transaction};
use crate::mayan::{
    client::MayanClient,
    model::{GetSwapSuiParams, MayanMctpQuote},
    tx_builder::{
        route::quote_destination_address,
        sui::{SuiClientSwapReplay, build_sui_quote_data, sui_error},
    },
    wormhole_chain::{self, WormholeChain},
};
use crate::{Quote, RpcProvider, SwapperError, SwapperQuoteData, client_factory::create_sui_client, fees::default_referral_address, route_cache::Cache};
use futures::try_join;
use gem_sui::{ESTIMATION_GAS_BUDGET, SuiClient};
use primitives::OptionStringExt;
use std::{fmt::Debug, sync::Arc};

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanMctpQuote, rpc_provider: Arc<dyn RpcProvider>, package_id_cache: &Cache<&'static str, String>) -> Result<SwapperQuoteData, SwapperError>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    let sender = quote.request.wallet_address.as_str();
    let sui_client = create_sui_client(rpc_provider)?;
    let destination_address = quote_destination_address(quote);
    let mctp_input_contract = route.mctp_input_contract.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let referrer_address = wormhole_chain::chain_for_name(&route.to_chain).ok().map(default_referral_address).non_empty();
    let prefetched = PrefetchedSuiData::prefetch(&sui_client, sender, route, ESTIMATION_GAS_BUDGET, package_id_cache);
    let swap = get_swap_transaction(client, &sui_client, quote, route, mctp_input_contract, referrer_address);
    let (prefetched, swap) = try_join!(prefetched, swap)?;

    build_sui_quote_data(&sui_client, |gas_budget| build_mctp_transaction(quote, route, &prefetched, destination_address, swap.as_ref(), gas_budget)).await
}

async fn get_swap_transaction<C>(client: &MayanClient<C>, sui_client: &SuiClient, quote: &Quote, route: &MayanMctpQuote, mctp_input_contract: &str, referrer_address: Option<String>) -> Result<Option<SuiClientSwapReplay>, SwapperError>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    if route.from_token.contract.as_str() == mctp_input_contract {
        return Ok(None);
    }

    let with_wh_fee = route.has_auction == Some(true) || route.cheaper_chain.as_deref() != Some(WormholeChain::Sui.name());
    let params = GetSwapSuiParams::new(route, quote.request.wallet_address.clone(), mctp_input_contract.to_string(), with_wh_fee, referrer_address);
    SuiClientSwapReplay::fetch(client, sui_client, params).await.map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_sui::SUI_COIN_TYPE;

    #[test]
    fn test_mctp_route_amounts() {
        let route = MayanMctpQuote::mock();
        assert_eq!(transaction::bridge_amount(&route, route.mctp_input_contract.as_deref().unwrap()).unwrap(), 1000000);

        let mut route = route;
        route.common.from_token.contract = SUI_COIN_TYPE.to_string();
        route.min_middle_amount = Some(serde_json::json!(0.99));
        assert_eq!(transaction::bridge_amount(&route, route.mctp_input_contract.as_deref().unwrap()).unwrap(), 990000);
    }
}
