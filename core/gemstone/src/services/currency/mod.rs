pub mod model;
mod rules;

use std::sync::Arc;

use primitives::Currency;

pub use model::{GemCurrencies, GemCurrencyRow};

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

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn currencies(&self, locale: Option<Currency>) -> GemCurrencies {
        rules::currencies(self.get_currency(), locale)
    }

    pub async fn set_currency(&self, currency: Currency) -> Result<(), GemServiceError> {
        if currency == self.get_currency() {
            return Ok(());
        }
        self.preferences.set_currency(currency.clone())?;
        self.prices.change_currency(currency).await?;
        let _ = self.device.synchronize_if_needed().await;
        Ok(())
    }
}
