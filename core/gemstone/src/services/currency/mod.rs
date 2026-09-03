mod rules;

use std::sync::Arc;

use primitives::Currency;

use crate::services::device::GemDeviceService;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemCurrencyService {
    preferences: Arc<GemPreferencesService>,
    prices: Arc<GemPriceService>,
    device: Arc<GemDeviceService>,
}

#[uniffi::export]
impl GemCurrencyService {
    #[uniffi::constructor]
    pub fn new(preferences: Arc<GemPreferencesService>, prices: Arc<GemPriceService>, device: Arc<GemDeviceService>) -> Self {
        Self { preferences, prices, device }
    }

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn recommended_currencies(&self, locale: Option<Currency>) -> Vec<Currency> {
        rules::recommended_currencies(self.currency(), locale)
    }

    pub fn other_currencies(&self, locale: Option<Currency>) -> Vec<Currency> {
        rules::other_currencies(&self.recommended_currencies(locale))
    }

    pub async fn set_currency(&self, currency: Currency) -> Result<(), GemServiceError> {
        if currency == self.currency() {
            return Ok(());
        }
        self.preferences.set_currency(currency.clone())?;
        self.prices.change_currency(currency).await?;
        let _ = self.device.synchronize_if_needed().await;
        Ok(())
    }
}
