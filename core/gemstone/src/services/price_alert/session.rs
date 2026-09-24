use primitives::{AssetId, Currency, PriceAlert, PriceAlertDirection, PriceAlertNotificationType};

use super::rules;
use crate::formatted_number::GemFormattedNumber;
use crate::percentage::GemPercentageStyle;
use crate::precision::GemCurrencyStyle;
use number_formatter::price_suggestion;

const SUGGESTION_OFFSET_PERCENT: f64 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPriceAlertPrompt {
    TargetPrice,
    PriceOver,
    PriceUnder,
    IncreasesBy,
    DecreasesBy,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPriceAlertViewState {
    pub prompt: GemPriceAlertPrompt,
    pub notification_type: PriceAlertNotificationType,
    pub selected_direction: PriceAlertDirection,
    pub direction: Option<PriceAlertDirection>,
    pub can_confirm: bool,
    pub is_saving: bool,
    pub percentage_suggestions: Vec<GemFormattedNumber>,
    pub price_suggestions: Vec<GemFormattedNumber>,
    pub saved_value: Option<GemFormattedNumber>,
    pub current_price: Option<GemFormattedNumber>,
    pub price_change: Option<GemFormattedNumber>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPriceAlertSession {
    pub asset_id: AssetId,
    pub currency: Currency,
    pub notification_type: PriceAlertNotificationType,
    pub selected_direction: PriceAlertDirection,
    pub input: Option<f64>,
    pub current_price: Option<f64>,
    pub price_change: Option<f64>,
    pub is_saving: bool,
}

impl GemPriceAlertSession {
    pub fn new(asset_id: AssetId, currency: Currency) -> Self {
        Self {
            asset_id,
            currency,
            notification_type: PriceAlertNotificationType::Price,
            selected_direction: PriceAlertDirection::Up,
            input: None,
            current_price: None,
            price_change: None,
            is_saving: false,
        }
    }
}

#[uniffi::export]
impl GemPriceAlertSession {
    pub fn on_type(&self, notification_type: PriceAlertNotificationType) -> Self {
        Self { notification_type, ..self.clone() }
    }

    pub fn on_direction(&self, selected_direction: PriceAlertDirection) -> Self {
        Self { selected_direction, ..self.clone() }
    }

    pub fn on_input(&self, input: Option<f64>) -> Self {
        Self { input, ..self.clone() }
    }

    pub fn on_price(&self, current_price: Option<f64>, price_change: Option<f64>) -> Self {
        Self { current_price, price_change, ..self.clone() }
    }

    pub fn on_saving(&self, is_saving: bool) -> Self {
        Self { is_saving, ..self.clone() }
    }

    pub fn alert(&self) -> Option<PriceAlert> {
        let input = self.input?;
        let direction = self.direction()?;
        Some(match self.notification_type {
            PriceAlertNotificationType::PricePercentChange => PriceAlert::new_price_percent(self.asset_id.clone(), self.currency.clone(), input, direction),
            PriceAlertNotificationType::Price | PriceAlertNotificationType::Auto => PriceAlert::new_price(self.asset_id.clone(), self.currency.clone(), input, direction),
        })
    }

    pub fn view_state(&self) -> GemPriceAlertViewState {
        let price = self.current_price.filter(|price| *price > 0.0);
        GemPriceAlertViewState {
            prompt: self.prompt(),
            notification_type: self.notification_type.clone(),
            selected_direction: self.selected_direction.clone(),
            direction: self.direction(),
            can_confirm: !self.is_saving && self.direction().is_some(),
            is_saving: self.is_saving,
            percentage_suggestions: price
                .map(price_suggestion::percentage_suggestions)
                .unwrap_or_default()
                .into_iter()
                .map(|value| GemFormattedNumber::percentage(value as f64, GemPercentageStyle::UnsignedCompact))
                .collect(),
            price_suggestions: price
                .map(|price| price_suggestion::price_rounded_values(price, SUGGESTION_OFFSET_PERCENT))
                .unwrap_or_default()
                .into_iter()
                .map(|value| self.price_number(value))
                .collect(),
            saved_value: self.input.map(|input| self.input_number(input)),
            current_price: price.map(|price| self.price_number(price)),
            price_change: self.price_change.map(|change| GemFormattedNumber::percentage(change, GemPercentageStyle::Signed).toned()),
        }
    }
}

impl GemPriceAlertSession {
    fn price_number(&self, value: f64) -> GemFormattedNumber {
        GemFormattedNumber::currency(value, self.currency.clone(), GemCurrencyStyle::Currency)
    }

    fn input_number(&self, input: f64) -> GemFormattedNumber {
        match self.notification_type {
            PriceAlertNotificationType::PricePercentChange => GemFormattedNumber::percentage(input, GemPercentageStyle::UnsignedCompact),
            PriceAlertNotificationType::Price | PriceAlertNotificationType::Auto => self.price_number(input),
        }
    }

    fn prompt(&self) -> GemPriceAlertPrompt {
        match self.notification_type {
            PriceAlertNotificationType::PricePercentChange => match self.selected_direction {
                PriceAlertDirection::Up => GemPriceAlertPrompt::IncreasesBy,
                PriceAlertDirection::Down => GemPriceAlertPrompt::DecreasesBy,
            },
            PriceAlertNotificationType::Price | PriceAlertNotificationType::Auto => match self.direction() {
                Some(PriceAlertDirection::Up) => GemPriceAlertPrompt::PriceOver,
                Some(PriceAlertDirection::Down) => GemPriceAlertPrompt::PriceUnder,
                None => GemPriceAlertPrompt::TargetPrice,
            },
        }
    }

    fn direction(&self) -> Option<PriceAlertDirection> {
        rules::alert_direction(self.notification_type.clone(), self.input, self.current_price, self.selected_direction.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatted_number::GemNumberUnit;

    #[test]
    fn test_the_prompt_follows_the_type_and_the_resolved_direction() {
        let session = GemPriceAlertSession::new(AssetId::from_chain(primitives::Chain::Ethereum), Currency::USD);
        assert_eq!(session.view_state().prompt, GemPriceAlertPrompt::TargetPrice, "a price alert with no input asks for a target");

        let priced = GemPriceAlertSession {
            current_price: Some(100.0),
            input: Some(120.0),
            ..session.clone()
        };
        assert_eq!(priced.view_state().prompt, GemPriceAlertPrompt::PriceOver);
        assert_eq!(GemPriceAlertSession { input: Some(80.0), ..priced.clone() }.view_state().prompt, GemPriceAlertPrompt::PriceUnder);

        let percentage = priced.on_type(PriceAlertNotificationType::PricePercentChange);
        assert_eq!(percentage.view_state().prompt, GemPriceAlertPrompt::IncreasesBy);
        assert_eq!(percentage.on_direction(PriceAlertDirection::Down).view_state().prompt, GemPriceAlertPrompt::DecreasesBy);
    }

    #[test]
    fn test_without_an_input_there_is_no_direction_and_nothing_to_confirm() {
        let state = GemPriceAlertSession::mock().view_state();

        assert_eq!(state.direction, None);
        assert!(!state.can_confirm);
        assert!(GemPriceAlertSession::mock().alert().is_none());
    }

    #[test]
    fn test_a_price_alert_carries_the_price_and_a_percentage_alert_the_percentage() {
        let price = GemPriceAlertSession::mock().on_input(Some(120.0)).alert().unwrap();
        let percent = GemPriceAlertSession::mock().on_type(PriceAlertNotificationType::PricePercentChange).on_input(Some(5.0)).alert().unwrap();

        assert_eq!((price.price, price.price_percent_change), (Some(120.0), None));
        assert_eq!((percent.price, percent.price_percent_change), (None, Some(5.0)));
    }

    #[test]
    fn test_an_alert_is_keyed_by_what_it_watches_and_never_takes_the_auto_alert_key() {
        let session = GemPriceAlertSession::mock();
        let auto = PriceAlert::new_auto(session.asset_id.clone(), Currency::USD);
        let price = session.on_input(Some(120.0)).alert().unwrap();
        let other_price = session.on_input(Some(130.0)).alert().unwrap();
        let percent = session.on_type(PriceAlertNotificationType::PricePercentChange).on_input(Some(5.0)).alert().unwrap();

        assert_eq!(price.id(), "ethereum_USD_120_up");
        assert_eq!(percent.id(), "ethereum_USD_5_up");
        assert_ne!(price.id(), auto.id());
        assert_ne!(price.id(), other_price.id());
    }

    #[test]
    fn test_saving_blocks_a_second_confirm() {
        let ready = GemPriceAlertSession::mock().on_input(Some(120.0));

        assert!(ready.view_state().can_confirm);
        assert!(!ready.on_saving(true).view_state().can_confirm);
    }

    #[test]
    fn test_the_saved_value_carries_the_unit_the_alert_was_set_in() {
        let price = GemPriceAlertSession::mock().on_input(Some(120.0));
        let percent = price.on_type(PriceAlertNotificationType::PricePercentChange).on_input(Some(5.0));

        assert_eq!(price.view_state().saved_value.map(|value| value.unit), Some(GemNumberUnit::Currency { code: Currency::USD.to_string() }));
        assert_eq!(percent.view_state().saved_value.map(|value| value.unit), Some(GemNumberUnit::Percent));
        assert_eq!(percent.view_state().saved_value.map(|value| value.value), Some(5.0));
        assert_eq!(GemPriceAlertSession::mock().view_state().saved_value, None, "nothing typed names no value");
        assert!(
            price.view_state().percentage_suggestions.iter().all(|value| value.unit == GemNumberUnit::Percent),
            "a percentage suggestion carries its unit instead of a pasted %"
        );
    }

    #[test]
    fn test_suggestions_need_a_price_to_offset_from() {
        let without_price = GemPriceAlertSession::new(AssetId::from_chain(primitives::Chain::Ethereum), Currency::USD);

        assert!(without_price.view_state().price_suggestions.is_empty());
        assert!(without_price.on_price(Some(0.0), None).view_state().percentage_suggestions.is_empty());
        assert!(!GemPriceAlertSession::mock().view_state().price_suggestions.is_empty());
    }

    #[test]
    fn test_the_current_price_and_its_change_come_formatted() {
        let session = GemPriceAlertSession::new(AssetId::from_chain(primitives::Chain::Ethereum), Currency::USD).on_price(Some(100.0), Some(-2.5));
        let state = session.view_state();

        assert_eq!(state.current_price.map(|price| price.value), Some(100.0));
        let change = state.price_change.unwrap();
        assert_eq!((change.value, change.tone), (-2.5, crate::formatted_number::GemValueTone::Negative));
        assert_eq!(session.on_price(Some(0.0), None).view_state().current_price, None, "a zero price is no price");
    }
}
