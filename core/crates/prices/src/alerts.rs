use std::error::Error;
use std::time::Duration;

use primitives::currency::Currency;
use primitives::{Asset, Device, FiatRate, Price, PriceAlert, PriceAlertDirection, PriceAlertType, PriceData};

const DEFAULT_RANK: i32 = 1000;

#[derive(Clone, Debug)]
pub struct PriceAlertRules {
    pub notification_cooldown: Duration,
    pub price_change_threshold: f64,
    pub rank_divisor: f64,
    pub milestones: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PriceAlertTrigger {
    pub alert_type: PriceAlertType,
    pub milestone: Option<f64>,
}

impl PriceAlertTrigger {
    fn new(alert_type: PriceAlertType) -> Self {
        Self { alert_type, milestone: None }
    }

    fn milestone(milestone: f64) -> Self {
        Self {
            alert_type: PriceAlertType::PriceMilestone,
            milestone: Some(milestone),
        }
    }
}

impl PriceAlertRules {
    pub fn evaluate(&self, price_alert: &PriceAlert, price_data: &PriceData, rates: &[FiatRate]) -> Option<PriceAlertTrigger> {
        if let Some(target_price) = price_alert.price {
            let direction = price_alert.price_direction.as_ref()?;
            let price = price_data.price * fiat_rate(rates, &price_alert.currency)?;
            let alert_type = match direction {
                PriceAlertDirection::Up if price >= target_price => Some(PriceAlertType::PriceUp),
                PriceAlertDirection::Down if price <= target_price => Some(PriceAlertType::PriceDown),
                _ => None,
            };
            return alert_type.map(PriceAlertTrigger::new);
        }

        if let Some(target_percent) = price_alert.price_percent_change {
            let direction = price_alert.price_direction.as_ref()?;
            let alert_type = match direction {
                PriceAlertDirection::Up if price_data.price_change_percentage_24h >= target_percent => Some(PriceAlertType::PricePercentChangeUp),
                PriceAlertDirection::Down if price_data.price_change_percentage_24h <= -target_percent => Some(PriceAlertType::PricePercentChangeDown),
                _ => None,
            };
            return alert_type.map(PriceAlertTrigger::new);
        }

        if price_data.all_time_high > 0.0 && price_data.price > price_data.all_time_high {
            return Some(PriceAlertTrigger::new(PriceAlertType::AllTimeHigh));
        }

        let price_24h_ago = price_24h_ago(price_data.price, price_data.price_change_percentage_24h);
        if let Some(milestone) = self.crossed_milestone(price_24h_ago, price_data.price) {
            return Some(PriceAlertTrigger::milestone(milestone));
        }

        let threshold = self.change_threshold(price_data.market_cap_rank.unwrap_or(0));
        if price_data.price_change_percentage_24h > threshold {
            return Some(PriceAlertTrigger::new(PriceAlertType::PriceChangesUp));
        }
        if price_data.price_change_percentage_24h < -threshold {
            return Some(PriceAlertTrigger::new(PriceAlertType::PriceChangesDown));
        }

        None
    }

    fn change_threshold(&self, rank: i32) -> f64 {
        let rank = if rank > 0 { rank } else { DEFAULT_RANK };
        self.price_change_threshold * (1.0 + (rank as f64).ln() / self.rank_divisor)
    }

    fn crossed_milestone(&self, price_24h_ago: f64, current_price: f64) -> Option<f64> {
        self.milestones.iter().find(|&&milestone| price_24h_ago < milestone && current_price >= milestone).copied()
    }
}

#[derive(Clone, Debug)]
pub struct PriceAlertNotification {
    pub device: Device,
    pub asset: Asset,
    pub price: Price,
    pub alert_type: PriceAlertType,
    pub price_alert: PriceAlert,
    pub milestone: Option<f64>,
}

impl PriceAlertNotification {
    pub fn currency(&self) -> &Currency {
        match self.alert_type {
            PriceAlertType::PriceUp | PriceAlertType::PriceDown => &self.price_alert.currency,
            PriceAlertType::PriceChangesUp | PriceAlertType::PriceChangesDown | PriceAlertType::PricePercentChangeUp | PriceAlertType::PricePercentChangeDown | PriceAlertType::AllTimeHigh | PriceAlertType::PriceMilestone => {
                &self.device.currency
            }
        }
    }

    pub fn target_value(&self) -> Option<f64> {
        match self.alert_type {
            PriceAlertType::PriceUp | PriceAlertType::PriceDown => self.price_alert.price,
            PriceAlertType::PriceMilestone => self.milestone,
            PriceAlertType::PriceChangesUp | PriceAlertType::PriceChangesDown | PriceAlertType::PricePercentChangeUp | PriceAlertType::PricePercentChangeDown | PriceAlertType::AllTimeHigh => None,
        }
    }

    pub fn with_rates(self, rates: &[FiatRate]) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let rate = fiat_rate(rates, self.currency()).ok_or_else(|| format!("missing fiat rate for {}", self.currency().as_ref()))?;
        Ok(Self { price: self.price.with_rate(rate), ..self })
    }
}

fn fiat_rate(rates: &[FiatRate], currency: &Currency) -> Option<f64> {
    rates.iter().find(|rate| rate.symbol == *currency).map(|rate| rate.rate)
}

