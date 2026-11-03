use crate::model::WorkerService;
use config_keys::{ConfigKey, ConfigParamKey};
use primitives::{Chain, FiatProviderName, FiatRateProvider, ListProviderName, OptionStringExt, PlatformStore, PriceProvider};
use services::ConfigCacher;
use std::error::Error;
use std::time::Duration;
use strum::AsRefStr;

#[derive(Clone, Debug)]
enum JobInterval {
    Config(ConfigKey),
    Provider,
}

#[derive(Clone, Debug)]
struct JobSpec {
    worker: WorkerService,
    interval: JobInterval,
}

impl JobSpec {
    const fn new(worker: WorkerService, interval: JobInterval) -> Self {
        Self { worker, interval }
    }
}

pub trait JobLabel {
    fn job_label(&self) -> String;
}

impl JobLabel for str {
    fn job_label(&self) -> String {
        self.to_string()
    }
}

impl JobLabel for String {
    fn job_label(&self) -> String {
        self.clone()
    }
}

impl<T> JobLabel for &T
where
    T: JobLabel + ?Sized,
{
    fn job_label(&self) -> String {
        (*self).job_label()
    }
}

impl JobLabel for Chain {
    fn job_label(&self) -> String {
        self.as_ref().to_string()
    }
}

impl JobLabel for FiatProviderName {
    fn job_label(&self) -> String {
        self.as_ref().to_string()
    }
}

impl JobLabel for PlatformStore {
    fn job_label(&self) -> String {
        self.as_ref().to_string()
    }
}

impl JobLabel for primitives::SwapProvider {
    fn job_label(&self) -> String {
        self.as_ref().to_string()
    }
}

impl JobLabel for PriceProvider {
    fn job_label(&self) -> String {
        self.id().to_string()
    }
}

impl JobLabel for FiatRateProvider {
    fn job_label(&self) -> String {
        self.as_ref().to_string()
    }
}

impl JobLabel for ListProviderName {
    fn job_label(&self) -> String {
        self.id().to_string()
    }
}

