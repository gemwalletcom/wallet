use super::{
    client::DedustClient,
    constants::SWAP_GAS,
    model::{DedustAsset, PoolRoute, PoolType},
    tx_builder::DedustSwap,
};
use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    client_factory::create_ton_client,
    fees::subtract_bps,
    models::nonzero_swap_value,
    route_cache::{OutpricedCache, amount_key, best_quote},
    route_path::RoutePath,
};
use async_trait::async_trait;
use futures::{TryStreamExt, future::join_all, stream};
use gem_client::Client;
use gem_ton::{address::Address, models::TonConnectMessage, rpc::client::TonClient};
use num_bigint::BigUint;
use primitives::{Chain, asset_constants::TON_USDT_ASSET_ID};
use std::{fmt::Debug, iter::once, sync::Arc};

#[derive(Debug)]
pub struct Dedust<C: Client> {
    provider: ProviderType,
    client: DedustClient<C>,
    outpriced_paths: OutpricedCache<RoutePath<DedustAsset>, DedustAsset>,
}

impl Dedust<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Option<Self> {
        Some(Self::with_client(create_ton_client(rpc_provider).ok()?))
    }
}

impl<C: Client> Dedust<C> {
    pub fn with_client(ton_client: TonClient<C>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Dedust),
            client: DedustClient::new(ton_client),
            outpriced_paths: OutpricedCache::default(),
        }
    }

    async fn find_route(&self, from: &DedustAsset, to: &DedustAsset, value: &BigUint) -> Result<(Vec<Address>, BigUint), SwapperError> {
        self.outpriced_paths
            .quote_live_first((from, to), amount_key(value), route_paths(from, to), Clone::clone, |paths| self.quote_paths(paths, from, to, value))
            .await
    }

    async fn quote_paths(&self, paths: Vec<RoutePath<DedustAsset>>, from: &DedustAsset, to: &DedustAsset, value: &BigUint) -> Result<(Vec<Address>, BigUint), SwapperError> {
        let quotes = self
            .outpriced_paths
            .quote_each((from, to), amount_key(value), paths, Clone::clone, |path| self.quote_path(path, from, to, value), quoted_value)
            .await;
        best_quote(quotes, quoted_value)
    }

    async fn quote_path(&self, path: RoutePath<DedustAsset>, from: &DedustAsset, to: &DedustAsset, value: &BigUint) -> Result<(Vec<Address>, BigUint), SwapperError> {
        let pairs = path.pairs(from, to);
        let hops = pairs
            .iter()
            .map(|(asset_in, asset_out)| PoolType::ALL.into_iter().map(|pool_type| pool_type.pool_address(asset_in, asset_out)).collect())
            .collect::<Result<Vec<Vec<Address>>, _>>()?;
        stream::iter(hops.iter().zip(&pairs).map(Ok))
            .try_fold((vec![], value.clone()), |(route, value_in), (pools, (asset_in, _))| async move {
                let (pool, value_out) = self.quote_hop(pools, asset_in, &value_in).await?;
                Ok(([route, vec![pool]].concat(), value_out))
            })
            .await
    }

    async fn quote_hop(&self, pools: &[Address], asset_in: &DedustAsset, value: &BigUint) -> Result<(Address, BigUint), SwapperError> {
        let quotes = join_all(pools.iter().map(|pool| async move { Ok((*pool, self.client.estimate_swap_out(pool, asset_in, value).await?)) })).await;
        best_quote(quotes, quoted_value)
    }
}

fn quoted_value<T>((_, value_out): &(T, BigUint)) -> u128 {
    amount_key(value_out)
}

fn route_paths(from: &DedustAsset, to: &DedustAsset) -> Vec<RoutePath<DedustAsset>> {
    let intermediaries = once(DedustAsset::Native).chain(DedustAsset::from_asset_id(&TON_USDT_ASSET_ID).ok());
    RoutePath::all(from, to, intermediaries, |left, right| left == right)
}

