use chrono::Utc;

use crate::{Asset, AssetId, Chain, Price, PriceAlert, PriceAlertData, PriceProvider, currency::Currency};

impl PriceAlert {
    pub fn mock(chain: Chain, price: Option<f64>) -> Self {
        Self {
            asset_id: AssetId::from_chain(chain),
            currency: Currency::USD,
            price,
            price_percent_change: None,
            price_direction: None,
            last_notified_at: None,
            identifier: String::new(),
        }
    }
}

impl PriceAlertData {
    pub fn mock(price_alert: PriceAlert, price: Option<f64>, price_change_percentage_24h: Option<f64>) -> Self {
        Self {
            asset: Asset::from_chain(Chain::Bitcoin),
            price: price.map(|price| Price::new(price, price_change_percentage_24h.unwrap_or_default(), Utc::now(), PriceProvider::default())),
            price_alert,
            rank_score: 20,
        }
    }
}
