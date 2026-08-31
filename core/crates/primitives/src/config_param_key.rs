use crate::duration::{DAY, HOUR, MINUTE, WEEK};
use crate::{Chain, ListProviderName, PriceProvider, SwapProvider};
use std::time::Duration;
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr)]
#[strum(serialize_all = "camelCase")]
pub enum RateLimitWindow {
    Minute,
    Hour,
    Day,
    Week,
}

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter)]
#[strum(serialize_all = "camelCase")]
pub enum RateLimitKey {
    // Fiat
    FiatQuoteRequestPerDeviceLimit,
    FiatQuoteRequestPerIpLimit,
    FiatQuoteUrlRequestPerDeviceLimit,
    FiatQuoteUrlRequestPerIpLimit,

    // Rewards username creation
    UsernameCreationGlobalLimit,
    UsernameCreationPerCountryLimit,
    UsernameCreationPerDeviceLimit,
    UsernameCreationPerIpLimit,

    // Rewards referrals
    ReferralGlobalLimit,
    ReferralPerCountryLimit,
    ReferralPerDeviceLimit,
    ReferralPerIpLimit,
    ReferralPerUserLimit,

    // Rewards redemptions
    RedemptionPerUserLimit,
}

impl RateLimitKey {
    fn default_limit(self) -> RateLimit {
        match self {
            Self::FiatQuoteRequestPerDeviceLimit => RateLimit::scaled(30),
            Self::FiatQuoteRequestPerIpLimit => RateLimit::scaled(300),
            Self::FiatQuoteUrlRequestPerDeviceLimit => RateLimit::scaled(20),
            Self::FiatQuoteUrlRequestPerIpLimit => RateLimit::scaled(100),
            Self::UsernameCreationGlobalLimit => RateLimit::daily(1000),
            Self::UsernameCreationPerCountryLimit => RateLimit::daily(100),
            Self::UsernameCreationPerDeviceLimit => RateLimit::flat(5),
            Self::UsernameCreationPerIpLimit => RateLimit::flat(10),
            Self::ReferralGlobalLimit => RateLimit::daily(1000),
            Self::ReferralPerCountryLimit => RateLimit::daily(100),
            Self::ReferralPerDeviceLimit => RateLimit::daily(1),
            Self::ReferralPerIpLimit => RateLimit::new(3, 3, 3, 10),
            Self::ReferralPerUserLimit => RateLimit::new(2, 2, 5, 15),
            Self::RedemptionPerUserLimit => RateLimit::new(1, 1, 1, 3),
        }
    }
}

impl RateLimitWindow {
    pub const ALL: [Self; 4] = [Self::Minute, Self::Hour, Self::Day, Self::Week];

    fn multiplier(self) -> i64 {
        match self {
            Self::Minute => 1,
            Self::Hour => 5,
            Self::Day => 25,
            Self::Week => 100,
        }
    }

    pub fn duration(self) -> Duration {
        match self {
            Self::Minute => MINUTE,
            Self::Hour => HOUR,
            Self::Day => DAY,
            Self::Week => WEEK,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimit {
    minute: i64,
    hour: i64,
    day: i64,
    week: i64,
}

impl RateLimit {
    pub fn new(minute: i64, hour: i64, day: i64, week: i64) -> Self {
        Self { minute, hour, day, week }
    }

    fn scaled(limit: i64) -> Self {
        let [minute, hour, day, week] = RateLimitWindow::ALL.map(|window| limit * window.multiplier());
        Self::new(minute, hour, day, week)
    }

    fn flat(limit: i64) -> Self {
        Self::new(limit, limit, limit, limit)
    }

    fn daily(limit: i64) -> Self {
        Self::new(limit, limit, limit, limit * 7)
    }

    pub fn get(self, window: RateLimitWindow) -> i64 {
        match window {
            RateLimitWindow::Minute => self.minute,
            RateLimitWindow::Hour => self.hour,
            RateLimitWindow::Day => self.day,
            RateLimitWindow::Week => self.week,
        }
    }
}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "camelCase")]
pub enum ConfigParamKey {
    TransactionsRequestLimit(Chain),
    TransactionsPendingErrorMaxAge(Chain),
    SwapperVaultAddresses(SwapProvider),
    PriceProviderAssetsLimit(PriceProvider),
    PriceProviderAssetsDuration(PriceProvider),
    PriceProviderAssetsNewDuration(PriceProvider),
    PriceProviderAssetsMetadataDuration(PriceProvider),
    PriceProviderPricesDuration(PriceProvider),
    PriceProviderChartsHourlyDuration(PriceProvider),
    PriceProviderMetricsDuration(PriceProvider),
    PriceProviderCleanOutdatedDuration(PriceProvider),
    ListProviderUpdateDuration(ListProviderName),
    RateLimit(RateLimitKey, RateLimitWindow),
}

impl ConfigParamKey {
    pub fn all() -> Vec<Self> {
        let transactions = Chain::all().into_iter().map(Self::TransactionsRequestLimit);
        let pending_transactions = Chain::all().into_iter().map(Self::TransactionsPendingErrorMaxAge);
        let swapper = SwapProvider::cross_chain_providers().into_iter().map(Self::SwapperVaultAddresses);
        let assets_limit = PriceProvider::all().into_iter().map(Self::PriceProviderAssetsLimit);
        let assets = PriceProvider::all().into_iter().map(Self::PriceProviderAssetsDuration);
        let assets_new = PriceProvider::all().into_iter().map(Self::PriceProviderAssetsNewDuration);
        let assets_metadata = PriceProvider::all().into_iter().map(Self::PriceProviderAssetsMetadataDuration);
        let prices = PriceProvider::all().into_iter().map(Self::PriceProviderPricesDuration);
        let charts_hourly = PriceProvider::all().into_iter().map(Self::PriceProviderChartsHourlyDuration);
        let metrics = PriceProvider::all().into_iter().map(Self::PriceProviderMetricsDuration);
        let clean_outdated = PriceProvider::all().into_iter().map(Self::PriceProviderCleanOutdatedDuration);
        let lists = ListProviderName::all().into_iter().map(Self::ListProviderUpdateDuration);
        let rate_limits = RateLimitKey::iter().flat_map(|key| RateLimitWindow::ALL.into_iter().map(move |window| Self::RateLimit(key, window)));
        transactions
            .chain(pending_transactions)
            .chain(swapper)
            .chain(assets_limit)
            .chain(assets)
            .chain(assets_new)
            .chain(assets_metadata)
            .chain(prices)
            .chain(charts_hourly)
            .chain(metrics)
            .chain(clean_outdated)
            .chain(lists)
            .chain(rate_limits)
            .collect()
    }

