use std::error::Error;
use std::sync::Arc;

use cacher::{ChartsHistoryCacher, ObservedAssetsCacher, PerpetualAddressCacher, PriceMetadataCacher, SwapVaultAddressCacher, ThrottleCacher};
use chain_providers::ChainProviders;
use chrono::{TimeDelta, Utc};
use coingecko::CoinGeckoClient;
use config_keys::ConfigKey;
use prices::{FiatRatesProvider, PriceAssetsProvider, PriceProvider};
use primitives::{AccessTokenCacher, Chain, ChartTimeframe, JobConfiguration};
use search_index::SearchIndexClient;
use settings::service_user_agent;
use storage::{Database, PricesProvidersRepository};
use streamer::{StreamProducer, StreamProducerQueue};
use swapper::swapper::GemSwapper;

use crate::assets::repository::{PostgresRepository, Repository};
use crate::assets::{AssetClassificationRules, AssetRankUpdater, AssetsHasPriceUpdater, AssetsImagesUpdater, PerpetualUpdater, StakeApyUpdater, UsageRankUpdater, UsageRankUpdaterConfig, ValidatorScanner};
use crate::fiat::{FiatAssetsUpdater, FiatRatesUpdater};
use crate::notifications::{StakeRewardsConfig, StakingRewardsNotifier};
use crate::perpetuals::{PerpetualAddressRefresher, PerpetualPositionClassifier, PerpetualPositionClassifierConfig, PerpetualPositionObserver};
use crate::prices::{
    AssetsProviders, ChartsHistoryConfig, ChartsHistoryUpdater, ChartsUpdater, MarketsClient, MarketsUpdater, MissingPricesPublisher, ObservedPricesConfig, ObservedPricesUpdater, PriceAlertClient, PriceAlertSender, PriceClient,
    PricesCleanupUpdater, PricesMetricsUpdater, PricesUpdater,
};
use crate::rewards::{RewardsAbuseChecker, RewardsEligibilityChecker};
use crate::search::{AssetListsIndexUpdater, AssetsIndexUpdater, NftsIndexUpdater, PerpetualsIndexUpdater, SearchSyncClient};
use crate::system::{DeviceUpdater, InactiveDevicesObserver, TransactionCleanup, TransactionCleanupConfig, VersionUpdater};
use crate::transactions::{CheckSchedule, InTransitConfig, InTransitUpdater, PendingTransactionsUpdater, PendingTransactionsUpdaterConfig, SwapVaultAddressClient, TransactionQueue, TransactionQueueMetrics, VaultAddressesUpdater};
use crate::{ConfigCacher, Services, StaticAssetsClient};

#[derive(Clone)]
pub struct AlerterJobs {
    database: Database,
    throttle: Arc<dyn ThrottleCacher>,
    config: Arc<ConfigCacher>,
    price_alert_client: PriceAlertClient,
    chain_providers: Arc<ChainProviders>,
    stake_rewards_config: StakeRewardsConfig,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl AlerterJobs {
    pub fn price_alert_sender(&self) -> PriceAlertSender {
        PriceAlertSender::new(self.config.clone(), self.price_alert_client.clone(), self.stream_producer.clone())
    }

    pub fn staking_rewards_notifier(&self) -> StakingRewardsNotifier {
        StakingRewardsNotifier::new(self.chain_providers.clone(), self.database.clone(), self.stake_rewards_config, self.throttle.clone(), self.stream_producer.clone())
    }
}

#[derive(Clone)]
pub struct AssetsJobs {
    database: Database,
    classification_rules: AssetClassificationRules,
    usage_rank_config: UsageRankUpdaterConfig,
    static_assets_client: StaticAssetsClient,
}

impl AssetsJobs {
    pub fn asset_rank_updater(&self) -> AssetRankUpdater {
        AssetRankUpdater::new(self.assets_repository(), self.classification_rules.clone())
    }

    fn assets_repository(&self) -> Arc<dyn Repository> {
        Arc::new(PostgresRepository::new(self.database.clone()))
    }

    pub fn perpetual_updater(&self, providers: Arc<ChainProviders>) -> PerpetualUpdater {
        PerpetualUpdater::new(providers, self.database.clone())
    }

