use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use primitives::{CacheFuture, ChartCandleStick, ChartPeriod};

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait ChartCandlesCacher: Send + Sync {
    async fn get_or_fetch_candles(&self, price_id: &str, period: ChartPeriod, duration: Duration, fetch: CacheFuture<'_, Vec<ChartCandleStick>>) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl ChartCandlesCacher for CacherClient {
    async fn get_or_fetch_candles(&self, price_id: &str, period: ChartPeriod, duration: Duration, fetch: CacheFuture<'_, Vec<ChartCandleStick>>) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Send + Sync>> {
        self.get_or_fetch_with_fallback(CacheKey::ChartCandles(price_id, period.as_ref()), duration, fetch).await
    }
}
