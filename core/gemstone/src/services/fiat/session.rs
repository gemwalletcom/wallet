use primitives::{FiatQuote, FiatQuoteType};

use super::model::GemFiatAmountCheck;
use super::rules;
use crate::config::fiat_config::get_fiat_config;
use crate::models::custom_types::GemBigUint;
use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatQuoteRequest {
    pub quote_type: FiatQuoteType,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatQuotesResult {
    pub request: GemFiatQuoteRequest,
    pub quotes: Vec<FiatQuote>,
    pub error: Option<GemServiceError>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemFiatQuotePhase {
    NoInput,
    InvalidInput,
    Invalid { check: GemFiatAmountCheck },
    Loading { amount: f64 },
    Ready,
    NoQuotes,
    Failed { error: GemServiceError },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemFiatButtonAction {
    Continue,
    RetryQuote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemFiatButtonState {
    Disabled,
    Loading,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatOperation {
    pub quote_type: FiatQuoteType,
    pub amount: String,
    pub quotes: Vec<FiatQuote>,
    pub selected_provider: Option<String>,
    pub phase: GemFiatQuotePhase,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatSession {
    pub quote_type: FiatQuoteType,
    pub buy: GemFiatOperation,
    pub sell: GemFiatOperation,
    pub available: GemBigUint,
}

impl GemFiatSession {
    pub fn new(quote_type: FiatQuoteType, amount: Option<u32>) -> Self {
        let config = get_fiat_config();
        let operation = |operation_type: FiatQuoteType| {
            let default = rules::default_amount(&config, operation_type);
            let initial = amount.filter(|_| operation_type == quote_type).unwrap_or(default);
            GemFiatOperation::new(operation_type, initial.to_string())
        };
        Self {
            quote_type,
            buy: operation(FiatQuoteType::Buy),
            sell: operation(FiatQuoteType::Sell),
            available: GemBigUint::default(),
        }
    }

    fn operation(&self, quote_type: FiatQuoteType) -> &GemFiatOperation {
        match quote_type {
            FiatQuoteType::Buy => &self.buy,
            FiatQuoteType::Sell => &self.sell,
        }
    }

    fn with_operation(&self, operation: GemFiatOperation) -> Self {
        match operation.quote_type {
            FiatQuoteType::Buy => Self { buy: operation, ..self.clone() },
            FiatQuoteType::Sell => Self { sell: operation, ..self.clone() },
        }
    }
}

#[uniffi::export]
impl GemFiatSession {
    pub fn current(&self) -> GemFiatOperation {
        self.operation(self.quote_type).clone()
    }

    pub fn on_type_changed(&self, quote_type: FiatQuoteType) -> GemFiatSession {
        GemFiatSession { quote_type, ..self.clone() }
    }

    pub fn on_sell_enabled_changed(&self, is_sell_enabled: bool) -> GemFiatSession {
        if is_sell_enabled || self.quote_type != FiatQuoteType::Sell {
            return self.clone();
        }
        GemFiatSession {
            quote_type: FiatQuoteType::Buy,
            buy: self.buy.on_amount_changed(self.sell.amount.clone()),
            ..self.clone()
        }
    }

    pub fn on_amount_changed(&self, amount: String) -> GemFiatSession {
        self.with_operation(self.current().on_amount_changed(amount))
    }

    pub fn on_balance_changed(&self, available: GemBigUint) -> GemFiatSession {
        GemFiatSession { available, ..self.clone() }
    }

    pub fn quote_request(&self) -> Option<GemFiatQuoteRequest> {
        self.current().quote_request()
    }

    pub fn on_fetch_started(&self, request: GemFiatQuoteRequest) -> GemFiatSession {
        self.with_operation(self.operation(request.quote_type).on_fetch_started(&request))
    }

    pub fn on_quote_results(&self, results: GemFiatQuotesResult) -> GemFiatSession {
        self.with_operation(self.operation(results.request.quote_type).on_quote_results(results))
    }

    pub fn on_provider_selected(&self, provider: String) -> GemFiatSession {
        self.with_operation(self.current().on_provider_selected(provider))
    }

    pub fn selected_quote(&self) -> Option<FiatQuote> {
        self.current().selected_quote()
    }

    pub fn can_select_provider(&self) -> bool {
        self.current().quotes.len() > 1
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.current().phase, GemFiatQuotePhase::Loading { .. })
    }

    pub fn amount_check(&self) -> GemFiatAmountCheck {
        let operation = self.current();
        match operation.parsed_amount() {
            Some(amount) => rules::amount_check(&get_fiat_config(), operation.quote_type, amount, operation.selected_quote().as_ref(), &self.available),
            None => GemFiatAmountCheck::Valid,
        }
    }

    pub fn button_action(&self) -> GemFiatButtonAction {
        match self.current().phase {
            GemFiatQuotePhase::Failed { .. } => GemFiatButtonAction::RetryQuote,
            _ => GemFiatButtonAction::Continue,
        }
    }

    pub fn button_state(&self, is_url_loading: bool) -> GemFiatButtonState {
        if is_url_loading {
            return GemFiatButtonState::Loading;
        }
        match self.current().phase {
            GemFiatQuotePhase::Loading { .. } => GemFiatButtonState::Loading,
            GemFiatQuotePhase::Failed { .. } => GemFiatButtonState::Enabled,
            GemFiatQuotePhase::Ready if self.selected_quote().is_some() && self.amount_check() == GemFiatAmountCheck::Valid => GemFiatButtonState::Enabled,
            _ => GemFiatButtonState::Disabled,
        }
    }
}

impl GemFiatOperation {
    fn new(quote_type: FiatQuoteType, amount: String) -> Self {
        Self {
            quote_type,
            phase: Self::input_phase(quote_type, &amount),
            amount,
            quotes: vec![],
            selected_provider: None,
        }
    }

    fn input_phase(quote_type: FiatQuoteType, amount: &str) -> GemFiatQuotePhase {
        match rules::parse_amount(amount) {
            rules::FiatAmountInput::Empty => GemFiatQuotePhase::NoInput,
            rules::FiatAmountInput::Invalid => GemFiatQuotePhase::InvalidInput,
            rules::FiatAmountInput::Value(value) => match rules::amount_check(&get_fiat_config(), quote_type, value, None, &Default::default()) {
                GemFiatAmountCheck::Valid => GemFiatQuotePhase::Loading { amount: value },
                check => GemFiatQuotePhase::Invalid { check },
            },
        }
    }

    fn parsed_amount(&self) -> Option<f64> {
        match rules::parse_amount(&self.amount) {
            rules::FiatAmountInput::Value(value) => Some(value),
            rules::FiatAmountInput::Empty | rules::FiatAmountInput::Invalid => None,
        }
    }

    fn on_amount_changed(&self, amount: String) -> Self {
        if amount == self.amount {
            return self.clone();
        }
        Self {
            selected_provider: self.selected_provider.clone(),
            ..Self::new(self.quote_type, amount)
        }
    }

    fn quote_request(&self) -> Option<GemFiatQuoteRequest> {
        match self.phase {
            GemFiatQuotePhase::NoInput | GemFiatQuotePhase::InvalidInput | GemFiatQuotePhase::Invalid { .. } => None,
            GemFiatQuotePhase::Loading { .. } | GemFiatQuotePhase::Ready | GemFiatQuotePhase::NoQuotes | GemFiatQuotePhase::Failed { .. } => Some(GemFiatQuoteRequest {
                quote_type: self.quote_type,
                amount: self.parsed_amount()?,
            }),
        }
    }

    fn on_fetch_started(&self, request: &GemFiatQuoteRequest) -> Self {
        if self.parsed_amount() != Some(request.amount) {
            return self.clone();
        }
        Self {
            quotes: vec![],
            phase: GemFiatQuotePhase::Loading { amount: request.amount },
            ..self.clone()
        }
    }

    fn on_quote_results(&self, results: GemFiatQuotesResult) -> Self {
        if self.phase != (GemFiatQuotePhase::Loading { amount: results.request.amount }) {
            return self.clone();
        }
        let (quotes, phase) = match results.error {
            Some(error) => (vec![], GemFiatQuotePhase::Failed { error }),
            None if results.quotes.is_empty() => (vec![], GemFiatQuotePhase::NoQuotes),
            None => (results.quotes, GemFiatQuotePhase::Ready),
        };
        Self { quotes, phase, ..self.clone() }
    }

    fn on_provider_selected(&self, provider: String) -> Self {
        if !self.quotes.iter().any(|quote| quote.provider.id.id() == provider) {
            return self.clone();
        }
        Self {
            selected_provider: Some(provider),
            ..self.clone()
        }
    }

    fn selected_quote(&self) -> Option<FiatQuote> {
        rules::selected_quote(&self.quotes, self.selected_provider.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::{Asset, Chain, FiatProvider, FiatProviderName};

    fn quote(provider: FiatProviderName, crypto_amount: f64) -> FiatQuote {
        FiatQuote::new(
            format!("{}-{crypto_amount}", provider.id()),
            Asset::from_chain(Chain::Ethereum),
            FiatProvider {
                id: provider,
                name: provider.name().to_string(),
                image_url: None,
                priority: None,
                threshold_bps: None,
                enabled: true,
                buy_enabled: true,
                sell_enabled: true,
                payment_methods: vec![],
            },
            FiatQuoteType::Buy,
            100.0,
            "USD".to_string(),
            crypto_amount,
            BigUint::ZERO,
            10,
            vec![],
        )
    }

    fn results(quote_type: FiatQuoteType, amount: f64, quotes: Vec<FiatQuote>) -> GemFiatQuotesResult {
        GemFiatQuotesResult {
            request: GemFiatQuoteRequest { quote_type, amount },
            quotes,
            error: None,
        }
    }

    fn ready(quotes: Vec<FiatQuote>) -> GemFiatSession {
        let session = GemFiatSession::new(FiatQuoteType::Buy, None);
        session.on_quote_results(results(FiatQuoteType::Buy, 50.0, quotes))
    }

    #[test]
    fn test_a_new_session_starts_each_type_on_its_default_and_the_initial_amount_on_its_type() {
        let session = GemFiatSession::new(FiatQuoteType::Sell, Some(25));

        assert_eq!(session.quote_type, FiatQuoteType::Sell);
        assert_eq!(session.buy.amount, "50");
        assert_eq!(session.sell.amount, "25");
        assert_eq!(
            session.quote_request(),
            Some(GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Sell,
                amount: 25.0
            })
        );
        assert_eq!(session.on_type_changed(FiatQuoteType::Buy).quote_request().map(|request| request.amount), Some(50.0));
    }

    #[test]
    fn test_the_amount_decides_the_phase_and_whether_a_quote_is_requested() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, None);

        assert_eq!(session.on_amount_changed("".to_string()).current().phase, GemFiatQuotePhase::NoInput);
        assert_eq!(session.on_amount_changed("0".to_string()).current().phase, GemFiatQuotePhase::NoInput);
        assert_eq!(session.on_amount_changed("abc".to_string()).current().phase, GemFiatQuotePhase::InvalidInput);
        assert_eq!(
            session.on_amount_changed("4".to_string()).current().phase,
            GemFiatQuotePhase::Invalid {
                check: GemFiatAmountCheck::BelowMinimum { minimum: 5 }
            }
        );
        assert_eq!(session.on_amount_changed("4".to_string()).quote_request(), None);
        assert_eq!(session.on_amount_changed("12,5".to_string()).current().phase, GemFiatQuotePhase::Loading { amount: 12.5 });
        assert_eq!(session.on_amount_changed("4".to_string()).button_state(false), GemFiatButtonState::Disabled);
    }

    #[test]
    fn test_changing_the_amount_clears_quotes_and_keeps_the_same_amount_untouched() {
        let session = ready(vec![quote(FiatProviderName::Transak, 1.0)]);

        assert_eq!(session.on_amount_changed("50".to_string()), session);
        let changed = session.on_amount_changed("75".to_string());
        assert!(changed.current().quotes.is_empty());
        assert_eq!(changed.current().phase, GemFiatQuotePhase::Loading { amount: 75.0 });
        assert_eq!(changed.selected_quote(), None);
        assert_eq!(changed.button_state(false), GemFiatButtonState::Loading);
    }

    #[test]
    fn test_quote_results_only_apply_to_the_amount_still_loading() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, None);
        let stale = session.on_quote_results(results(FiatQuoteType::Buy, 40.0, vec![quote(FiatProviderName::Transak, 1.0)]));
        assert_eq!(stale, session);

        let ready = session.on_quote_results(results(FiatQuoteType::Buy, 50.0, vec![quote(FiatProviderName::Transak, 1.0)]));
        assert_eq!(ready.current().phase, GemFiatQuotePhase::Ready);
        assert_eq!(ready.selected_quote().map(|quote| quote.provider.id), Some(FiatProviderName::Transak));
        assert_eq!(ready.button_state(false), GemFiatButtonState::Enabled);
        assert_eq!(ready.button_action(), GemFiatButtonAction::Continue);

        let refreshed = ready.on_fetch_started(GemFiatQuoteRequest {
            quote_type: FiatQuoteType::Buy,
            amount: 50.0,
        });
        assert_eq!(refreshed.current().phase, GemFiatQuotePhase::Loading { amount: 50.0 });
        assert!(refreshed.current().quotes.is_empty());
        assert_eq!(
            ready.on_fetch_started(GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Buy,
                amount: 60.0
            }),
            ready
        );
        assert_eq!(ready.on_quote_results(results(FiatQuoteType::Buy, 50.0, vec![])), ready);
    }

    #[test]
    fn test_empty_quotes_and_failures_are_distinct_and_only_a_failure_offers_a_retry() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, None);

        let empty = session.on_quote_results(results(FiatQuoteType::Buy, 50.0, vec![]));
        assert_eq!(empty.current().phase, GemFiatQuotePhase::NoQuotes);
        assert_eq!(empty.button_state(false), GemFiatButtonState::Disabled);
        assert_eq!(empty.button_action(), GemFiatButtonAction::Continue);
        assert!(empty.quote_request().is_some());

        let failed = session.on_quote_results(GemFiatQuotesResult {
            error: Some(GemServiceError::Api { msg: "offline".to_string() }),
            ..results(FiatQuoteType::Buy, 50.0, vec![quote(FiatProviderName::Transak, 1.0)])
        });
        assert_eq!(
            failed.current().phase,
            GemFiatQuotePhase::Failed {
                error: GemServiceError::Api { msg: "offline".to_string() }
            }
        );
        assert!(failed.current().quotes.is_empty());
        assert_eq!(failed.button_state(false), GemFiatButtonState::Enabled);
        assert_eq!(failed.button_action(), GemFiatButtonAction::RetryQuote);
        assert_eq!(failed.button_state(true), GemFiatButtonState::Loading);
    }

    #[test]
    fn test_the_chosen_provider_survives_a_refresh_and_an_unknown_one_is_ignored() {
        let session = ready(vec![quote(FiatProviderName::Transak, 1.0), quote(FiatProviderName::MoonPay, 2.0)]);
        assert!(session.can_select_provider());

        let selected = session.on_provider_selected("moonpay".to_string());
        assert_eq!(selected.selected_quote().map(|quote| quote.provider.id), Some(FiatProviderName::MoonPay));
        assert_eq!(session.on_provider_selected("banxa".to_string()), session);

        let refreshed = selected
            .on_fetch_started(GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Buy,
                amount: 50.0,
            })
            .on_quote_results(results(
                FiatQuoteType::Buy,
                50.0,
                vec![quote(FiatProviderName::MoonPay, 3.0), quote(FiatProviderName::Transak, 1.0)],
            ));
        assert_eq!(refreshed.selected_quote().map(|quote| quote.crypto_amount), Some(3.0));

        let gone = selected
            .on_fetch_started(GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Buy,
                amount: 50.0,
            })
            .on_quote_results(results(FiatQuoteType::Buy, 50.0, vec![quote(FiatProviderName::Transak, 1.0)]));
        assert_eq!(gone.selected_quote().map(|quote| quote.provider.id), Some(FiatProviderName::Transak));
        assert!(!gone.can_select_provider());
    }

    #[test]
    fn test_a_sell_quote_above_the_balance_disables_the_button_until_the_balance_covers_it() {
        let mut sell = quote(FiatProviderName::Transak, 1.0);
        sell.quote_type = FiatQuoteType::Sell;
        let session = GemFiatSession::new(FiatQuoteType::Sell, None).on_quote_results(results(FiatQuoteType::Sell, 100.0, vec![sell]));

        assert_eq!(session.current().phase, GemFiatQuotePhase::Ready);
        assert!(matches!(session.amount_check(), GemFiatAmountCheck::InsufficientBalance { .. }));
        assert_eq!(session.button_state(false), GemFiatButtonState::Disabled);

        let funded = session.on_balance_changed(BigUint::from(1_000_000_000_000_000_000u64));
        assert_eq!(funded.amount_check(), GemFiatAmountCheck::Valid);
        assert_eq!(funded.button_state(false), GemFiatButtonState::Enabled);
    }

    #[test]
    fn test_losing_sell_support_moves_to_buy_with_the_sell_amount() {
        let session = GemFiatSession::new(FiatQuoteType::Sell, Some(25));

        assert_eq!(session.on_sell_enabled_changed(true), session);
        let buy = session.on_sell_enabled_changed(false);
        assert_eq!(buy.quote_type, FiatQuoteType::Buy);
        assert_eq!(buy.buy.amount, "25");
        assert_eq!(
            buy.quote_request(),
            Some(GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Buy,
                amount: 25.0
            })
        );
        assert_eq!(GemFiatSession::new(FiatQuoteType::Buy, None).on_sell_enabled_changed(false).quote_type, FiatQuoteType::Buy);
    }
}