    pub fn usage_rank_updater(&self) -> UsageRankUpdater {
        UsageRankUpdater::new(self.database.clone(), self.usage_rank_config)
    }

    pub fn assets_images_updater(&self) -> AssetsImagesUpdater {
        AssetsImagesUpdater::new(self.static_assets_client.clone(), self.database.clone())
    }

    pub fn assets_has_price_updater(&self) -> AssetsHasPriceUpdater {
        AssetsHasPriceUpdater::new(self.database.clone())
    }

    pub fn stake_apy_updater(&self, providers: Arc<ChainProviders>) -> StakeApyUpdater {
        StakeApyUpdater::new(providers, self.assets_repository())
    }

    pub fn validator_scanner(&self, providers: Arc<ChainProviders>) -> ValidatorScanner {
        ValidatorScanner::new(providers, self.static_assets_client.clone(), self.assets_repository())
    }
}

#[derive(Clone)]
pub struct FiatJobs {
    services: Services,
    price_client: PriceClient,
    access_token_cacher: Arc<dyn AccessTokenCacher>,
}

impl FiatJobs {
    pub fn rates_updater(&self, provider: Arc<dyn FiatRatesProvider>) -> FiatRatesUpdater {
        FiatRatesUpdater::new(provider, self.price_client.clone())
    }

    pub fn assets_updater(&self) -> FiatAssetsUpdater {
        FiatAssetsUpdater::new(self.services.database(), self.services.fiat_providers(self.access_token_cacher.clone()))
    }
}

#[derive(Clone)]
pub struct PerpetualJobs {
    database: Database,
    addresses: Arc<dyn PerpetualAddressCacher>,
    config: Arc<ConfigCacher>,
    providers: Arc<ChainProviders>,
    classifier_config: PerpetualPositionClassifierConfig,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl PerpetualJobs {
    pub fn classifier(&self, chain: Chain) -> PerpetualPositionClassifier {
        PerpetualPositionClassifier::new(chain, self.providers.clone(), self.addresses.clone(), self.classifier_config)
    }

    pub fn observer(&self, chain: Chain) -> PerpetualPositionObserver {
        PerpetualPositionObserver::new(chain, self.providers.clone(), self.addresses.clone(), self.config.clone(), self.stream_producer.clone())
    }

    pub fn address_refresher(&self) -> PerpetualAddressRefresher {
        PerpetualAddressRefresher::new(self.providers.clone(), self.database.clone(), self.addresses.clone())
    }
}

#[derive(Clone)]
pub struct PriceJobs {
    database: Database,
    metadata_cooldowns: Arc<dyn PriceMetadataCacher>,
    observed_assets: Arc<dyn ObservedAssetsCacher>,
    charts_history: Arc<dyn ChartsHistoryCacher>,
    config: Arc<ConfigCacher>,
    price_client: PriceClient,
    markets_client: MarketsClient,
    coingecko: CoinGeckoClient,
    providers: AssetsProviders,
    enabled_providers: Vec<PriceProvider>,
    assets_producer: Arc<dyn StreamProducerQueue>,
    prices_producer: Arc<dyn StreamProducerQueue>,
}

impl PriceJobs {
    pub fn config(&self) -> Arc<ConfigCacher> {
        self.config.clone()
    }

    pub fn enabled_providers(&self) -> &[PriceProvider] {
        &self.enabled_providers
    }

    pub fn provider(&self, kind: PriceProvider) -> Arc<dyn PriceAssetsProvider> {
        self.providers[&kind].clone()
    }

    pub fn assets_updater(&self, kind: PriceProvider) -> PricesUpdater {
        PricesUpdater::new(self.provider(kind), self.database.clone(), self.price_client.clone(), self.assets_producer.clone())
    }

    pub fn prices_updater(&self, kind: PriceProvider) -> PricesUpdater {
        PricesUpdater::new(self.provider(kind), self.database.clone(), self.price_client.clone(), self.prices_producer.clone())
    }

    pub async fn publish_assets_metadata(&self, kind: PriceProvider) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.assets_updater(kind).publish_assets_metadata(self.metadata_cooldowns.as_ref(), &self.config).await
    }

