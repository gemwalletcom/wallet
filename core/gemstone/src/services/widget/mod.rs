pub mod model;
pub mod rules;

use std::sync::Arc;

pub use model::{GemWidgetCoin, GemWidgetSize};

use crate::api::{GemApiClient, GemApiError};
use crate::services::error::GemServiceError;

#[derive(uniffi::Object)]
pub struct GemWidgetService {
    api: Arc<GemApiClient>,
}

#[uniffi::export]
impl GemWidgetService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>) -> Self {
        Self { api }
    }

    pub fn refresh_interval_seconds(&self) -> u32 {
        rules::REFRESH_INTERVAL_SECONDS
    }

    pub async fn coins(&self, size: GemWidgetSize, currency: String) -> Result<Vec<GemWidgetCoin>, GemServiceError> {
        let ids = rules::coin_ids(size);
        let assets = self.api.client.get_assets(ids.clone(), Some(currency.clone())).await.map_err(GemApiError::from)?;
        Ok(rules::coins(&ids, assets, &currency, size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;

    const RESPONSE: &str = r#"[{
        "asset": {"id": "bitcoin", "name": "Bitcoin", "symbol": "BTC", "decimals": 8, "type": "NATIVE"},
        "properties": {"isEnabled": true, "isBuyable": true, "isSellable": true, "isSwapable": true, "isStakeable": false, "isEarnable": false, "hasImage": true, "hasPrice": true},
        "score": {"rank": 1, "type": "high"},
        "price": {"price": 69000.0, "priceChangePercentage24h": 2.5, "updatedAt": "2026-09-16T00:00:00Z", "provider": "coingecko"}
    }]"#;

    #[test]
    fn the_widget_reads_its_coins_through_the_api() {
        let service = GemWidgetService::new(Arc::new(GemApiClient::new(Arc::new(TestAlienProvider::with_json(200, RESPONSE)))));
        let coins = block_on(service.coins(GemWidgetSize::Small, "USD".to_string())).unwrap();
        assert_eq!(coins.len(), 1);
        assert_eq!(coins[0].symbol, "BTC");
        assert_eq!(service.refresh_interval_seconds(), 900);
    }
}
