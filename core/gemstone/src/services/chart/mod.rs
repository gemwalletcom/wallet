pub mod rules;

use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, ChartDateValue, ChartPeriod};

use crate::api::{GemApiClient, GemApiError};
use crate::services::error::GemServiceError;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemChartService {
    api: Arc<GemApiClient>,
    price: Arc<GemPriceService>,
}

#[uniffi::export]
impl GemChartService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, price: Arc<GemPriceService>) -> Self {
        Self { api, price }
    }

    pub async fn sync_charts(&self, asset_id: AssetId, period: ChartPeriod, currency: Currency) -> Result<Vec<ChartDateValue>, GemServiceError> {
        let charts = self.api.client.get_charts(asset_id.clone(), period).await.map_err(GemApiError::from)?;
        if let Some(market) = charts.market {
            self.price.update_market(asset_id, market, currency.clone()).await?;
        }
        let rate = self.price.rate(currency.clone()).await?.ok_or(GemServiceError::InvalidInput {
            msg: format!("unknown currency: {currency}"),
        })?;
        Ok(rules::converted_values(charts.prices, rate.rate))
    }
}
