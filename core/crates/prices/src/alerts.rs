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
    pub rate: FiatRate,
    pub target: Option<f64>,
}

impl PriceAlertRules {
    pub fn evaluate(&self, price_alert: &PriceAlert, device: &Device, price_data: &PriceData, rates: &[FiatRate]) -> Option<PriceAlertTrigger> {
        let currency = alert_currency(price_alert, device);
        let rate = rates.iter().find(|rate| rate.symbol == *currency)?.clone();
        let price = rate.multiplier(price_data.price);
        let (alert_type, target) = self.alert_type(price_alert, price_data, price)?;
        Some(PriceAlertTrigger { alert_type, rate, target })
    }

    fn alert_type(&self, price_alert: &PriceAlert, price_data: &PriceData, price: f64) -> Option<(PriceAlertType, Option<f64>)> {
        if let Some(target_price) = price_alert.price {
            let direction = price_alert.price_direction.as_ref()?;
            return match direction {
                PriceAlertDirection::Up if price >= target_price => Some((PriceAlertType::PriceUp, Some(target_price))),
                PriceAlertDirection::Down if price <= target_price => Some((PriceAlertType::PriceDown, Some(target_price))),
                _ => None,
            };
        }

        if let Some(target_percent) = price_alert.price_percent_change {
            let direction = price_alert.price_direction.as_ref()?;
            let price_change = price_data.price_change_percentage_24h?;
            return match direction {
                PriceAlertDirection::Up if price_change >= target_percent => Some((PriceAlertType::PricePercentChangeUp, None)),
                PriceAlertDirection::Down if price_change <= -target_percent => Some((PriceAlertType::PricePercentChangeDown, None)),
                _ => None,
            };
        }

        if price_data.all_time_high > 0.0 && price_data.price > price_data.all_time_high {
            return Some((PriceAlertType::AllTimeHigh, None));
        }

        let price_change = price_data.price_change_percentage_24h?;
        let price_24h_ago = price_24h_ago(price, price_change);
        if let Some(milestone) = self.crossed_milestone(price_24h_ago, price) {
            return Some((PriceAlertType::PriceMilestone, Some(milestone)));
        }

        let threshold = self.change_threshold(price_data.market_cap_rank.unwrap_or(0));
        if price_change > threshold {
            return Some((PriceAlertType::PriceChangesUp, None));
        }
        if price_change < -threshold {
            return Some((PriceAlertType::PriceChangesDown, None));
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
    pub currency: Currency,
    pub price: Price,
    pub alert_type: PriceAlertType,
    pub price_alert: PriceAlert,
    pub target: Option<f64>,
}

impl PriceAlertNotification {
    pub fn new(device: Device, asset: Asset, price_alert: PriceAlert, price_data: &PriceData, trigger: PriceAlertTrigger) -> Self {
        Self {
            device,
            asset,
            price: price_data.as_price().with_rate(trigger.rate.rate),
            currency: trigger.rate.symbol,
            alert_type: trigger.alert_type,
            price_alert,
            target: trigger.target,
        }
    }
}

fn alert_currency<'a>(price_alert: &'a PriceAlert, device: &'a Device) -> &'a Currency {
    match price_alert.price {
        Some(_) => &price_alert.currency,
        None => &device.currency,
    }
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
    use primitives::{AssetId, Chain};

    use super::*;

    const USD: FiatRate = FiatRate { symbol: Currency::USD, rate: 1.0 };
    const EUR: FiatRate = FiatRate { symbol: Currency::EUR, rate: 0.86 };
    const TEST_RATES: [FiatRate; 2] = [USD, EUR];

    fn trigger(alert_type: PriceAlertType, rate: FiatRate) -> Option<PriceAlertTrigger> {
        Some(PriceAlertTrigger { alert_type, rate, target: None })
    }

    fn target(alert_type: PriceAlertType, target: f64, rate: FiatRate) -> Option<PriceAlertTrigger> {
        Some(PriceAlertTrigger { alert_type, rate, target: Some(target) })
    }

    fn device(currency: Currency) -> Device {
        Device { currency, ..Device::mock() }
    }

    #[test]
    fn test_evaluate() {
        let rules = PriceAlertRules::mock();
        let usd = device(Currency::USD);
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let price_data = PriceData::mock_with(78_987.0, -1.4);
        let over = PriceAlert::new_price(asset_id.clone(), Currency::EUR, 71_000.0, PriceAlertDirection::Up);
        let under = PriceAlert::new_price(asset_id.clone(), Currency::EUR, 71_000.0, PriceAlertDirection::Down);
        let over_usd = PriceAlert::new_price(asset_id.clone(), Currency::USD, 71_000.0, PriceAlertDirection::Up);
        let over_unknown = PriceAlert::new_price(asset_id.clone(), Currency::JPY, 71_000.0, PriceAlertDirection::Up);
        let percent_up = PriceAlert::new_price_percent(asset_id.clone(), Currency::USD, 5.0, PriceAlertDirection::Up);
        let auto = PriceAlert::new_auto(asset_id, Currency::EUR);

        assert_eq!(rules.evaluate(&over, &usd, &price_data, &TEST_RATES), None);
        assert_eq!(rules.evaluate(&under, &usd, &price_data, &TEST_RATES), target(PriceAlertType::PriceDown, 71_000.0, EUR));
        assert_eq!(rules.evaluate(&over_usd, &device(Currency::EUR), &price_data, &TEST_RATES), target(PriceAlertType::PriceUp, 71_000.0, USD));
        assert_eq!(rules.evaluate(&over_unknown, &usd, &price_data, &TEST_RATES), None);
        assert_eq!(rules.evaluate(&percent_up, &usd, &PriceData::mock_with(78_987.0, 5.0), &TEST_RATES), trigger(PriceAlertType::PricePercentChangeUp, USD));
        assert_eq!(rules.evaluate(&percent_up, &usd, &PriceData::mock_with(78_987.0, 4.9), &TEST_RATES), None);
        assert_eq!(rules.evaluate(&auto, &usd, &PriceData::mock_with(78_987.0, 6.0), &TEST_RATES), trigger(PriceAlertType::PriceChangesUp, USD));
        assert_eq!(rules.evaluate(&auto, &usd, &PriceData::mock_with(78_987.0, -6.0), &TEST_RATES), trigger(PriceAlertType::PriceChangesDown, USD));
        assert_eq!(rules.evaluate(&auto, &device(Currency::EUR), &PriceData::mock_with(78_987.0, 6.0), &TEST_RATES), trigger(PriceAlertType::PriceChangesUp, EUR));
        assert_eq!(rules.evaluate(&auto, &device(Currency::JPY), &PriceData::mock_with(78_987.0, 6.0), &TEST_RATES), None);
        assert_eq!(rules.evaluate(&auto, &usd, &PriceData::mock_with(78_987.0, 1.0), &TEST_RATES), None);
        let unknown_change = PriceData {
            price_change_percentage_24h: None,
            ..PriceData::mock_with(78_987.0, 0.0)
        };
        assert_eq!(rules.evaluate(&percent_up, &usd, &unknown_change, &TEST_RATES), None);
        assert_eq!(rules.evaluate(&auto, &usd, &unknown_change, &TEST_RATES), None);
        assert_eq!(rules.evaluate(&under, &usd, &unknown_change, &TEST_RATES), target(PriceAlertType::PriceDown, 71_000.0, EUR));
        assert_eq!(
            rules.evaluate(
                &auto,
                &device(Currency::EUR),
                &PriceData {
                    all_time_high: 78_000.0,
                    ..PriceData::mock_with(78_987.0, 1.0)
                },
                &TEST_RATES
            ),
            trigger(PriceAlertType::AllTimeHigh, EUR)
        );
    }

    #[test]
    fn test_evaluate_milestone_in_device_currency() {
        let rules = PriceAlertRules {
            milestones: vec![50_000.0, 100_000.0],
            ..PriceAlertRules::mock()
        };
        let auto = PriceAlert::new_auto(AssetId::from_chain(Chain::Bitcoin), Currency::USD);

        assert_eq!(
            rules.evaluate(&auto, &device(Currency::USD), &PriceData::mock_with(101_000.0, 2.0), &TEST_RATES),
            target(PriceAlertType::PriceMilestone, 100_000.0, USD)
        );
        assert_eq!(rules.evaluate(&auto, &device(Currency::USD), &PriceData::mock_with(99_000.0, -2.0), &TEST_RATES), None);
        assert_eq!(rules.evaluate(&auto, &device(Currency::EUR), &PriceData::mock_with(101_000.0, 2.0), &TEST_RATES), None);
        assert_eq!(
            rules.evaluate(&auto, &device(Currency::EUR), &PriceData::mock_with(117_000.0, 2.0), &TEST_RATES),
            target(PriceAlertType::PriceMilestone, 100_000.0, EUR)
        );
    }

    #[test]
    fn test_notification_is_in_the_trigger_currency() {
        let asset = Asset::from_chain(Chain::Bitcoin);
        let price_data = PriceData::mock_with(117_000.0, 2.0);
        let notification = PriceAlertNotification::new(
            device(Currency::EUR),
            asset.clone(),
            PriceAlert::new_auto(asset.id, Currency::USD),
            &price_data,
            target(PriceAlertType::PriceMilestone, 100_000.0, EUR).unwrap(),
        );

        assert_eq!(notification.currency, Currency::EUR);
        assert_eq!(notification.price, price_data.as_price().with_rate(0.86));
        assert_eq!(notification.target, Some(100_000.0));
    }
}