    pub async fn update_observed_prices(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let observed_config = ObservedPricesConfig {
            max_assets: self.config.get_usize(ConfigKey::PriceObservedMaxAssets).await?,
            min_observers: self.config.get_usize(ConfigKey::PriceObservedMinObservers).await?,
            primary_price_max_age: self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?,
        };
        ObservedPricesUpdater::new(
            self.observed_assets.clone(),
            self.database.clone(),
            self.price_client.clone(),
            self.providers.clone(),
            self.prices_producer.clone(),
            observed_config,
        )
        .update()
        .await
    }

    pub fn missing_prices_publisher(&self) -> MissingPricesPublisher {
        MissingPricesPublisher::new(self.database.clone(), self.prices_producer.clone())
    }

    pub fn cleanup_updater(&self, kind: PriceProvider) -> PricesCleanupUpdater {
        PricesCleanupUpdater::new(self.database.clone(), self.charts_history.clone(), self.config.clone(), kind)
    }

    pub fn metrics_updater(&self, kind: PriceProvider) -> PricesMetricsUpdater {
        PricesMetricsUpdater::new(self.database.clone(), kind)
    }

    pub fn charts_history_updater(&self, kind: PriceProvider, config: ChartsHistoryConfig) -> ChartsHistoryUpdater {
        ChartsHistoryUpdater::new(self.provider(kind), self.database.clone(), self.charts_history.clone(), config)
    }

    pub fn markets_updater(&self) -> MarketsUpdater {
        MarketsUpdater::new(self.markets_client.clone(), self.coingecko.clone())
    }

    pub async fn aggregate_charts(&self, timeframe: ChartTimeframe) -> Result<usize, Box<dyn Error + Send + Sync>> {
        ChartsUpdater::new(self.price_client.clone()).aggregate_charts(timeframe).await
    }

    pub async fn delete_expired_charts(&self, timeframe: ChartTimeframe) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let retention = self.config.get_duration(charts_retention_key(timeframe)).await?;
        let before = (Utc::now() - TimeDelta::from_std(retention)?).naive_utc();
        ChartsUpdater::new(self.price_client.clone()).delete_charts(timeframe, before).await
    }
}

fn charts_retention_key(timeframe: ChartTimeframe) -> ConfigKey {
    match timeframe {
        ChartTimeframe::Raw => ConfigKey::PriceChartsRetentionRaw,
        ChartTimeframe::Hourly => ConfigKey::PriceChartsRetentionHourly,
        ChartTimeframe::Daily => ConfigKey::PriceChartsRetentionDaily,
    }
}

#[derive(Clone)]
pub struct RewardsJobs {
    database: Database,
    config: Arc<ConfigCacher>,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl RewardsJobs {
    pub fn abuse_checker(&self) -> RewardsAbuseChecker {
        RewardsAbuseChecker::new(self.database.clone(), self.config.clone(), self.stream_producer.clone())
    }

    pub fn eligibility_checker(&self) -> RewardsEligibilityChecker {
        RewardsEligibilityChecker::new(self.database.clone(), self.config.clone(), self.stream_producer.clone())
    }
}

#[derive(Clone)]
pub struct SearchJobs {
    database: Database,
    config: Arc<ConfigCacher>,
    search_index: SearchIndexClient,
}

impl SearchJobs {
    pub fn assets_index_updater(&self) -> AssetsIndexUpdater {
        AssetsIndexUpdater::new(self.database.clone(), self.config.clone(), self.sync_client())
    }

    pub fn asset_lists_index_updater(&self) -> AssetListsIndexUpdater {
        AssetListsIndexUpdater::new(self.database.clone(), self.search_index.clone())
    }

    pub fn perpetuals_index_updater(&self) -> PerpetualsIndexUpdater {
        PerpetualsIndexUpdater::new(self.database.clone(), self.sync_client())
    }

    pub fn nfts_index_updater(&self) -> NftsIndexUpdater {
        NftsIndexUpdater::new(self.database.clone(), self.sync_client())
    }

