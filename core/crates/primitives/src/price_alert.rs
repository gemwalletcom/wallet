use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::{Asset, AssetId, Device, Price, currency::Currency};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlert {
    pub asset_id: AssetId,
    pub currency: Currency,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_percent_change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_direction: Option<PriceAlertDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_notified_at: Option<DateTime<Utc>>,
    #[typeshare(skip)]
    #[serde(skip)]
    pub identifier: String,
}

impl PriceAlert {
    pub fn new_auto(asset_id: AssetId, currency: Currency) -> Self {
        Self {
            identifier: asset_id.to_string(),
            asset_id,
            currency,
            price: None,
            price_percent_change: None,
            price_direction: None,
            last_notified_at: None,
        }
    }

    pub fn new_price(asset_id: AssetId, currency: Currency, price: f64, direction: PriceAlertDirection) -> Self {
        Self {
            identifier: Self::generate_id(&asset_id, &currency, Some(price), None, Some(&direction)),
            asset_id,
            currency,
            price: Some(price),
            price_percent_change: None,
            price_direction: Some(direction),
            last_notified_at: None,
        }
    }

    pub fn new_price_percent(asset_id: AssetId, currency: Currency, percent_change: f64, direction: PriceAlertDirection) -> Self {
        Self {
            identifier: Self::generate_id(&asset_id, &currency, None, Some(percent_change), Some(&direction)),
            asset_id,
            currency,
            price: None,
            price_percent_change: Some(percent_change),
            price_direction: Some(direction),
            last_notified_at: None,
        }
    }

    pub fn id(&self) -> String {
        if !self.identifier.is_empty() {
            return self.identifier.clone();
        }
        Self::generate_id(&self.asset_id, &self.currency, self.price, self.price_percent_change, self.price_direction.as_ref())
    }

    fn generate_id(asset_id: &AssetId, currency: &Currency, price: Option<f64>, price_percent_change: Option<f64>, price_direction: Option<&PriceAlertDirection>) -> String {
        if price.is_none() && price_percent_change.is_none() && price_direction.is_none() {
            return asset_id.to_string();
        }
        [
            Some(asset_id.to_string()),
            Some(currency.to_string()),
            price.map(|p| p.to_string()),
            price_percent_change.map(|p| p.to_string()),
            price_direction.map(|d| d.as_ref().to_string()),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("_")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlertData {
    pub asset: Asset,
    pub price: Option<Price>,
    pub price_alert: PriceAlert,
}

#[derive(Clone, Debug, Serialize, Deserialize, AsRefStr, EnumString, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PriceAlertDirection {
    Up,
    Down,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PriceAlertType {
    PriceChangesUp,
    PriceChangesDown,
    PriceUp,
    PriceDown,
    PricePercentChangeUp,
    PricePercentChangeDown,
    AllTimeHigh,
    PriceMilestone,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum PriceAlertNotificationType {
    Auto,
    Price,
    PricePercentChange,
}

pub type PriceAlerts = Vec<PriceAlert>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePriceAlert {
    pub device: Device,
    pub price_alert: PriceAlert,
}

#[cfg(test)]
mod tests {
    use crate::Chain;

    use super::*;

    #[test]
    fn test_generate_id() {
        let eth = AssetId::from_chain(Chain::Ethereum);
        assert_eq!(PriceAlert::generate_id(&eth, &Currency::USD, None, None, None), "ethereum");
        assert_eq!(
            PriceAlert::generate_id(&eth, &Currency::USD, Some(100.0), None, Some(&PriceAlertDirection::Up)),
            "ethereum_USD_100_up"
        );
        assert_eq!(
            PriceAlert::generate_id(&eth, &Currency::USD, Some(1.12344), None, Some(&PriceAlertDirection::Down)),
            "ethereum_USD_1.12344_down"
        );
        assert_eq!(
            PriceAlert::generate_id(&eth, &Currency::USD, None, Some(5.0), Some(&PriceAlertDirection::Up)),
            "ethereum_USD_5_up"
        );
        assert_eq!(
            PriceAlert::generate_id(&eth, &Currency::USD, None, Some(10_000.10), Some(&PriceAlertDirection::Down)),
            "ethereum_USD_10000.1_down"
        );
    }

    #[test]
    fn test_new_auto_price_percent() {
        let eth = AssetId::from_chain(Chain::Ethereum);
        assert_eq!(PriceAlert::new_auto(eth.clone(), Currency::USD).identifier, "ethereum");
        assert_eq!(
            PriceAlert::new_price(eth.clone(), Currency::USD, 100.0, PriceAlertDirection::Up).identifier,
            "ethereum_USD_100_up"
        );
        assert_eq!(
            PriceAlert::new_price_percent(eth, Currency::USD, 5.0, PriceAlertDirection::Down).identifier,
            "ethereum_USD_5_down"
        );
    }

    #[test]
    fn test_id_returns_stored_identifier() {
        let alert = PriceAlert {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            currency: Currency::USD,
            price: Some(100.0),
            price_percent_change: None,
            price_direction: Some(PriceAlertDirection::Up),
            last_notified_at: None,
            identifier: "stored_from_db".to_string(),
        };
        assert_eq!(alert.id(), "stored_from_db");
    }
}
