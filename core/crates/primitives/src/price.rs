use crate::{Asset, AssetLink, AssetMarket, PriceAlert, PriceProvider};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub updated_at: DateTime<Utc>,
    #[typeshare(skip)]
    #[serde(default)]
    pub provider: PriceProvider,
}

impl Price {
    pub fn new(price: f64, price_change_percentage_24h: f64, updated_at: DateTime<Utc>, provider: PriceProvider) -> Self {
        Price {
            price,
            price_change_percentage_24h,
            updated_at,
            provider,
        }
    }

    pub fn with_rate(self, rate: f64) -> Self {
        Price { price: self.price * rate, ..self }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Equatable")]
struct PriceData {
    asset: Asset,
    price: Option<Price>,
    price_alerts: Vec<PriceAlert>,
    market: Option<AssetMarket>,
    links: Vec<AssetLink>,
}
