use primitives::{AssetId, Currency, PriceAlert, PriceAlertDirection, PriceAlertNotificationType};

use super::rules;
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
    pub percentage_suggestions: Vec<i32>,
    pub price_suggestions: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPriceAlertSession {
    pub asset_id: AssetId,
    pub currency: Currency,
    pub notification_type: PriceAlertNotificationType,
    pub selected_direction: PriceAlertDirection,
    pub input: Option<f64>,
    pub current_price: Option<f64>,
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
            is_saving: false,
        }
    }
}

#[uniffi::export]
impl GemPriceAlertSession {
    pub fn on_type(&self, notification_type: PriceAlertNotificationType) -> Self {
        Self {
            notification_type,
            ..self.clone()
        }
    }

    pub fn on_direction(&self, selected_direction: PriceAlertDirection) -> Self {
        Self {
            selected_direction,
            ..self.clone()
        }
    }

    pub fn on_input(&self, input: Option<f64>) -> Self {
        Self { input, ..self.clone() }
    }

    pub fn on_price(&self, current_price: Option<f64>) -> Self {
        Self { current_price, ..self.clone() }
    }

    pub fn on_saving(&self, is_saving: bool) -> Self {
        Self { is_saving, ..self.clone() }
    }

    pub fn alert(&self) -> Option<PriceAlert> {
        let input = self.input?;
        let direction = self.direction()?;
        let (price, price_percent_change) = match self.notification_type {
            PriceAlertNotificationType::PricePercentChange => (None, Some(input)),
            PriceAlertNotificationType::Price | PriceAlertNotificationType::Auto => (Some(input), None),
        };
        Some(PriceAlert {
            identifier: self.asset_id.to_string(),
            asset_id: self.asset_id.clone(),
            currency: self.currency.clone(),
            price,
            price_percent_change,
            price_direction: Some(direction),
            last_notified_at: None,
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
            percentage_suggestions: price.map(price_suggestion::percentage_suggestions).unwrap_or_default(),
            price_suggestions: price
                .map(|price| price_suggestion::price_rounded_values(price, SUGGESTION_OFFSET_PERCENT))
                .unwrap_or_default(),
        }
    }
}

impl GemPriceAlertSession {
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

    #[test]
    fn test_the_prompt_follows_the_type_and_the_resolved_direction() {
        let session = GemPriceAlertSession::new(AssetId::from_chain(primitives::Chain::Ethereum), Currency::USD);
        assert_eq!(
            session.view_state().prompt,
            GemPriceAlertPrompt::TargetPrice,
            "a price alert with no input asks for a target"
        );

        let priced = GemPriceAlertSession {
            current_price: Some(100.0),
            input: Some(120.0),
            ..session.clone()
        };
        assert_eq!(priced.view_state().prompt, GemPriceAlertPrompt::PriceOver);
        assert_eq!(
            GemPriceAlertSession {
                input: Some(80.0),
                ..priced.clone()
            }
            .view_state()
            .prompt,
            GemPriceAlertPrompt::PriceUnder
        );

        let percentage = priced.on_type(PriceAlertNotificationType::PricePercentChange);
        assert_eq!(percentage.view_state().prompt, GemPriceAlertPrompt::IncreasesBy);
        assert_eq!(percentage.on_direction(PriceAlertDirection::Down).view_state().prompt, GemPriceAlertPrompt::DecreasesBy);
    }

    fn session() -> GemPriceAlertSession {
        GemPriceAlertSession::new(AssetId::from_chain(primitives::Chain::Ethereum), Currency::USD).on_price(Some(100.0))
    }

    #[test]
    fn test_without_an_input_there_is_no_direction_and_nothing_to_confirm() {
        let state = session().view_state();

        assert_eq!(state.direction, None);
        assert!(!state.can_confirm);
        assert!(session().alert().is_none());
    }

    #[test]
    fn test_a_price_alert_carries_the_price_and_a_percentage_alert_the_percentage() {
        let price = session().on_input(Some(120.0)).alert().unwrap();
        let percent = session().on_type(PriceAlertNotificationType::PricePercentChange).on_input(Some(5.0)).alert().unwrap();

        assert_eq!((price.price, price.price_percent_change), (Some(120.0), None));
        assert_eq!((percent.price, percent.price_percent_change), (None, Some(5.0)));
    }

    #[test]
    fn test_saving_blocks_a_second_confirm() {
        let ready = session().on_input(Some(120.0));

        assert!(ready.view_state().can_confirm);
        assert!(!ready.on_saving(true).view_state().can_confirm);
    }

    #[test]
    fn test_suggestions_need_a_price_to_offset_from() {
        let without_price = GemPriceAlertSession::new(AssetId::from_chain(primitives::Chain::Ethereum), Currency::USD);

        assert!(without_price.view_state().price_suggestions.is_empty());
        assert!(without_price.on_price(Some(0.0)).view_state().percentage_suggestions.is_empty());
        assert!(!session().view_state().price_suggestions.is_empty());
    }
}
