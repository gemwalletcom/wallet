use std::str::FromStr;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::ChartPeriod;
use primitives::currency::Currency;
use primitives::{Chain, ConfigResponse};

use crate::services::assets::AssetList;

pub use store::{GemPreferencesStore, GemSecureStore};

const PRICE_ALERTS_ENABLED: &str = "price_alerts_enabled";
const CURRENCY: &str = "currency";
const CHART_PERIOD: &str = "chart_period";
const PERPETUAL_CHART_PERIOD: &str = "perpetual_chart_period";
const PUSH_NOTIFICATIONS_ENABLED: &str = "is_push_notifications_enabled";
const LAUNCHES_COUNT: &str = "launches_count";
const RATE_APPLICATION_SHOWN: &str = "rate_application_shown";
const SKIPPED_APP_VERSION: &str = "skipped_app_version";
const CONFIG: &str = "config";
const BUY_ASSETS_VERSION: &str = "buy_assets_version";
const SELL_ASSETS_VERSION: &str = "sell_assets_version";
const SWAP_ASSETS_VERSION: &str = "swap_assets_version";
const EXPLORER_NAME: &str = "explorer_name";
const PERPETUAL_MARKETS_UPDATED_AT: &str = "perpetual_markets_updated_at";
const PERPETUAL_PRICES_UPDATED_AT: &str = "perpetual_prices_updated_at";

#[derive(uniffi::Object)]
pub struct GemPreferencesService {
    store: Arc<dyn GemPreferencesStore>,
}

#[uniffi::export]
impl GemPreferencesService {
    pub fn get_skipped_app_version(&self) -> Option<String> {
        self.store.get(SKIPPED_APP_VERSION.to_string())
    }
    pub fn set_skipped_app_version(&self, version: String) -> Result<(), GemServiceError> {
        self.store.set(SKIPPED_APP_VERSION.to_string(), version)
    }
    pub fn get_currency(&self) -> Currency {
        self.stored_currency().unwrap_or(Currency::USD)
    }

    pub fn set_currency(&self, currency: Currency) -> Result<(), GemServiceError> {
        self.store.set(CURRENCY.to_string(), currency.as_ref().to_string())
    }

    pub fn setup_currency(&self, locale_currency: Option<String>) -> Result<Currency, GemServiceError> {
        if let Some(currency) = self.stored_currency() {
            return Ok(currency);
        }
        let currency = rules::default_currency(locale_currency);
        self.set_currency(currency.clone())?;
        Ok(currency)
    }

    pub fn get_chart_period(&self) -> ChartPeriod {
        self.store
            .get(CHART_PERIOD.to_string())
            .and_then(|value| ChartPeriod::from_str(&value).ok())
            .unwrap_or(ChartPeriod::Day)
    }

    pub fn set_chart_period(&self, period: ChartPeriod) -> Result<(), GemServiceError> {
        self.store.set(CHART_PERIOD.to_string(), period.as_ref().to_string())
    }

    pub fn get_perpetual_chart_period(&self) -> ChartPeriod {
        self.store
            .get(PERPETUAL_CHART_PERIOD.to_string())
            .and_then(|value| ChartPeriod::from_str(&value).ok())
            .unwrap_or(ChartPeriod::Day)
    }

    pub fn set_perpetual_chart_period(&self, period: ChartPeriod) -> Result<(), GemServiceError> {
        self.store.set(PERPETUAL_CHART_PERIOD.to_string(), period.as_ref().to_string())
    }

    pub fn is_push_notifications_enabled(&self) -> bool {
        self.store.get(PUSH_NOTIFICATIONS_ENABLED.to_string()).as_deref() == Some("true")
    }

    pub fn set_push_notifications_enabled(&self, enabled: bool) -> Result<(), GemServiceError> {
        self.store.set(PUSH_NOTIFICATIONS_ENABLED.to_string(), enabled.to_string())
    }

    pub fn get_launches_count(&self) -> u32 {
        self.store.get(LAUNCHES_COUNT.to_string()).and_then(|value| value.parse().ok()).unwrap_or(0)
    }

    pub fn increment_launches_count(&self) -> Result<u32, GemServiceError> {
        let count = self.get_launches_count() + 1;
        self.store.set(LAUNCHES_COUNT.to_string(), count.to_string())?;
        Ok(count)
    }

    pub fn should_request_review(&self) -> bool {
        let shown = self.store.get(RATE_APPLICATION_SHOWN.to_string()).as_deref() == Some("true");
        rules::should_request_review(self.get_launches_count(), shown)
    }

    pub fn set_rate_application_shown(&self) -> Result<(), GemServiceError> {
        self.store.set(RATE_APPLICATION_SHOWN.to_string(), "true".to_string())
    }