    pub fn key(&self) -> String {
        match self {
            Self::TransactionsRequestLimit(chain) => format!("{}.{}", self.as_ref(), chain.as_ref()),
            Self::TransactionsPendingErrorMaxAge(chain) => format!("{}.{}", self.as_ref(), chain.as_ref()),
            Self::SwapperVaultAddresses(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderAssetsLimit(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderAssetsDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderAssetsNewDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderAssetsMetadataDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderPricesDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderChartsHourlyDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderMetricsDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderCleanOutdatedDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::ListProviderUpdateDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::RateLimit(key, window) => format!("{}.{}", key.as_ref(), window.as_ref()),
        }
    }

    pub fn default_value(&self) -> String {
        match self {
            Self::TransactionsRequestLimit(_) => "100".to_string(),
            Self::TransactionsPendingErrorMaxAge(_) => "3d".to_string(),
            Self::SwapperVaultAddresses(_) => "5m".to_string(),
            Self::PriceProviderAssetsLimit(PriceProvider::TonApi) => "1000".to_string(),
            Self::PriceProviderAssetsLimit(_) => "5000".to_string(),
            Self::PriceProviderAssetsDuration(_) => "1d".to_string(),
            Self::PriceProviderAssetsNewDuration(_) => "15m".to_string(),
            Self::PriceProviderAssetsMetadataDuration(_) => "30d".to_string(),
            Self::PriceProviderPricesDuration(_) => "60s".to_string(),
            Self::PriceProviderChartsHourlyDuration(_) => "7d".to_string(),
            Self::PriceProviderMetricsDuration(_) => "5m".to_string(),
            Self::PriceProviderCleanOutdatedDuration(_) => "1d".to_string(),
            Self::ListProviderUpdateDuration(_) => "1d".to_string(),
            Self::RateLimit(key, window) => key.default_limit().get(*window).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_provider_assets_limit() {
        let tonapi = ConfigParamKey::PriceProviderAssetsLimit(PriceProvider::TonApi);
        let coingecko = ConfigParamKey::PriceProviderAssetsLimit(PriceProvider::Coingecko);

        assert_eq!(tonapi.key(), "priceProviderAssetsLimit.tonapi");
        assert_eq!(tonapi.default_value(), "1000");
        assert_eq!(coingecko.default_value(), "5000");
    }

    #[test]
    fn test_transactions_pending_error_max_age() {
        let bitcoin = ConfigParamKey::TransactionsPendingErrorMaxAge(Chain::Bitcoin);

        assert_eq!(bitcoin.key(), "transactionsPendingErrorMaxAge.bitcoin");
        assert_eq!(bitcoin.default_value(), "3d");
    }

    #[test]
    fn test_rate_limits() {
        let scaled = RateLimit::scaled(30);
        assert_eq!(RateLimitWindow::ALL.map(|window| scaled.get(window)), [30, 150, 750, 3000]);

        let daily = RateLimit::daily(100);
        assert_eq!(RateLimitWindow::ALL.map(|window| daily.get(window)), [100, 100, 100, 700]);

        let flat = RateLimit::flat(1);
        assert_eq!(RateLimitWindow::ALL.map(|window| flat.get(window)), [1, 1, 1, 1]);
    }
}
