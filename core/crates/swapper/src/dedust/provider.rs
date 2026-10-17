use super::{
    client::DedustClient,
    constants::SWAP_GAS,
    model::{DedustAsset, PoolRoute},
    tx_builder::DedustSwap,
};
use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    client_factory::create_ton_client,
    fees::{ReferralFee, default_referral_fees, subtract_bps},
    models::nonzero_swap_value,
    route_cache::{OutpricedCache, amount_key, quote_outcome},
    route_path::RoutePath,
};
use async_trait::async_trait;
use futures::{
    StreamExt, TryStreamExt,
    future::{join_all, try_join_all},
    stream,
};
use gem_client::Client;
use gem_ton::{address::Address, models::TonConnectMessage, rpc::client::TonClient};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, asset_constants::TON_USDT_ASSET_ID};
use std::{fmt::Debug, sync::Arc};

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

    fn referral_fee() -> ReferralFee {
        default_referral_fees().ton
    }

    async fn find_route(&self, from: &DedustAsset, to: &DedustAsset, value: &BigUint) -> Result<(Vec<Address>, BigUint), SwapperError> {
        self.outpriced_paths
            .quote_live_first((from, to), amount_key(value), route_paths(from, to)?, Clone::clone, |paths| self.quote_paths(paths, from, to, value))
            .await
    }

    async fn quote_paths(&self, paths: Vec<RoutePath<DedustAsset>>, from: &DedustAsset, to: &DedustAsset, value: &BigUint) -> Result<(Vec<Address>, BigUint), SwapperError> {
        let quotes = join_all(paths.into_iter().map(|path| async move { (path.clone(), self.quote_path(path, from, to, value).await) })).await;
        let outcomes = quotes.iter().filter_map(|(path, quote)| Some((path.clone(), quote_outcome(quote.as_ref().map(|(_, value_out)| amount_key(value_out)))?)));
        self.outpriced_paths.record(from, to, amount_key(value), outcomes);
        let best = quotes.iter().filter_map(|(_, quote)| quote.as_ref().ok()).max_by_key(|(_, value_out)| value_out).cloned();
        best.ok_or_else(|| quotes.into_iter().find_map(|(_, quote)| quote.err()).unwrap_or(SwapperError::NoQuoteAvailable))
    }

    async fn quote_path(&self, path: RoutePath<DedustAsset>, from: &DedustAsset, to: &DedustAsset, value: &BigUint) -> Result<(Vec<Address>, BigUint), SwapperError> {
        let pairs = path.pairs(from, to);
        let pools = try_join_all(pairs.iter().map(|(asset_in, asset_out)| self.client.pool_address(asset_in, asset_out))).await?;
        let value_out = stream::iter(pools.iter().zip(&pairs).map(Ok))
            .try_fold(value.clone(), |value_in, (pool, (asset_in, _))| async move { self.client.estimate_swap_out(pool, asset_in, &value_in).await })
            .await?;
        Ok((pools, value_out))
    }
}

fn route_paths(from: &DedustAsset, to: &DedustAsset) -> Result<Vec<RoutePath<DedustAsset>>, SwapperError> {
    let intermediaries = [DedustAsset::Native, DedustAsset::from_asset_id(&TON_USDT_ASSET_ID)?];
    Ok(RoutePath::all(from, to, intermediaries, |left, right| left == right))
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

    async fn preload_routes(&self, from_asset: &AssetId, to_asset: &AssetId) {
        if from_asset.chain != Chain::Ton || to_asset.chain != Chain::Ton {
            return;
        }
        let (Ok(from), Ok(to)) = (DedustAsset::from_asset_id(from_asset), DedustAsset::from_asset_id(to_asset)) else {
            return;
        };
        let pairs = route_paths(&from, &to).unwrap_or_default().iter().flat_map(|path| path.pairs(&from, &to)).collect::<Vec<_>>();
        stream::iter(pairs)
            .for_each_concurrent(None, |(asset_in, asset_out)| async move { _ = self.client.pool_address(&asset_in, &asset_out).await })
            .await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let request = request.reserve_native_attachment(&BigUint::from(SWAP_GAS))?;
        let from = DedustAsset::from_asset_id(&request.from_asset.asset_id())?;
        let to = DedustAsset::from_asset_id(&request.to_asset.asset_id())?;
        let swap_value = subtract_bps(&request.value, Self::referral_fee().bps);
        let (pools, value_out) = self.find_route(&from, &to, nonzero_swap_value(&swap_value)?).await?;

        Quote::from_route(&request, self.provider(), value_out, &PoolRoute::new(&pools), None)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let swap = DedustSwap::new(quote, &quote.route_data()?, &Self::referral_fee())?;
        let from = DedustAsset::from_asset_id(&quote.request.from_asset.asset_id())?;
        let messages = match &from {
            DedustAsset::Native => swap.native_messages(&self.client.vault_address(&from).await?)?,
            DedustAsset::Jetton(jetton) => {
                let (vault, jetton_wallet) = futures::try_join!(self.client.vault_address(&from), self.client.jetton_wallet(&quote.request.wallet_address, jetton))?;
                swap.jetton_messages(&vault, &jetton_wallet)?
            }
        };
        TonConnectMessage::swap_quote_data(&messages).map_err(SwapperError::compute_quote_error)
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        dedust::testkit::{mock_request, not, ton, usdt},
        testkit::dynode_provider,
    };

    #[tokio::test]
    async fn test_dedust_quotes_and_quote_data() -> Result<(), SwapperError> {
        let provider = Dedust::new(dynode_provider(Chain::Ton)).unwrap();
        for (from, to, value) in [(ton(), usdt(), 1_000_000_000u64), (usdt(), ton(), 5_000_000), (usdt(), not(), 5_000_000)] {
            let quote = provider.get_quote(&mock_request(from, to, value)).await?;
            let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
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
            testkit::{NATIVE_VAULT, TEST_USDT_WALLET, TON_NOT_POOL, TON_USDT_POOL, mock_request, mock_ton_client, mock_ton_client_with_calls, not, ton, usdt},
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
    async fn test_max_ton_quote_reserves_swap_gas() {
        let provider = Dedust::with_client(mock_ton_client(include_str!("testdata/ton_to_usdt_get_methods.json")));
        let request = QuoteRequest {
            options: Options {
                use_max_amount: true,
                ..Options::new_with_slippage(100.into())
            },
            ..mock_request(ton(), usdt(), 1_250_000_000)
        };

        let quote = provider.get_quote(&request).await.unwrap();

        assert_eq!(quote.from_value, BigUint::from(1_000_000_000u64));
        assert_eq!(quote.to_value, BigUint::from(1_361_362u64));
    }

    #[tokio::test]
    async fn test_repeat_quote_skips_outpriced_route() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let provider = Dedust::with_client(mock_ton_client_with_calls(include_str!("testdata/usdt_to_not_get_methods.json"), calls.clone()));
        let request = mock_request(usdt(), not(), 5_000_000);
        let estimates = |calls: &Mutex<Vec<String>>| calls.lock().unwrap().drain(..).filter(|method| method == ESTIMATE_SWAP_OUT_METHOD).count();

        let quote = provider.get_quote(&request).await.unwrap();
        assert_eq!(estimates(&calls), 3);
        assert_eq!(provider.get_quote(&request).await.unwrap(), quote);
        assert_eq!(estimates(&calls), 2);
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