    pub fn is_price_alerts_enabled(&self) -> bool {
        self.store.get(PRICE_ALERTS_ENABLED.to_string()).as_deref() == Some("true")
    }

    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemPreferencesStore>) -> Self {
        Self { store }
    }

    pub fn default_currency(&self, locale_currency: Option<String>) -> Currency {
        rules::default_currency(locale_currency)
    }

    pub fn set_price_alerts_enabled(&self, enabled: bool) -> Result<(), GemServiceError> {
        self.store.set(PRICE_ALERTS_ENABLED.to_string(), enabled.to_string())
    }
}

impl GemPreferencesService {
    pub fn get_perpetual_markets_updated_at(&self) -> Result<Option<i64>, GemServiceError> {
        self.get_timestamp(PERPETUAL_MARKETS_UPDATED_AT)
    }

    pub fn set_perpetual_markets_updated_at(&self, timestamp: Option<i64>) -> Result<(), GemServiceError> {
        self.set_timestamp(PERPETUAL_MARKETS_UPDATED_AT, timestamp)
    }

    pub fn get_perpetual_prices_updated_at(&self) -> Result<Option<i64>, GemServiceError> {
        self.get_timestamp(PERPETUAL_PRICES_UPDATED_AT)
    }

    pub fn set_perpetual_prices_updated_at(&self, timestamp: Option<i64>) -> Result<(), GemServiceError> {
        self.set_timestamp(PERPETUAL_PRICES_UPDATED_AT, timestamp)
    }

    fn get_timestamp(&self, key: &str) -> Result<Option<i64>, GemServiceError> {
        Ok(self.store.get(key.to_string()).and_then(|value| value.parse().ok()))
    }

    fn set_timestamp(&self, key: &str, timestamp: Option<i64>) -> Result<(), GemServiceError> {
        match timestamp {
            Some(timestamp) => self.store.set(key.to_string(), timestamp.to_string()),
            None => self.store.remove(key.to_string()),
        }
    }

    pub fn get_assets_version(&self, list: AssetList) -> Option<String> {
        self.store.get(assets_version_key(list).to_string())
    }

    pub fn set_assets_version(&self, list: AssetList, version: String) -> Result<(), GemServiceError> {
        self.store.set(assets_version_key(list).to_string(), version)
    }

    pub fn get_config(&self) -> Option<ConfigResponse> {
        self.store.get(CONFIG.to_string()).and_then(|json| serde_json::from_str(&json).ok())
    }

    pub fn get_explorer_name(&self, chain: Chain) -> Option<String> {
        self.store.get(explorer_name_key(chain))
    }

    pub fn set_explorer_name(&self, chain: Chain, name: String) -> Result<(), GemServiceError> {
        self.store.set(explorer_name_key(chain), name)
    }

    pub fn set_config(&self, config: &ConfigResponse) -> Result<(), GemServiceError> {
        let json = serde_json::to_string(config).map_err(|error| GemServiceError::Store { msg: error.to_string() })?;
        self.store.set(CONFIG.to_string(), json)
    }
}

fn explorer_name_key(chain: Chain) -> String {
    format!("{EXPLORER_NAME}_{}", chain.as_ref())
}

fn assets_version_key(list: AssetList) -> &'static str {
    match list {
        AssetList::Buy => BUY_ASSETS_VERSION,
        AssetList::Sell => SELL_ASSETS_VERSION,
        AssetList::Swap => SWAP_ASSETS_VERSION,
    }
}

impl GemPreferencesService {
    fn stored_currency(&self) -> Option<Currency> {
        self.store.get(CURRENCY.to_string()).and_then(|code| Currency::from_str(&code).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryStore {
        values: Mutex<HashMap<String, String>>,
    }

    impl GemPreferencesStore for MemoryStore {
        fn get(&self, key: String) -> Option<String> {
            self.values.lock().unwrap().get(&key).cloned()
        }

        fn set(&self, key: String, value: String) -> Result<(), GemServiceError> {
            self.values.lock().unwrap().insert(key, value);
            Ok(())
        }

        fn remove(&self, key: String) -> Result<(), GemServiceError> {
            self.values.lock().unwrap().remove(&key);
            Ok(())
        }
    }

    #[test]
    fn test_price_alerts_enabled_defaults_to_false_and_round_trips() {
        let service = GemPreferencesService::new(Arc::new(MemoryStore::default()));

        assert!(!service.is_price_alerts_enabled());

        service.set_price_alerts_enabled(true).unwrap();
        assert!(service.is_price_alerts_enabled());

        service.set_price_alerts_enabled(false).unwrap();
        assert!(!service.is_price_alerts_enabled());
    }
}