#[async_trait]
impl<C> Swapper for Dedust<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Ton)]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let request = request.reserve_native_attachment(&BigUint::from(SWAP_GAS))?;
        let from = DedustAsset::from_asset_id(&request.from_asset.asset_id())?;
        let to = DedustAsset::from_asset_id(&request.to_asset.asset_id())?;
        let swap_value = subtract_bps(&request.value, request.on_chain_referral_fee().bps);
        let (pools, value_out) = self.find_route(&from, &to, nonzero_swap_value(&swap_value)?).await?;

        Quote::from_route(&request, self.provider(), value_out, request.options.slippage.bps, &PoolRoute::new(&pools), None)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let swap = DedustSwap::new(quote, &quote.route_data()?, &quote.request.on_chain_referral_fee())?;
        let from = DedustAsset::from_asset_id(&quote.request.from_asset.asset_id())?;
        let vault = from.vault_address()?;
        let messages = match &from {
            DedustAsset::Native => swap.native_messages(&vault)?,
            DedustAsset::Jetton(jetton) => swap.jetton_messages(&vault, &self.client.jetton_wallet(&quote.request.wallet_address, jetton).await?)?,
        };
        TonConnectMessage::swap_quote_data(&messages).map_err(SwapperError::compute_quote_error)
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        dedust::testkit::{mock_request, not, stton, ton, usdt},
        testkit::dynode_provider,
    };

    #[tokio::test]
    async fn test_dedust_quotes_and_quote_data() -> Result<(), SwapperError> {
        let provider = Dedust::new(dynode_provider([Chain::Ton])).unwrap();
        for (from, to, value) in [(ton(), usdt(), 1_000_000_000u64), (usdt(), ton(), 5_000_000), (usdt(), not(), 5_000_000), (ton(), stton(), 10_000_000_000)] {
            let quote = provider.get_quote(&mock_request(from, to, value)).await?;
            let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            (&provider as &dyn Swapper).verify_contracts(&quote, &data)?;
            let messages: Vec<TonConnectMessage> = serde_json::from_str(&data.data)?;
            println!("DeDust quote: to_value={} route={} messages={}", quote.to_value, quote.data.routes[0].route_data, messages.len());

            assert!(quote.to_value > BigUint::ZERO);
            assert_eq!(messages.len(), 2);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Options,
        dedust::{
            constants::ESTIMATE_SWAP_OUT_METHOD,
            testkit::{NATIVE_VAULT, TEST_USDT_WALLET, TON_NOT_POOL, TON_STTON_STABLE_POOL, TON_USDT_POOL, mock_request, mock_ton_client, mock_ton_client_with_calls, not, stton, ton, usdt},
        },
    };
    use std::sync::Mutex;

    fn quote_messages(data: &SwapperQuoteData) -> Vec<TonConnectMessage> {
        serde_json::from_str(&data.data).unwrap()
    }

    #[tokio::test]
    async fn test_ton_to_usdt_quote_and_quote_data() {
        let provider = Dedust::with_client(mock_ton_client(include_str!("testdata/ton_to_usdt_get_methods.json")));

        let quote = provider.get_quote(&mock_request(ton(), usdt(), 1_000_000_000)).await.unwrap();
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote.to_value, BigUint::from(1_361_362u64));
        assert_eq!(quote.route_data::<PoolRoute>().unwrap().pools, vec![TON_USDT_POOL]);
        assert_eq!(data.to, NATIVE_VAULT);
        assert_eq!(data.value, BigUint::from(1_250_000_000u64));
        assert_eq!(quote_messages(&data).len(), 2);
    }

    #[tokio::test]
    async fn test_quote_data_refuses_a_vault_the_app_does_not_ship() {
        let mut records: Vec<serde_json::Value> = serde_json::from_str(include_str!("testdata/ton_to_usdt_get_methods.json")).unwrap();
        records.push(serde_json::json!({
            "request": {"address": "EQBfBWT7X2BHg9tXAxzhz2aKiNTU1tpt5NsiK0uSDW_YAJ67", "method": "get_vault_address", "stack": [{"type": "slice", "value": "te6cckEBAQEAAwAAAQiFl+L/"}]},
            "response": {"exit_code": 0, "stack": [{"type": "cell", "value": Address::parse(TEST_USDT_WALLET).unwrap().to_boc_base64().unwrap()}]}
        }));
        let calls = Arc::new(Mutex::new(Vec::new()));
        let provider = Dedust::with_client(mock_ton_client_with_calls(&serde_json::to_string(&records).unwrap(), calls.clone()));

        let quote = provider.get_quote(&mock_request(ton(), usdt(), 1_000_000_000)).await.unwrap();
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(data.to, NATIVE_VAULT);
        assert!(!calls.lock().unwrap().iter().any(|method| method == "get_vault_address"));
    }

    #[tokio::test]
    async fn test_max_ton_quote_reserves_swap_gas() {
        let provider = Dedust::with_client(mock_ton_client(include_str!("testdata/ton_to_usdt_get_methods.json")));
        let request = QuoteRequest {
            options: Options { use_max_amount: true, ..Options::default() },
            ..mock_request(ton(), usdt(), 1_250_000_000)
        };

        let quote = provider.get_quote(&request).await.unwrap();

        assert_eq!(quote.from_value, BigUint::from(1_000_000_000u64));
        assert_eq!(quote.to_value, BigUint::from(1_361_362u64));
    }

    #[tokio::test]
    async fn test_repeat_quote_skips_outpriced_route_and_undeployed_pools() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let provider = Dedust::with_client(mock_ton_client_with_calls(include_str!("testdata/usdt_to_not_get_methods.json"), calls.clone()));
        let request = mock_request(usdt(), not(), 5_000_000);
        let estimates = |calls: &Mutex<Vec<String>>| calls.lock().unwrap().drain(..).filter(|method| method == ESTIMATE_SWAP_OUT_METHOD).count();

        let quote = provider.get_quote(&request).await.unwrap();
        assert_eq!(estimates(&calls), 6);
        assert_eq!(provider.get_quote(&request).await.unwrap(), quote);
        assert_eq!(estimates(&calls), 2);
    }

    #[tokio::test]
    async fn test_ton_to_stton_quote_takes_the_stable_pool() {
        let provider = Dedust::with_client(mock_ton_client(include_str!("testdata/ton_to_stton_get_methods.json")));

        let quote = provider.get_quote(&mock_request(ton(), stton(), 10_000_000_000)).await.unwrap();
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote.to_value, BigUint::from(8_311_262_099u64));
        assert_eq!(quote.route_data::<PoolRoute>().unwrap().pools, vec![TON_STTON_STABLE_POOL]);
        assert_eq!(data.to, NATIVE_VAULT);
    }

    #[tokio::test]
    async fn test_usdt_to_not_quote_routes_through_ton() {
        let provider = Dedust::with_client(mock_ton_client(include_str!("testdata/usdt_to_not_get_methods.json")));

        let quote = provider.get_quote(&mock_request(usdt(), not(), 5_000_000)).await.unwrap();
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote.to_value, BigUint::from(12_505_120_147_875u64));
        assert_eq!(quote.route_data::<PoolRoute>().unwrap().pools, vec![TON_USDT_POOL, TON_NOT_POOL]);
        assert_eq!(data.to, TEST_USDT_WALLET);
        assert_eq!(data.value, BigUint::from(400_000_000u64));
        assert_eq!(quote_messages(&data).len(), 2);
    }
}
