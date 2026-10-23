use std::sync::{Arc, Mutex};

use num_bigint::{BigInt, BigUint};
use primitives::{Asset, AssetData, AssetId, Balance, Chain, RecentActivityType, WalletId};
use swapper::testkit::MockSwapper;
use swapper::{FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData};

use super::GemSwapService;
use super::model::{GemSwapButtonInput, GemSwapPair};
use super::quote::GemSwapQuoteService;
use super::session::{GemSwapQuoteInput, GemSwapQuotesResult, GemSwapRequest, GemSwapSession};
use super::store::GemSwapStore;
use crate::gem_swapper::GemSwapper;
use crate::services::asset_discovery::testkit::DiscoveryTestkit;
use crate::services::assets::GemAssetFilter;
use crate::services::config::GemConfigService;
use crate::services::error::GemServiceError;
use crate::services::node::GemNodeService;
use crate::services::stream::testkit::{MemoryStreamConnection, SubscriptionTestkit};
use crate::testkit::TestAlienProvider;
use primitives::Wallet;

#[derive(Default)]
pub struct MemorySwapStore {
    pub pairs: Mutex<Vec<GemSwapPair>>,
    pub recent_asset_ids: Mutex<Vec<AssetId>>,
    pub asset_ids: Mutex<Vec<AssetId>>,
    pub recent_requests: Mutex<Vec<(Vec<RecentActivityType>, Vec<GemAssetFilter>)>>,
    pub asset_requests: Mutex<Vec<Vec<GemAssetFilter>>>,
    pub limits: Mutex<Vec<u32>>,
}

#[async_trait::async_trait]
impl GemSwapStore for MemorySwapStore {
    async fn get_swap_pairs(&self, _wallet_id: WalletId) -> Result<Vec<GemSwapPair>, GemServiceError> {
        Ok(self.pairs.lock().unwrap().clone())
    }

    async fn get_recent_asset_ids(&self, _wallet_id: WalletId, types: Vec<RecentActivityType>, filters: Vec<GemAssetFilter>, limit: u32) -> Result<Vec<AssetId>, GemServiceError> {
        self.limits.lock().unwrap().push(limit);
        self.recent_requests.lock().unwrap().push((types, filters));
        Ok(self.recent_asset_ids.lock().unwrap().clone())
    }

    async fn get_asset_ids(&self, _wallet_id: WalletId, filters: Vec<GemAssetFilter>, limit: u32) -> Result<Vec<AssetId>, GemServiceError> {
        self.limits.lock().unwrap().push(limit);
        self.asset_requests.lock().unwrap().push(filters);
        Ok(self.asset_ids.lock().unwrap().clone())
    }
}

impl GemSwapService {
    pub fn mock_with_swappers(swappers: Vec<Box<dyn swapper::Swapper>>) -> Self {
        Self::new(Arc::new(GemSwapper::mock(swappers)), Arc::new(MemorySwapStore::default()))
    }

    pub fn mock(store: Arc<MemorySwapStore>) -> Self {
        Self::new(Arc::new(GemSwapper::new(Arc::new(TestAlienProvider::with_status(200)), Arc::new(GemNodeService::mock()))), store)
    }
}

impl GemSwapPair {
    pub fn mock(from: Chain, to: Chain) -> Self {
        Self {
            from_asset_id: AssetId::from_chain(from),
            to_asset_id: AssetId::from_chain(to),
        }
    }
}

impl GemSwapRequest {
    pub fn mock() -> Self {
        Self {
            pay_asset_id: AssetId::from_chain(Chain::Ethereum),
            receive_asset_id: AssetId::from_chain(Chain::Solana),
            value: BigUint::from(100u32),
            slippage_bps: None,
        }
    }
}

impl GemSwapButtonInput {
    pub fn mock(value: u64, available_balance: u64) -> Self {
        Self {
            value: BigInt::from(value),
            available_balance: BigInt::from(available_balance),
            quote_error: None,
        }
    }
}

impl GemSwapQuotesResult {
    pub fn mock(quotes: Vec<Quote>) -> Self {
        Self {
            request: GemSwapRequest::mock(),
            quotes,
            error: None,
        }
    }
}

pub fn mock_asset_data(chain: Chain, available: u64) -> AssetData {
    AssetData::mock(Asset::from_chain(chain), Balance::coin_balance(available.into()))
}

impl GemSwapSession {
    pub fn mock_ready() -> Self {
        let session = Self::default()
            .on_request_changed(Some(GemSwapRequest::mock()))
            .on_quote_results(GemSwapQuotesResult::mock(vec![Quote::mock_with_provider(SwapperProvider::Okx, "10"), Quote::mock_with_provider(SwapperProvider::Jupiter, "9")]));
        Self {
            input: Some(GemSwapQuoteInput {
                request: GemSwapRequest::mock(),
                use_max_amount: false,
            }),
            ..session
        }
    }
}

pub struct SwapQuoteTestkit {
    pub service: GemSwapQuoteService,
    pub discovery: DiscoveryTestkit,
    pub connection: Arc<MemoryStreamConnection>,
}

impl SwapQuoteTestkit {
    pub fn with_status(status: u16) -> Self {
        Self::new(status, GemSwapService::mock(Arc::new(MemorySwapStore::default())))
    }

    pub fn with_swapper(swapper: MockSwapper, assets: Vec<Asset>) -> Self {
        let testkit = Self::new(200, GemSwapService::mock_with_swappers(vec![Box::new(swapper)]));
        *testkit.discovery.asset_store.assets.lock().unwrap() = assets.iter().map(Asset::as_basic_primitive).collect();
        testkit
    }

    fn new(status: u16, swap: GemSwapService) -> Self {
        let provider = Arc::new(TestAlienProvider::with_status(status));
        let discovery = DiscoveryTestkit::with_provider(provider.clone(), Wallet::mock());
        let subscription = SubscriptionTestkit::new(&[], &[]);
        let connection = subscription.connection.clone();
        let service = GemSwapQuoteService::new(
            Arc::new(swap),
            discovery.assets.clone(),
            discovery.preferences.clone(),
            discovery.balance.clone(),
            Arc::new(subscription.service),
            discovery.session.clone(),
            Arc::new(GemConfigService::mock(provider)),
        );
        Self { service, discovery, connection }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StubQuote {
    Silent,
    Panics,
}

#[derive(Debug)]
pub struct StubSwapper {
    provider: ProviderType,
    quote: StubQuote,
}

impl StubSwapper {
    pub fn new(provider: SwapperProvider, quote: StubQuote) -> Self {
        Self {
            provider: ProviderType::new(provider),
            quote,
        }
    }
}

#[async_trait::async_trait]
impl Swapper for StubSwapper {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Ethereum)]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, _request: &QuoteRequest) -> Result<Quote, SwapperError> {
        match self.quote {
            StubQuote::Silent => futures::future::pending().await,
            StubQuote::Panics => panic!("provider failed"),
        }
    }

    async fn get_quote_data(&self, _quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        Err(SwapperError::NoQuoteAvailable)
    }
}