fn price_24h_ago(current_price: f64, change_percent: f64) -> f64 {
    let divisor = 1.0 + change_percent / 100.0;
    if divisor <= 0.0 {
        return current_price;
    }
    current_price / divisor
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use primitives::{AssetId, Chain, PriceProvider};

    use super::*;

    const TEST_RATES: [FiatRate; 2] = [FiatRate { symbol: Currency::USD, rate: 1.0 }, FiatRate { symbol: Currency::EUR, rate: 0.86 }];

    #[test]
    fn test_evaluate() {
        let rules = PriceAlertRules::mock();
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let price_data = PriceData::mock_with(78_987.0, -1.4);
        let over = PriceAlert::new_price(asset_id.clone(), Currency::EUR, 71_000.0, PriceAlertDirection::Up);
        let under = PriceAlert::new_price(asset_id.clone(), Currency::EUR, 71_000.0, PriceAlertDirection::Down);
        let over_usd = PriceAlert::new_price(asset_id.clone(), Currency::USD, 71_000.0, PriceAlertDirection::Up);
        let over_unknown = PriceAlert::new_price(asset_id.clone(), Currency::JPY, 71_000.0, PriceAlertDirection::Up);
        let percent_up = PriceAlert::new_price_percent(asset_id.clone(), Currency::USD, 5.0, PriceAlertDirection::Up);
        let auto = PriceAlert::new_auto(asset_id, Currency::EUR);

        assert_eq!(rules.evaluate(&over, &price_data, &TEST_RATES), None);
        assert_eq!(rules.evaluate(&under, &price_data, &TEST_RATES), Some(PriceAlertTrigger::new(PriceAlertType::PriceDown)));
        assert_eq!(rules.evaluate(&over_usd, &price_data, &TEST_RATES), Some(PriceAlertTrigger::new(PriceAlertType::PriceUp)));
        assert_eq!(rules.evaluate(&over_unknown, &price_data, &TEST_RATES), None);
        assert_eq!(rules.evaluate(&percent_up, &PriceData::mock_with(78_987.0, 5.0), &[]), Some(PriceAlertTrigger::new(PriceAlertType::PricePercentChangeUp)));
        assert_eq!(rules.evaluate(&percent_up, &PriceData::mock_with(78_987.0, 4.9), &[]), None);
        assert_eq!(rules.evaluate(&auto, &PriceData::mock_with(78_987.0, 6.0), &[]), Some(PriceAlertTrigger::new(PriceAlertType::PriceChangesUp)));
        assert_eq!(rules.evaluate(&auto, &PriceData::mock_with(78_987.0, -6.0), &[]), Some(PriceAlertTrigger::new(PriceAlertType::PriceChangesDown)));
        assert_eq!(rules.evaluate(&auto, &PriceData::mock_with(78_987.0, 1.0), &[]), None);
        assert_eq!(
            rules.evaluate(
                &auto,
                &PriceData {
                    all_time_high: 78_000.0,
                    ..PriceData::mock_with(78_987.0, 1.0)
                },
                &[]
            ),
            Some(PriceAlertTrigger::new(PriceAlertType::AllTimeHigh))
        );
        assert_eq!(
            PriceAlertRules {
                milestones: vec![50_000.0, 100_000.0],
                ..PriceAlertRules::mock()
            }
            .evaluate(&auto, &PriceData::mock_with(101_000.0, 2.0), &[]),
            Some(PriceAlertTrigger::milestone(100_000.0))
        );
        assert_eq!(
            PriceAlertRules {
                milestones: vec![100_000.0],
                ..PriceAlertRules::mock()
            }
            .evaluate(&auto, &PriceData::mock_with(99_000.0, -2.0), &[]),
            None
        );
    }

    #[test]
    fn test_with_rates() {
        let asset = Asset::from_chain(Chain::Bitcoin);
        let price = Price::new(78_987.0, -1.4, DateTime::from_timestamp(1_788_821_182, 0).unwrap(), PriceProvider::Coingecko);
        let target = PriceAlertNotification {
            device: Device::mock(),
            asset: asset.clone(),
            price,
            alert_type: PriceAlertType::PriceUp,
            price_alert: PriceAlert::new_price(asset.id.clone(), Currency::EUR, 71_000.0, PriceAlertDirection::Up),
            milestone: None,
        };
        let automatic = PriceAlertNotification {
            alert_type: PriceAlertType::PriceChangesUp,
            price_alert: PriceAlert::new_auto(asset.id, Currency::EUR),
            ..target.clone()
        };

        assert_eq!(target.currency(), &Currency::EUR);
        assert_eq!(target.clone().with_rates(&TEST_RATES).unwrap().price, price.with_rate(0.86));
        assert_eq!(automatic.currency(), &Currency::USD);
        assert_eq!(automatic.with_rates(&TEST_RATES).unwrap().price, price);
        assert!(target.with_rates(&[]).is_err());
    }

    #[test]
    fn test_target_value() {
        let asset = Asset::from_chain(Chain::Bitcoin);
        let alert = PriceAlertNotification {
            device: Device::mock(),
            asset: asset.clone(),
            price: Price::new(80_954.0, -0.27, Utc::now(), PriceProvider::Coingecko),
            alert_type: PriceAlertType::PriceDown,
            price_alert: PriceAlert::new_price(asset.id.clone(), Currency::USD, 81_000.0, PriceAlertDirection::Down),
            milestone: None,
        };
        let milestone_alert = PriceAlertNotification {
            alert_type: PriceAlertType::PriceMilestone,
            price_alert: PriceAlert::new_auto(asset.id, Currency::USD),
            milestone: Some(100_000.0),
            ..alert.clone()
        };
        let automatic_alert = PriceAlertNotification {
            alert_type: PriceAlertType::PriceChangesUp,
            ..alert.clone()
        };

        assert_eq!(alert.target_value(), Some(81_000.0));
        assert_eq!(milestone_alert.target_value(), Some(100_000.0));
        assert_eq!(automatic_alert.target_value(), None);
    }
}
