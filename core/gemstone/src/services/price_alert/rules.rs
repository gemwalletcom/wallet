use std::cmp::Ordering;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use primitives::{AssetId, Currency, PriceAlert, PriceAlertData, PriceAlertDirection, PriceAlertNotificationType};

use crate::formatted_number::GemFormattedNumber;
use crate::percentage::GemPercentageStyle;
use crate::precision::GemCurrencyStyle;
use crate::services::collections::stale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPriceAlertKind {
    Auto,
    Over,
    Under,
    Increase,
    Decrease,
}

#[uniffi::export]
impl GemPriceAlertKind {
    pub fn groups_by_asset(&self) -> bool {
        match self {
            Self::Auto => false,
            Self::Over | Self::Under | Self::Increase | Self::Decrease => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPriceAlertToggle {
    Enabled,
    Disabled,
}

#[uniffi::export]
impl GemPriceAlertToggle {
    pub fn toggled(&self) -> Self {
        match self {
            Self::Enabled => Self::Disabled,
            Self::Disabled => Self::Enabled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPriceAlertLabel {
    Over,
    Under,
    IncreasesBy,
    DecreasesBy,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPriceAlertText {
    Empty,
    Label { label: GemPriceAlertLabel },
    Number { value: GemFormattedNumber },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPriceAlertRow {
    pub asset_id: AssetId,
    pub title: String,
    pub symbol: String,
    pub kind: GemPriceAlertKind,
    pub direction: Option<PriceAlertDirection>,
    pub prefix: GemPriceAlertText,
    pub suffix: GemPriceAlertText,
}

fn text(number: Option<GemFormattedNumber>) -> GemPriceAlertText {
    match number {
        Some(value) => GemPriceAlertText::Number { value },
        None => GemPriceAlertText::Empty,
    }
}

fn label(label: GemPriceAlertLabel) -> GemPriceAlertText {
    GemPriceAlertText::Label { label }
}

pub struct PriceAlertSync {
    pub delete_ids: Vec<String>,
    pub alerts: Vec<PriceAlert>,
}

pub fn reconcile(local: Vec<PriceAlert>, remote: Vec<PriceAlert>) -> PriceAlertSync {
    let delete_ids = stale(local.iter().map(PriceAlert::id), remote.iter().map(PriceAlert::id));
    let local_notified: HashMap<String, Option<DateTime<Utc>>> = local.iter().map(|alert| (alert.id(), alert.last_notified_at)).collect();
    let alerts = remote
        .into_iter()
        .filter(|alert| local_notified.get(&alert.id()) != Some(&alert.last_notified_at))
        .collect();
    PriceAlertSync { delete_ids, alerts }
}

pub fn displayed_price_alert_ids(alerts: Vec<PriceAlert>) -> Vec<String> {
    sorted_price_alerts(alerts.into_iter().filter(PriceAlert::should_display).collect())
        .iter()
        .map(PriceAlert::id)
        .collect()
}

pub fn price_alert_toggle(alerts: &[PriceAlert]) -> GemPriceAlertToggle {
    match alerts.iter().any(|alert| alert.notification_type() == PriceAlertNotificationType::Auto) {
        true => GemPriceAlertToggle::Enabled,
        false => GemPriceAlertToggle::Disabled,
    }
}

pub fn price_alert_row(data: &PriceAlertData, price_currency: Currency) -> GemPriceAlertRow {
    let PriceAlertData {
        asset,
        price: market,
        price_alert: alert,
    } = data;
    let current_price = market.map(|price| price.price);
    let price_change_percentage_24h = market.map(|price| price.price_change_percentage_24h);
    let kind = alert_kind(alert);
    let price = alert.price.or(current_price).map(|price| {
        let currency = match alert.price {
            Some(_) => alert.currency.clone(),
            None => price_currency,
        };
        GemFormattedNumber::currency(price, currency, GemCurrencyStyle::Currency)
    });
    let percent = alert
        .price_percent_change
        .or(price_change_percentage_24h)
        .map(|percent| GemFormattedNumber::percentage(percent, percent_style(kind)));

    let (prefix, suffix) = match kind {
        GemPriceAlertKind::Auto => (text(price), text(percent)),
        GemPriceAlertKind::Over => (label(GemPriceAlertLabel::Over), text(price)),
        GemPriceAlertKind::Under => (label(GemPriceAlertLabel::Under), text(price)),
        GemPriceAlertKind::Increase => (label(GemPriceAlertLabel::IncreasesBy), text(percent)),
        GemPriceAlertKind::Decrease => (label(GemPriceAlertLabel::DecreasesBy), text(percent)),
    };

    GemPriceAlertRow {
        asset_id: asset.id.clone(),
        title: asset.name.clone(),
        symbol: asset.symbol.clone(),
        kind,
        direction: row_direction(alert, current_price, price_change_percentage_24h),
        prefix,
        suffix,
    }
}

fn percent_style(kind: GemPriceAlertKind) -> GemPercentageStyle {
    match kind {
        GemPriceAlertKind::Increase | GemPriceAlertKind::Decrease => GemPercentageStyle::Unsigned,
        GemPriceAlertKind::Auto | GemPriceAlertKind::Over | GemPriceAlertKind::Under => GemPercentageStyle::Signed,
    }
}

pub fn alert_kind(alert: &PriceAlert) -> GemPriceAlertKind {
    match (alert.notification_type(), alert.price_direction.clone()) {
        (PriceAlertNotificationType::Price, Some(PriceAlertDirection::Up)) => GemPriceAlertKind::Over,
        (PriceAlertNotificationType::Price, Some(PriceAlertDirection::Down)) => GemPriceAlertKind::Under,
        (PriceAlertNotificationType::PricePercentChange, Some(PriceAlertDirection::Up)) => GemPriceAlertKind::Increase,
        (PriceAlertNotificationType::PricePercentChange, Some(PriceAlertDirection::Down)) => GemPriceAlertKind::Decrease,
        (PriceAlertNotificationType::Auto, _) | (_, None) => GemPriceAlertKind::Auto,
    }
}

fn row_direction(alert: &PriceAlert, current_price: Option<f64>, price_change_percentage_24h: Option<f64>) -> Option<PriceAlertDirection> {
    match alert.price_direction.clone() {
        Some(direction) => Some(direction),
        None => match alert.price {
            Some(price) => alert_direction(PriceAlertNotificationType::Price, Some(price), current_price, PriceAlertDirection::Up),
            None => value_direction(price_change_percentage_24h),
        },
    }
}

fn value_direction(value: Option<f64>) -> Option<PriceAlertDirection> {
    let value = value.filter(|value| value.is_finite())?;
    match value.total_cmp(&0.0) {
        Ordering::Greater => Some(PriceAlertDirection::Up),
        Ordering::Less => Some(PriceAlertDirection::Down),
        Ordering::Equal => None,
    }
}

pub fn alert_direction(
    notification_type: PriceAlertNotificationType,
    input_value: Option<f64>,
    current_price: Option<f64>,
    selected_direction: PriceAlertDirection,
) -> Option<PriceAlertDirection> {
    let input_value = input_value.filter(|value| is_positive(*value))?;
    match notification_type {
        PriceAlertNotificationType::Price => match input_value.total_cmp(&current_price.filter(|price| is_positive(*price))?) {
            Ordering::Greater => Some(PriceAlertDirection::Up),
            Ordering::Less => Some(PriceAlertDirection::Down),
            Ordering::Equal => None,
        },
        PriceAlertNotificationType::PricePercentChange => Some(selected_direction),
        PriceAlertNotificationType::Auto => None,
    }
}

fn is_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn sorted_price_alerts(alerts: Vec<PriceAlert>) -> Vec<PriceAlert> {
    let mut sorted = alerts;
    sorted.sort_by(|left, right| {
        price(right)
            .total_cmp(&price(left))
            .then(direction(right).cmp(&direction(left)))
            .then(percent(right).total_cmp(&percent(left)))
    });
    sorted
}

fn price(alert: &PriceAlert) -> f64 {
    alert.price.unwrap_or_default()
}

fn percent(alert: &PriceAlert) -> f64 {
    alert.price_percent_change.unwrap_or_default()
}

fn direction(alert: &PriceAlert) -> u8 {
    match alert.price_direction {
        Some(PriceAlertDirection::Up) => 1,
        Some(PriceAlertDirection::Down) => 0,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_only_an_auto_alert_stands_outside_its_asset_group() {
        assert!(!GemPriceAlertKind::Auto.groups_by_asset());
        assert!(GemPriceAlertKind::Over.groups_by_asset());
        assert!(GemPriceAlertKind::Under.groups_by_asset());
        assert!(GemPriceAlertKind::Increase.groups_by_asset());
        assert!(GemPriceAlertKind::Decrease.groups_by_asset());
    }

    use super::*;
    use primitives::{AssetId, Chain, currency::Currency};

    fn alert(price: f64, notified: Option<i64>) -> PriceAlert {
        PriceAlert {
            asset_id: AssetId::from_chain(Chain::Bitcoin),
            currency: Currency::USD,
            price: Some(price),
            price_percent_change: None,
            price_direction: None,
            last_notified_at: notified.and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0)),
            identifier: String::new(),
        }
    }

    use crate::formatted_number::GemNumberUnit;
    use primitives::{Asset, Price, PriceProvider};

    fn data(alert: &PriceAlert, asset: &Asset, price: Option<f64>, change: Option<f64>) -> PriceAlertData {
        PriceAlertData {
            asset: asset.clone(),
            price: price.map(|price| Price::new(price, change.unwrap_or_default(), Utc::now(), PriceProvider::default())),
            price_alert: alert.clone(),
        }
    }

    #[test]
    fn test_each_kind_names_which_slot_holds_the_price_and_which_the_percent() {
        let asset = Asset::from_chain(Chain::Bitcoin);
        let asset_id = asset.id.clone();
        let row = |alert: &PriceAlert| price_alert_row(&data(alert, &asset, Some(100.0), Some(-2.0)), Currency::USD);
        let is_price = |text: &GemPriceAlertText| matches!(text, GemPriceAlertText::Number { value } if matches!(value.unit, GemNumberUnit::Currency { .. }));
        let is_percent = |text: &GemPriceAlertText| matches!(text, GemPriceAlertText::Number { value } if matches!(value.unit, GemNumberUnit::Percent));

        let auto = row(&PriceAlert::new_auto(asset_id.clone(), Currency::USD));
        assert!(is_price(&auto.prefix), "an auto alert leads with the price");
        assert!(is_percent(&auto.suffix), "and follows with the day's change");

        for (direction, label) in [(PriceAlertDirection::Up, GemPriceAlertLabel::Over), (PriceAlertDirection::Down, GemPriceAlertLabel::Under)] {
            let priced = row(&PriceAlert::new_price(asset_id.clone(), Currency::USD, 120.0, direction));
            assert_eq!(priced.prefix, GemPriceAlertText::Label { label });
            assert!(is_price(&priced.suffix), "a price alert shows the target it waits for");
        }

        for (direction, label) in [
            (PriceAlertDirection::Up, GemPriceAlertLabel::IncreasesBy),
            (PriceAlertDirection::Down, GemPriceAlertLabel::DecreasesBy),
        ] {
            let percent = row(&PriceAlert::new_price_percent(asset_id.clone(), Currency::USD, 5.0, direction));
            assert_eq!(percent.prefix, GemPriceAlertText::Label { label });
            assert!(is_percent(&percent.suffix), "a percent alert shows its own target");
        }
    }

    #[test]
    fn test_the_row_picks_the_percent_style_from_the_kind_not_from_the_stored_percent() {
        let asset = Asset::from_chain(Chain::Bitcoin);
        let asset_id = asset.id.clone();
        let directed = PriceAlert::new_price_percent(asset_id.clone(), Currency::USD, 5.0, PriceAlertDirection::Up);
        assert_eq!(
            price_alert_row(&data(&directed, &asset, Some(100.0), None), Currency::USD).suffix,
            GemPriceAlertText::Number {
                value: GemFormattedNumber::percentage(5.0, GemPercentageStyle::Unsigned)
            }
        );

        let undirected = PriceAlert {
            price_direction: None,
            ..directed
        };
        let row = price_alert_row(&data(&undirected, &asset, Some(100.0), None), Currency::USD);
        assert_eq!(row.kind, GemPriceAlertKind::Auto);
        assert_eq!(
            row.suffix,
            GemPriceAlertText::Number {
                value: GemFormattedNumber::percentage(5.0, GemPercentageStyle::Signed)
            },
            "a stored percent with no direction still reads as the day's change"
        );
    }

    #[test]
    fn test_price_alert_row_names_the_kind_and_the_direction_it_shows() {
        let asset = Asset::from_chain(Chain::Bitcoin);
        let asset_id = asset.id.clone();
        let auto = PriceAlert::new_auto(asset_id.clone(), Currency::USD);
        assert_eq!(
            price_alert_row(&data(&auto, &asset, Some(100.0), Some(-2.0)), Currency::EUR),
            GemPriceAlertRow {
                asset_id: asset.id.clone(),
                title: asset.name.clone(),
                symbol: asset.symbol.clone(),
                kind: GemPriceAlertKind::Auto,
                prefix: GemPriceAlertText::Number {
                    value: GemFormattedNumber::currency(100.0, Currency::EUR, GemCurrencyStyle::Currency)
                },
                suffix: GemPriceAlertText::Number {
                    value: GemFormattedNumber::percentage(-2.0, GemPercentageStyle::Signed)
                },
                direction: Some(PriceAlertDirection::Down),
            }
        );
        assert_eq!(
            price_alert_row(&data(&auto, &asset, Some(100.0), None), Currency::USD).direction,
            None,
            "no change is neutral"
        );

        let over = PriceAlert::new_price(asset_id.clone(), Currency::USD, 120.0, PriceAlertDirection::Up);
        assert_eq!(
            price_alert_row(&data(&over, &asset, Some(100.0), Some(-2.0)), Currency::EUR),
            GemPriceAlertRow {
                asset_id: asset.id.clone(),
                title: asset.name.clone(),
                symbol: asset.symbol.clone(),
                kind: GemPriceAlertKind::Over,
                prefix: GemPriceAlertText::Label { label: GemPriceAlertLabel::Over },
                suffix: GemPriceAlertText::Number {
                    value: GemFormattedNumber::currency(120.0, Currency::USD, GemCurrencyStyle::Currency)
                },
                direction: Some(PriceAlertDirection::Up),
            },
            "the alert's own direction wins over the day's change"
        );

        let increase = PriceAlert::new_price_percent(asset_id.clone(), Currency::USD, 5.0, PriceAlertDirection::Up);
        let increase_row = price_alert_row(&data(&increase, &asset, Some(100.0), None), Currency::USD);
        assert_eq!(increase_row.kind, GemPriceAlertKind::Increase);
        assert_eq!(
            (increase_row.prefix, increase_row.suffix),
            (
                GemPriceAlertText::Label {
                    label: GemPriceAlertLabel::IncreasesBy
                },
                GemPriceAlertText::Number {
                    value: GemFormattedNumber::percentage(5.0, GemPercentageStyle::Unsigned)
                }
            ),
            "a percent alert prices off the market and shows its own target"
        );

        let under = PriceAlert::new_price(asset_id.clone(), Currency::USD, 80.0, PriceAlertDirection::Down);
        assert_eq!(
            price_alert_row(&data(&under, &asset, Some(100.0), Some(2.0)), Currency::USD),
            GemPriceAlertRow {
                asset_id: asset.id.clone(),
                title: asset.name.clone(),
                symbol: asset.symbol.clone(),
                kind: GemPriceAlertKind::Under,
                prefix: GemPriceAlertText::Label { label: GemPriceAlertLabel::Under },
                suffix: GemPriceAlertText::Number {
                    value: GemFormattedNumber::currency(80.0, Currency::USD, GemCurrencyStyle::Currency)
                },
                direction: Some(PriceAlertDirection::Down),
            }
        );

        let decrease = PriceAlert::new_price_percent(asset_id.clone(), Currency::USD, 5.0, PriceAlertDirection::Down);
        assert_eq!(
            price_alert_row(&data(&decrease, &asset, Some(100.0), None), Currency::USD).kind,
            GemPriceAlertKind::Decrease
        );

        assert_eq!(
            price_alert_row(&data(&auto, &asset, None, None), Currency::USD).direction,
            None,
            "an alert with no price to compare shows no direction"
        );

        let priced = PriceAlert {
            price: Some(120.0),
            ..auto.clone()
        };
        assert_eq!(
            price_alert_row(&data(&priced, &asset, Some(100.0), Some(-2.0)), Currency::USD),
            GemPriceAlertRow {
                asset_id: asset.id.clone(),
                title: asset.name.clone(),
                symbol: asset.symbol.clone(),
                kind: GemPriceAlertKind::Auto,
                prefix: GemPriceAlertText::Number {
                    value: GemFormattedNumber::currency(120.0, Currency::USD, GemCurrencyStyle::Currency)
                },
                suffix: GemPriceAlertText::Number {
                    value: GemFormattedNumber::percentage(-2.0, GemPercentageStyle::Signed)
                },
                direction: Some(PriceAlertDirection::Up),
            },
            "a target without a direction points at the price it waits for"
        );
    }

    #[test]
    fn test_reconcile_deletes_stale_and_keeps_changed_alerts() {
        let local = vec![alert(1.0, None), alert(2.0, Some(10)), alert(3.0, None)];
        let remote = vec![alert(1.0, None), alert(2.0, Some(20)), alert(4.0, None)];

        let sync = reconcile(local, remote);

        assert_eq!(sync.delete_ids, vec![alert(3.0, None).id()]);
        assert_eq!(sync.alerts.iter().map(|alert| alert.price).collect::<Vec<_>>(), vec![Some(2.0), Some(4.0)]);
    }

    #[test]
    fn test_displayed_price_alert_ids_drops_notified_alerts_and_sorts_by_price_then_direction_then_percent() {
        let asset_id = AssetId::from_chain(Chain::Ethereum);
        let high = PriceAlert::new_price(asset_id.clone(), Currency::USD, 3000.0, PriceAlertDirection::Down);
        let low_up = PriceAlert::new_price(asset_id.clone(), Currency::USD, 100.0, PriceAlertDirection::Up);
        let low_down = PriceAlert::new_price(asset_id.clone(), Currency::USD, 100.0, PriceAlertDirection::Down);
        let percent = PriceAlert::new_price_percent(asset_id.clone(), Currency::USD, 5.0, PriceAlertDirection::Up);
        let auto = PriceAlert::new_auto(asset_id.clone(), Currency::USD);
        let mut notified = PriceAlert::new_price(asset_id.clone(), Currency::USD, 5000.0, PriceAlertDirection::Up);
        notified.last_notified_at = DateTime::<Utc>::from_timestamp(10, 0);
        let mut notified_auto = PriceAlert::new_auto(AssetId::from_chain(Chain::Bitcoin), Currency::USD);
        notified_auto.last_notified_at = DateTime::<Utc>::from_timestamp(10, 0);

        let displayed = displayed_price_alert_ids(vec![
            percent.clone(),
            notified.clone(),
            low_down.clone(),
            auto.clone(),
            high.clone(),
            notified_auto.clone(),
            low_up.clone(),
        ]);

        assert_eq!(displayed, vec![high.id(), low_up.id(), low_down.id(), percent.id(), auto.id(), notified_auto.id()]);
        assert!(!displayed.contains(&notified.id()));
    }

    #[test]
    fn test_alert_direction() {
        let up = Some(PriceAlertDirection::Up);
        let down = Some(PriceAlertDirection::Down);

        assert_eq!(alert_direction(PriceAlertNotificationType::Price, Some(200.0), Some(150.0), PriceAlertDirection::Up), up);
        assert_eq!(alert_direction(PriceAlertNotificationType::Price, Some(100.0), Some(150.0), PriceAlertDirection::Up), down);
        assert_eq!(alert_direction(PriceAlertNotificationType::Price, Some(150.0), Some(150.0), PriceAlertDirection::Up), None);

        assert_eq!(alert_direction(PriceAlertNotificationType::Price, Some(200.0), None, PriceAlertDirection::Up), None);
        assert_eq!(alert_direction(PriceAlertNotificationType::Price, Some(200.0), Some(0.0), PriceAlertDirection::Up), None);
        assert_eq!(alert_direction(PriceAlertNotificationType::Price, Some(200.0), Some(-1.0), PriceAlertDirection::Up), None);

        assert_eq!(alert_direction(PriceAlertNotificationType::Price, Some(-200.0), Some(150.0), PriceAlertDirection::Up), None);
        assert_eq!(alert_direction(PriceAlertNotificationType::Price, Some(0.0), Some(150.0), PriceAlertDirection::Up), None);
        assert_eq!(alert_direction(PriceAlertNotificationType::Price, None, Some(150.0), PriceAlertDirection::Up), None);
        assert_eq!(
            alert_direction(PriceAlertNotificationType::Price, Some(f64::NAN), Some(150.0), PriceAlertDirection::Up),
            None
        );

        assert_eq!(
            alert_direction(PriceAlertNotificationType::PricePercentChange, Some(5.0), None, PriceAlertDirection::Down),
            down
        );
        assert_eq!(
            alert_direction(PriceAlertNotificationType::PricePercentChange, Some(5.0), Some(150.0), PriceAlertDirection::Up),
            up
        );
        assert_eq!(
            alert_direction(PriceAlertNotificationType::PricePercentChange, Some(-5.0), Some(150.0), PriceAlertDirection::Up),
            None
        );

        assert_eq!(alert_direction(PriceAlertNotificationType::Auto, Some(5.0), Some(150.0), PriceAlertDirection::Up), None);
    }
}