    fn sync_client(&self) -> SearchSyncClient {
        SearchSyncClient::new(self.config.clone(), self.search_index.clone())
    }
}

#[derive(Clone)]
pub struct SystemJobs {
    database: Database,
    throttle: Arc<dyn ThrottleCacher>,
    cleanup_config: TransactionCleanupConfig,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl SystemJobs {
    pub fn transaction_cleanup(&self) -> TransactionCleanup {
        TransactionCleanup::new(self.database.clone(), self.cleanup_config.clone())
    }

    pub fn device_updater(&self) -> DeviceUpdater {
        DeviceUpdater::new(self.database.clone())
    }

    pub fn inactive_devices_observer(&self) -> InactiveDevicesObserver {
        InactiveDevicesObserver::new(self.database.clone(), self.throttle.clone(), self.stream_producer.clone())
    }

    pub fn version_updater(&self) -> VersionUpdater {
        VersionUpdater::new(self.database.clone())
    }
}

#[derive(Clone)]
pub struct TransactionJobs {
    in_transit_updater: Arc<InTransitUpdater>,
    pending_updater: Arc<PendingTransactionsUpdater>,
    swapper: Arc<GemSwapper>,
    vault_addresses: Arc<dyn SwapVaultAddressCacher>,
}

impl TransactionJobs {
    pub fn in_transit_updater(&self) -> Arc<InTransitUpdater> {
        self.in_transit_updater.clone()
    }

    pub fn pending_updater(&self) -> Arc<PendingTransactionsUpdater> {
        self.pending_updater.clone()
    }

    pub fn vault_addresses_updater(&self) -> VaultAddressesUpdater {
        VaultAddressesUpdater::new(self.swapper.clone(), self.vault_addresses.clone())
    }
}

impl Services {
    pub async fn alerter_jobs(&self, stream_producer: StreamProducer) -> Result<AlerterJobs, Box<dyn Error + Send + Sync>> {
        let config = self.config();
        Ok(AlerterJobs {
            database: self.database(),
            throttle: Arc::new(self.cacher().await?),
            price_alert_client: self.price_alerts(),
            chain_providers: Arc::new(self.chain_providers(&service_user_agent("daemon", Some("stake_rewards")))),
            stake_rewards_config: StakeRewardsConfig {
                threshold: config.get_f64(ConfigKey::AlerterStakeRewardsThreshold).await?,
                lookback: config.get_duration(ConfigKey::AlerterStakeRewardsLookback).await?,
            },
            config,
            stream_producer: Arc::new(stream_producer),
        })
    }

    pub async fn assets_jobs(&self) -> Result<AssetsJobs, Box<dyn Error + Send + Sync>> {
        let config = self.config();
        Ok(AssetsJobs {
            database: self.database(),
            classification_rules: AssetClassificationRules::from_config(&config).await?,
            usage_rank_config: UsageRankUpdaterConfig {
                batch_size: config.get_usize(ConfigKey::AssetsUsageRankBatchSize).await?,
            },
            static_assets_client: StaticAssetsClient::new(&self.settings().assets.url),
        })
    }

    pub async fn fiat_jobs(&self) -> Result<FiatJobs, Box<dyn Error + Send + Sync>> {
        Ok(FiatJobs {
            services: self.clone(),
            price_client: self.prices(self.cacher().await?),
            access_token_cacher: self.fiat_access_token_cacher().await?,
        })
    }

    pub async fn perpetual_jobs(&self, stream_producer: StreamProducer) -> Result<PerpetualJobs, Box<dyn Error + Send + Sync>> {
        let config = self.config();
        Ok(PerpetualJobs {
            database: self.database(),
            addresses: Arc::new(self.cacher().await?),
            providers: Arc::new(self.chain_providers(&service_user_agent("daemon", Some("perpetual_observer")))),
            classifier_config: PerpetualPositionClassifierConfig {
                trigger_bps: config.get_i64(ConfigKey::PerpetualPriorityTriggerBps).await?,
                liquidation_bps: config.get_i64(ConfigKey::PerpetualPriorityLiquidationBps).await?,
                concurrency: config.get_usize(ConfigKey::PerpetualClassifierConcurrency).await?,
            },
            config,
            stream_producer: Arc::new(stream_producer),
        })
    }