fn compose_job_name(base: &str, label: Option<&str>) -> String {
    match label.map(str::trim).non_empty() {
        Some(suffix) => format!("{base}.{suffix}"),
        None => base.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum WorkerJob {
    AlertPriceAlerts,
    UpdateSuspiciousAssetRanks,
    UpdateStakeApy,
    UpdatePerpetuals,
    UpdateUsageRanks,
    UpdateAssetsImages,
    UpdateAssetsHasPrice,
    CleanupStaleDeviceSubscriptions,
    ObserveInactiveDevices,
    UpdateFiatAssets,
    UpdateFiatProviderCountries,
    UpdateFiatBuyableAssets,
    UpdateFiatSellableAssets,
    UpdateTrendingFiatAssets,
    UpdateAssetsIndex,
    UpdateAssetListsIndex,
    UpdatePerpetualsIndex,
    UpdateNftsIndex,
    CleanupProcessedTransactions,
    UpdateStoreVersion,
    UpdateChainValidators,
    UpdateValidatorsFromStaticAssets,
    CheckRewardsAbuse,
    CheckRewardsEligibility,
    CleanupOutdatedAssets,
    UpdateFiatRates,
    UpdatePricesTop,
    UpdatePricesHigh,
    UpdatePricesLow,
    UpdatePrices,
    AggregateHourlyCharts,
    AggregateDailyCharts,
    CleanupChartsRaw,
    CleanupChartsHourly,
    UpdateMarkets,
    UpdateChartsHistory,
    UpdateObservedPrices,
    UpdatePricesAssets,
    UpdatePricesAssetsNew,
    PublishPricesAssetsMetadata,
    PublishMissingPrices,
    UpdateInTransitTransactions,
    UpdatePendingTransactions,
    UpdateSwapVaultAddresses,
    AlertStakeRewards,
    ClassifyPerpetualAddresses,
    ObservePerpetualActiveAddresses,
    ObservePerpetualPriorityAddresses,
    RefreshPerpetualTrackedAddresses,
    UpdateLists,
}

impl WorkerJob {
    fn spec(&self) -> JobSpec {
        use WorkerJob::*;
        match self {
            AlertPriceAlerts => JobSpec::new(WorkerService::Alerter, JobInterval::Config(ConfigKey::AlerterPriceAlertsTimer)),
            UpdateSuspiciousAssetRanks => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateSuspicious)),
            UpdateStakeApy => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateStakeApy)),
            UpdatePerpetuals => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdatePerpetuals)),
            UpdateUsageRanks => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateUsageRank)),
            UpdateAssetsImages => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateImages)),
            UpdateAssetsHasPrice => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateHasPrice)),
            CleanupStaleDeviceSubscriptions => JobSpec::new(WorkerService::System, JobInterval::Config(ConfigKey::DeviceTimerUpdater)),
            ObserveInactiveDevices => JobSpec::new(WorkerService::System, JobInterval::Config(ConfigKey::DeviceTimerInactiveObserver)),
            UpdateFiatAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateAssets)),
            UpdateFiatProviderCountries => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateProviderCountries)),
            UpdateFiatBuyableAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateBuyableAssets)),
            UpdateFiatSellableAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateSellableAssets)),
            UpdateTrendingFiatAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateTrending)),
            UpdateAssetsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchAssetsUpdateInterval)),
            UpdateAssetListsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchAssetListsUpdateInterval)),
            UpdatePerpetualsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchPerpetualsUpdateInterval)),
            UpdateNftsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchNftsUpdateInterval)),
            CleanupProcessedTransactions => JobSpec::new(WorkerService::System, JobInterval::Config(ConfigKey::TransactionTimerCleanup)),
            UpdateStoreVersion => JobSpec::new(WorkerService::System, JobInterval::Config(ConfigKey::VersionTimerUpdateStoreVersions)),
            UpdateChainValidators => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::ScanTimerUpdateValidators)),
            UpdateValidatorsFromStaticAssets => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::ScanTimerUpdateValidatorsStatic)),
            CheckRewardsAbuse => JobSpec::new(WorkerService::Rewards, JobInterval::Config(ConfigKey::RewardsTimerAbuseChecker)),
            CheckRewardsEligibility => JobSpec::new(WorkerService::Rewards, JobInterval::Config(ConfigKey::RewardsTimerEligibilityChecker)),
            CleanupOutdatedAssets => JobSpec::new(WorkerService::Prices, JobInterval::Provider),
            UpdateFiatRates => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::PriceTimerFiatRates)),
            UpdatePricesTop => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerTopMarketCap)),
            UpdatePricesHigh => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerHighMarketCap)),
            UpdatePricesLow => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerLowMarketCap)),
            UpdatePrices => JobSpec::new(WorkerService::Prices, JobInterval::Provider),
            AggregateHourlyCharts => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerChartsHourly)),
            AggregateDailyCharts => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerChartsDaily)),
            CleanupChartsRaw => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerCleanupChartsRaw)),
            CleanupChartsHourly => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerCleanupChartsHourly)),
            UpdateMarkets => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerMarkets)),
            UpdateChartsHistory => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerChartsHistory)),
            UpdateObservedPrices => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceObservedFetchInterval)),
            UpdatePricesAssets => JobSpec::new(WorkerService::Prices, JobInterval::Provider),
            UpdatePricesAssetsNew => JobSpec::new(WorkerService::Prices, JobInterval::Provider),
            PublishPricesAssetsMetadata => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceMetadataPublishInterval)),
            PublishMissingPrices => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceMissingPublishInterval)),
            UpdateInTransitTransactions => JobSpec::new(WorkerService::Transactions, JobInterval::Config(ConfigKey::TransactionTimerInTransitUpdate)),
            UpdatePendingTransactions => JobSpec::new(WorkerService::Transactions, JobInterval::Config(ConfigKey::TransactionPendingTimer)),
            UpdateSwapVaultAddresses => JobSpec::new(WorkerService::Transactions, JobInterval::Provider),
            AlertStakeRewards => JobSpec::new(WorkerService::Alerter, JobInterval::Config(ConfigKey::AlerterStakeRewardsTimer)),
            ClassifyPerpetualAddresses => JobSpec::new(WorkerService::Perpetuals, JobInterval::Config(ConfigKey::PerpetualClassifierInterval)),
            ObservePerpetualActiveAddresses => JobSpec::new(WorkerService::Perpetuals, JobInterval::Config(ConfigKey::PerpetualObserverInterval)),
            ObservePerpetualPriorityAddresses => JobSpec::new(WorkerService::Perpetuals, JobInterval::Config(ConfigKey::PerpetualPriorityObserverInterval)),
            RefreshPerpetualTrackedAddresses => JobSpec::new(WorkerService::Perpetuals, JobInterval::Config(ConfigKey::PerpetualAddressRefreshInterval)),
            UpdateLists => JobSpec::new(WorkerService::Lists, JobInterval::Provider),
        }
    }

    pub fn worker(&self) -> WorkerService {
        self.spec().worker
    }

    fn interval(&self) -> JobInterval {
        self.spec().interval
    }
}

#[derive(Clone, Debug)]
pub struct JobVariant {
    job: WorkerJob,
    label: Option<String>,
    provider_interval: Option<ConfigParamKey>,
}

impl JobVariant {
    pub fn new(job: WorkerJob) -> Self {
        Self { job, label: None, provider_interval: None }
    }

    pub fn labeled(job: WorkerJob, label: impl JobLabel) -> Self {
        Self {
            job,
            label: Some(label.job_label()),
            provider_interval: None,
        }
    }

    pub fn provider_interval(mut self, key: ConfigParamKey) -> Self {
        self.provider_interval = Some(key);
        self
    }

    pub fn name(&self) -> String {
        job_name(self.job, self.label.as_deref())
    }

    pub fn worker(&self) -> WorkerService {
        self.job.worker()
    }

    pub async fn interval_duration(&self, config: &ConfigCacher) -> Result<Duration, Box<dyn Error + Send + Sync>> {
        match (self.job.interval(), &self.provider_interval) {
            (JobInterval::Config(key), None) => Ok(config.get_duration(key).await?),
            (JobInterval::Provider, Some(key)) => Ok(config.get_param_duration(key).await?),
            (JobInterval::Config(_), Some(_)) => Err(format!("job {} has a global interval and takes no provider interval", self.name()).into()),
            (JobInterval::Provider, None) => Err(format!("job {} needs a provider interval", self.name()).into()),
        }
    }
}

impl From<WorkerJob> for JobVariant {
    fn from(job: WorkerJob) -> Self {
        JobVariant::new(job)
    }
}

pub fn job_name(job: WorkerJob, label: Option<&str>) -> String {
    compose_job_name(job.as_ref(), label)
}
