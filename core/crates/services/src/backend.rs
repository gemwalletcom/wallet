use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;

use cacher::{AccessTokenCacherClient, CacherClient};
use chain_providers::{ChainProviders, ProviderFactory};
use coingecko::CoinGeckoClient;
use config_keys::ConfigKey;
use defi::{DefiProviderClient, DefiProviderConfig};
use fiat::{FiatProvider, FiatProviderFactory};
use gem_client::ReqwestClient;
use gem_evm::rpc::{EthereumClient, EthereumProvider};
use gem_jsonrpc::JsonRpcClient;
use lists::CoinGeckoListProvider;
use nft::NFTProviderConfig;
use primitives::{AccessTokenCacher, Chain, ChainType, EVMChain, FiatProviderName};
use pusher::PusherClient;
use rewards::{AbuseIPDBClient, EvmClientProvider, IpApiClient, IpCheckProvider, TransferRedemptionService, WalletConfig};
use search_index::{SearchIndexClient, SearchIndexConfig};
use security::TransactionScanProviders;
use settings::Settings;
use storage::{Database, DatabaseError};
use streamer::{Retry, ShutdownReceiver, StreamProducer, StreamProducerConfig};
use tokio::sync::OnceCell;

use crate::access::AccessClient;
use crate::app::ConfigClient;
use crate::assets::ListsClient;
use crate::assets::{AssetsClient, SearchClient};
use crate::auth::AuthClient;
use crate::chain::{ChainClient, FeeEstimatesClient, NodesStatusClient};
use crate::config::ConfigCacher;
use crate::defi::DefiClient;
use crate::devices::DeviceStreamClient;
use crate::devices::{DevicesClient, WalletConfigurationClient, WalletsClient};
use crate::fiat::FiatClient;
use crate::indexer::IndexerClient;
use crate::nft::NFTClient;
use crate::notifications::NotificationsClient;
use crate::prices::PortfolioClient;
use crate::prices::{ChartClient, MarketsClient, PriceAlertClient, PriceClient};
use crate::rewards::IpSecurityClient;
use crate::rewards::{RewardsClient, RewardsRedemptionClient};
use crate::security::{ScanClient, ScanMetrics, TransactionScanConfig, scan_providers};
use crate::support::SupportApiClient;
use crate::support::SupportClient;
use crate::swap::{NearIntentsProxyClient, SwapClient, SwapsXyzProxyClient};
use crate::transactions::{AddressNamesClient, TransactionsClient};
use crate::webhooks::WebhooksClient;

#[derive(Clone)]
pub struct Services {
    settings: Arc<Settings>,
    database: Database,
    config: Arc<ConfigCacher>,
    cacher: Arc<OnceCell<CacherClient>>,
}

impl Services {
    pub fn new(settings: Arc<Settings>) -> Result<Self, DatabaseError> {
        let database = Database::new(&settings.postgres.url, settings.postgres.pool)?;
        let config = Arc::new(ConfigCacher::new(database.clone()));
        Ok(Self {
            settings,
            database,
            config,
            cacher: Arc::new(OnceCell::new()),
        })
    }