    pub async fn price_jobs(&self, assets_producer: StreamProducer, prices_producer: StreamProducer) -> Result<PriceJobs, Box<dyn Error + Send + Sync>> {
        let database = self.database();
        let cacher = self.cacher().await?;
        let enabled_providers: Vec<PriceProvider> = database
            .run(PricesProvidersRepository::get_prices_providers)
            .await?
            .into_iter()
            .filter(|provider| provider.enabled)
            .map(|provider| provider.provider)
            .collect();
        Ok(PriceJobs {
            providers: Arc::new(self.price_providers(enabled_providers.iter().copied())),
            price_client: self.prices(cacher.clone()),
            markets_client: self.markets(cacher.clone()),
            coingecko: CoinGeckoClient::new(self.settings().prices.coingecko.remote_provider_config()),
            config: self.config(),
            database,
            metadata_cooldowns: Arc::new(cacher.clone()),
            observed_assets: Arc::new(cacher.clone()),
            charts_history: Arc::new(cacher),
            enabled_providers,
            assets_producer: Arc::new(assets_producer),
            prices_producer: Arc::new(prices_producer),
        })
    }

    pub fn rewards_jobs(&self, stream_producer: StreamProducer) -> RewardsJobs {
        RewardsJobs {
            database: self.database(),
            config: self.config(),
            stream_producer: Arc::new(stream_producer),
        }
    }

    pub async fn search_jobs(&self) -> Result<SearchJobs, Box<dyn Error + Send + Sync>> {
        let config = self.config();
        Ok(SearchJobs {
            database: self.database(),
            search_index: self.search_index().await?,
            config,
        })
    }

    pub async fn system_jobs(&self, stream_producer: StreamProducer) -> Result<SystemJobs, Box<dyn Error + Send + Sync>> {
        let config = self.config();
        Ok(SystemJobs {
            database: self.database(),
            throttle: Arc::new(self.cacher().await?),
            cleanup_config: TransactionCleanupConfig {
                address_max_count: config.get_i64(ConfigKey::TransactionCleanupAddressMaxCount).await?,
                address_limit: config.get_usize(ConfigKey::TransactionCleanupAddressLimit).await?,
                lookback: config.get_duration(ConfigKey::TransactionCleanupLookback).await?,
            },
            stream_producer: Arc::new(stream_producer),
        })
    }

    pub async fn transaction_jobs(&self, stream_producer: StreamProducer, metrics: Arc<dyn TransactionQueueMetrics>) -> Result<TransactionJobs, Box<dyn Error + Send + Sync>> {
        let config = self.config();
        let database = self.database();
        let cacher = self.cacher().await?;
        let stream_producer: Arc<dyn StreamProducerQueue> = Arc::new(stream_producer);
        let in_transit_config = InTransitConfig {
            timeout: config.get_duration(ConfigKey::TransactionInTransitTimeout).await?,
            query_limit: config.get_i64(ConfigKey::TransactionInTransitQueryLimit).await?,
            check_interval: JobConfiguration {
                initial_interval_ms: config.get_duration(ConfigKey::TransactionTimerInTransitUpdate).await?.as_millis() as u32,
                max_interval_ms: config.get_duration(ConfigKey::TransactionInTransitMaxCheckInterval).await?.as_millis() as u32,
                step_factor: config.get_f64(ConfigKey::TransactionInTransitCheckIntervalFactor).await? as f32,
            },
        };
        let pending_config = PendingTransactionsUpdaterConfig::from_config(&config).await?;
        let providers = Arc::new(self.chain_providers(&service_user_agent("daemon", Some("transactions"))));
        let swapper = self.swapper();
        let in_transit_updater = InTransitUpdater::new(
            database.clone(),
            in_transit_config,
            swapper.clone(),
            stream_producer.clone(),
            SwapVaultAddressClient::new(Arc::new(cacher.clone())),
            metrics.clone(),
            CheckSchedule::new(Arc::new(cacher.clone()), TransactionQueue::InTransit),
        );
        let pending_updater = PendingTransactionsUpdater::new(
            providers,
            Arc::new(cacher.clone()),
            stream_producer,
            database,
            pending_config,
            metrics,
            CheckSchedule::new(Arc::new(cacher.clone()), TransactionQueue::Pending),
        );
        Ok(TransactionJobs {
            in_transit_updater: Arc::new(in_transit_updater),
            pending_updater: Arc::new(pending_updater),
            swapper,
            vault_addresses: Arc::new(cacher),
        })
    }
}
