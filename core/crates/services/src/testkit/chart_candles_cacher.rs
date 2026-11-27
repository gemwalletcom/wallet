use std::{collections::HashMap, error::Error, sync::Mutex, time::Duration};

use async_trait::async_trait;
use cacher::ChartCandlesCacher;
use primitives::{CacheFuture, ChartCandleStick, ChartPeriod};

#[derive(Default)]
pub(crate) struct MemoryChartCandlesCacher {
    candles: Mutex<HashMap<(String, ChartPeriod), Vec<ChartCandleStick>>>,
}

#[async_trait]
impl ChartCandlesCacher for MemoryChartCandlesCacher {
    async fn get_or_set_candles(&self, price_id: &str, period: ChartPeriod, _duration: Duration, request: CacheFuture<'_, Vec<ChartCandleStick>>) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Send + Sync>> {
        let key = (price_id.to_string(), period);
        if let Some(candles) = self.candles.lock().unwrap().get(&key) {
            return Ok(candles.clone());
        }
        let candles = request.await?;
        self.candles.lock().unwrap().insert(key, candles.clone());
        Ok(candles)
    }
}