    pub fn settings(&self) -> Arc<Settings> {
        self.settings.clone()
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn config(&self) -> Arc<ConfigCacher> {
        self.config.clone()
    }

    pub async fn cacher(&self) -> Result<CacherClient, Box<dyn Error + Send + Sync>> {
        Ok(self.cacher.get_or_try_init(|| CacherClient::new(&self.settings.redis.url)).await?.clone())
    }

    pub async fn auth(&self) -> Result<AuthClient, Box<dyn Error + Send + Sync>> {
        Ok(AuthClient::new(self.cacher().await?))
    }

    pub async fn stream_producer(&self, name: &str, shutdown_rx: ShutdownReceiver) -> Result<StreamProducer, Box<dyn Error + Send + Sync>> {
        let rabbitmq = &self.settings.rabbitmq;
        let config = StreamProducerConfig::new(rabbitmq.url.clone(), Retry::new(rabbitmq.retry.delay, rabbitmq.retry.timeout));
        StreamProducer::new(&config, name, shutdown_rx).await
    }

    pub async fn support(&self, shutdown_rx: ShutdownReceiver) -> Result<SupportClient, Box<dyn Error + Send + Sync>> {
        let stream_producer = self.stream_producer("daemon_support_producer", shutdown_rx).await?;
        Ok(SupportClient::new(self.database(), stream_producer, self.cacher().await?))
    }

    pub async fn search_index(&self) -> Result<SearchIndexClient, Box<dyn Error + Send + Sync>> {
        let config = SearchIndexConfig {
            batch_size: self.config().get_usize(ConfigKey::SearchIndexBatchSize).await?,
        };
        Ok(SearchIndexClient::new(&self.settings.meilisearch.url, &self.settings.meilisearch.key, config))
    }

    pub fn defi(&self) -> DefiClient {
        DefiClient::new(self.database(), DefiProviderClient::new(DefiProviderConfig::from_settings(&self.settings)))
    }

    pub async fn fiat(&self, stream_producer: StreamProducer) -> Result<FiatClient, Box<dyn Error + Send + Sync>> {
        let cacher = self.cacher().await?;
        let providers = self.fiat_providers(fiat_access_token_cacher(cacher.clone()));
        Ok(FiatClient::new(self.database(), self.config(), cacher, providers, FiatProviderFactory::new_ip_check_client(&self.settings), stream_producer))
    }

    pub async fn fiat_access_token_cacher(&self) -> Result<Arc<dyn AccessTokenCacher>, Box<dyn Error + Send + Sync>> {
        Ok(fiat_access_token_cacher(self.cacher().await?))
    }

    pub fn fiat_providers(&self, access_token_cacher: Arc<dyn AccessTokenCacher>) -> Vec<Box<dyn FiatProvider + Send + Sync>> {
        FiatProviderFactory::new_providers(&self.settings, access_token_cacher)
    }

    pub fn lists(&self) -> ListsClient {
        let coingecko = CoinGeckoClient::new(self.settings.coingecko.remote_provider_config());
        ListsClient::new(self.database(), vec![Arc::new(CoinGeckoListProvider::new(coingecko))])
    }

    pub fn nft(&self) -> NFTClient {
        NFTClient::from_config(self.database(), NFTProviderConfig::from_settings(&self.settings), self.settings.nft.url.clone())
    }

    pub fn prices(&self, cacher: CacherClient) -> PriceClient {
        PriceClient::new(self.database(), self.config(), cacher)
    }

    pub fn charts(&self) -> ChartClient {
        ChartClient::new(self.database(), self.config())
    }

    pub fn markets(&self, cacher: CacherClient) -> MarketsClient {
        MarketsClient::new(self.database(), cacher)
    }

    pub fn price_alerts(&self) -> PriceAlertClient {
        PriceAlertClient::new(self.database())
    }

    pub async fn ip_security(&self) -> Result<IpSecurityClient, Box<dyn Error + Send + Sync>> {
        let security = &self.settings.security;
        let providers: Vec<Arc<dyn IpCheckProvider>> = vec![
            Arc::new(AbuseIPDBClient::new(security.abuseipdb.url.clone(), security.abuseipdb.key.secret.clone())),
            Arc::new(IpApiClient::new(security.ipapi.url.clone(), security.ipapi.key.secret.clone())),
        ];
        Ok(IpSecurityClient::new(providers, self.cacher().await?))
    }

    pub fn redemption_service(&self) -> Result<TransferRedemptionService, Box<dyn Error + Send + Sync>> {
        let wallets = self
            .settings
            .rewards
            .wallets
            .iter()
            .map(|(chain_type, wallet)| {
                let chain_type = ChainType::from_str(chain_type).map_err(|_| format!("Invalid chain type: {chain_type}"))?;
                Ok((
                    chain_type,
                    WalletConfig {
                        key: wallet.key.clone(),
                        address: wallet.address.clone(),
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;
        let settings = self.settings.clone();
        let client_provider: EvmClientProvider = Arc::new(move |chain: EVMChain| {
            let client = ReqwestClient::new(ProviderFactory::get_chain_url(chain.to_chain(), &settings), gem_client::builder().build().ok()?);
            Some(EthereumProvider::new_rpc_only(EthereumClient::new(JsonRpcClient::new(client), chain)))
        });
        Ok(TransferRedemptionService::new(wallets, client_provider))
    }

    pub fn pusher(&self) -> PusherClient {
        PusherClient::new(self.settings.pusher.url.clone(), self.settings.pusher.ios.topic.clone())
    }

    pub fn chain_providers(&self, user_agent: &str) -> ChainProviders {
        ChainProviders::from_settings(&self.settings, user_agent)
    }

    pub fn chain_providers_for(&self, chain: Chain, user_agent: &str) -> ChainProviders {
        ChainProviders::for_chain(chain, &self.settings, user_agent)
    }

    pub fn assets(&self) -> AssetsClient {
        AssetsClient::new(self.database(), self.config())
    }

    pub async fn search(&self, price_client: PriceClient) -> Result<SearchClient, Box<dyn Error + Send + Sync>> {
        Ok(SearchClient::new(self.search_index().await?, price_client))
    }

    pub fn devices(&self) -> DevicesClient {
        DevicesClient::new(self.database(), self.pusher())
    }

    pub fn wallets(&self, stream_producer: StreamProducer) -> WalletsClient {
        WalletsClient::new(self.database(), stream_producer)
    }

    pub fn wallet_configuration(&self, cacher: CacherClient, user_agent: &str) -> WalletConfigurationClient {
        WalletConfigurationClient::new(self.database(), self.chain_providers(user_agent), cacher)
    }

    pub fn notifications(&self) -> NotificationsClient {
        NotificationsClient::new(self.database())
    }

    pub fn rewards(&self, cacher: CacherClient, stream_producer: StreamProducer, ip_security: IpSecurityClient) -> RewardsClient {
        RewardsClient::new(self.database(), self.config(), cacher, stream_producer, ip_security, self.pusher())
    }

    pub fn rewards_redemption(&self, stream_producer: StreamProducer) -> RewardsRedemptionClient {
        RewardsRedemptionClient::new(self.database(), self.config(), stream_producer)
    }

    pub fn portfolio(&self) -> PortfolioClient {
        PortfolioClient::new(self.database(), self.config())
    }

    pub fn transactions(&self) -> TransactionsClient {
        TransactionsClient::new(self.database())
    }

    pub fn address_names(&self) -> AddressNamesClient {
        AddressNamesClient::new(self.database())
    }

    pub fn indexer(&self, cacher: CacherClient, stream_producer: StreamProducer) -> IndexerClient {
        IndexerClient::new(self.database(), cacher, stream_producer)
    }

    pub fn access(&self) -> AccessClient {
        AccessClient::new(self.database())
    }

    pub fn webhooks(&self, stream_producer: StreamProducer) -> WebhooksClient {
        WebhooksClient::new(stream_producer, self.settings.support.webhook.key.secret.clone())
    }

    pub fn app_config(&self) -> ConfigClient {
        ConfigClient::new(self.database())
    }

    pub fn chain(&self, user_agent: &str) -> ChainClient {
        ChainClient::new(self.chain_providers(user_agent))
    }

    pub fn fee_estimates(&self, assets: AssetsClient, prices: PriceClient, cacher: CacherClient, user_agent: &str) -> FeeEstimatesClient {
        FeeEstimatesClient::new(self.chain(user_agent), assets, prices, cacher)
    }

    pub fn nodes_status(&self) -> NodesStatusClient {
        NodesStatusClient::default()
    }

    pub async fn scan_providers(&self, cacher: CacherClient) -> Result<TransactionScanProviders, Box<dyn Error + Send + Sync>> {
        scan_providers(&self.settings, cacher, self.config().get_duration(ConfigKey::ScanTimeout).await?)
    }

    pub async fn scan(&self, providers: TransactionScanProviders, cacher: CacherClient, metrics: Arc<dyn ScanMetrics>) -> Result<ScanClient, Box<dyn Error + Send + Sync>> {
        let config = TransactionScanConfig {
            providers,
            required_successes: self.config().get_usize(ConfigKey::ScanRequiredSuccesses).await?,
        };
        Ok(ScanClient::new(self.database(), self.config(), cacher, config, metrics))
    }

    pub fn support_api(&self) -> SupportApiClient {
        let support = &self.settings.support;
        SupportApiClient::new(support.url.clone(), support.widget.ios.clone(), support.widget.android.clone(), self.database())
    }

    pub async fn device_stream(&self, cacher: CacherClient) -> Result<DeviceStreamClient, Box<dyn Error + Send + Sync>> {
        let config = self.config();
        Ok(DeviceStreamClient::new(
            cacher,
            config.get_duration(ConfigKey::DeviceStreamRetention).await?,
            config.get_usize(ConfigKey::DeviceStreamHistoryLimit).await?,
        ))
    }

    pub fn swap(&self) -> SwapClient {
        SwapClient::new(self.database())
    }

    pub fn near_intents(&self, cacher: CacherClient) -> NearIntentsProxyClient {
        NearIntentsProxyClient::new(self.settings.swap.nearintents.url.clone(), cacher)
    }

    pub fn swaps_xyz(&self, cacher: CacherClient) -> SwapsXyzProxyClient {
        SwapsXyzProxyClient::new(self.settings.swap.swapsxyz.url.clone(), cacher)
    }
}

fn fiat_access_token_cacher(cacher: CacherClient) -> Arc<dyn AccessTokenCacher> {
    Arc::new(AccessTokenCacherClient::new(cacher, FiatProviderName::Transak.id()))
}
