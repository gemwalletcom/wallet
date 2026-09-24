use crate::models::button::GemButtonState;
use primitives::{FiatProviderName, FiatQuote, FiatQuoteType};

use super::model::{GemFiatAmountCheck, GemFiatQuoteRow};
use super::rules;
use crate::config::fiat_config::get_fiat_config;
use crate::models::custom_types::GemBigUint;
use crate::models::list::{GemListRow, GemListRowTitle};
use crate::services::error::GemServiceError;
use crate::services::error_text::GemErrorText;

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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemFiatQuotesMessage {
    EnterAmount,
    NoResults,
    Failed { error: GemErrorText },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemFiatButtonAction {
    Continue,
    RetryQuote,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatOperation {
    pub quote_type: FiatQuoteType,
    pub amount: String,
    pub quotes: Vec<FiatQuote>,
    pub selected_provider: Option<FiatProviderName>,
    pub phase: GemFiatQuotePhase,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatViewState {
    pub quote_type: FiatQuoteType,
    pub amount: String,
    pub phase: GemFiatQuotePhase,
    pub quote_rows: Vec<GemFiatQuoteRow>,
    pub selected_quote_row: Option<GemFiatQuoteRow>,
    pub rate_row: Option<GemListRow>,
    pub can_select_provider: bool,
    pub amount_check: GemFiatAmountCheck,
    pub button_action: GemFiatButtonAction,
    pub button_state: GemButtonState,
    pub shows_type_picker: bool,
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
impl GemFiatViewState {
    pub fn quotes_message(&self) -> Option<GemFiatQuotesMessage> {
        match &self.phase {
            GemFiatQuotePhase::NoInput | GemFiatQuotePhase::InvalidInput => Some(GemFiatQuotesMessage::EnterAmount),
            GemFiatQuotePhase::Invalid { .. } | GemFiatQuotePhase::NoQuotes => Some(GemFiatQuotesMessage::NoResults),
            GemFiatQuotePhase::Failed { error } => Some(GemFiatQuotesMessage::Failed { error: error.text() }),
            GemFiatQuotePhase::Loading { .. } | GemFiatQuotePhase::Ready => None,
        }
    }
}

#[uniffi::export]
impl GemFiatSession {
    pub fn view_state(&self, asset_price: Option<f64>, is_url_loading: bool, is_sell_enabled: bool) -> GemFiatViewState {
        let operation = self.current();
        let selected_quote_row = self.selected_quote_row(asset_price);
        GemFiatViewState {
            quote_type: operation.quote_type,
            amount: operation.amount.clone(),
            phase: operation.phase.clone(),
            quote_rows: self.quote_rows(asset_price),
            rate_row: selected_quote_row.as_ref().and_then(|row| row.rate.clone()).map(|rate| GemListRow::Rate { title: GemListRowTitle::Rate, rate }),
            selected_quote_row,
            can_select_provider: self.can_select_provider(),
            amount_check: self.amount_check(),
            button_action: self.button_action(),
            button_state: self.button_state(is_url_loading),
            shows_type_picker: is_sell_enabled,
        }
    }

    fn current(&self) -> GemFiatOperation {
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

    pub fn refreshes_quotes(&self, is_screen_active: bool) -> bool {
        is_screen_active && !matches!(self.current().phase, GemFiatQuotePhase::Failed { .. })
    }

    pub fn on_fetch_started(&self, request: GemFiatQuoteRequest) -> GemFiatSession {
        self.with_operation(self.operation(request.quote_type).on_fetch_started(&request))
    }

    pub fn on_quote_results(&self, results: GemFiatQuotesResult) -> GemFiatSession {
        self.with_operation(self.operation(results.request.quote_type).on_quote_results(results))
    }

    pub fn on_provider_selected(&self, provider: FiatProviderName) -> GemFiatSession {
        self.with_operation(self.current().on_provider_selected(provider))
    }

    fn quote_rows(&self, asset_price: Option<f64>) -> Vec<GemFiatQuoteRow> {
        self.current().quotes.iter().map(|quote| rules::quote_row(quote, asset_price)).collect()
    }

    fn selected_quote_row(&self, asset_price: Option<f64>) -> Option<GemFiatQuoteRow> {
        self.selected_quote().map(|quote| rules::quote_row(&quote, asset_price))
    }

    fn selected_quote(&self) -> Option<FiatQuote> {
        self.current().selected_quote()
    }

    fn can_select_provider(&self) -> bool {
        self.current().quotes.len() > 1
    }

    fn amount_check(&self) -> GemFiatAmountCheck {
        let operation = self.current();
        match operation.parsed_amount() {
            Some(amount) => rules::amount_check(&get_fiat_config(), operation.quote_type, amount, operation.selected_quote().as_ref(), &self.available, super::quote::CURRENCY),
            None => GemFiatAmountCheck::Valid,
        }
    }

    fn button_action(&self) -> GemFiatButtonAction {
        match self.current().phase {
            GemFiatQuotePhase::Failed { .. } => GemFiatButtonAction::RetryQuote,
            _ => GemFiatButtonAction::Continue,
        }
    }

    fn button_state(&self, is_url_loading: bool) -> GemButtonState {
        if is_url_loading {
            return GemButtonState::Loading;
        }
        match self.current().phase {
            GemFiatQuotePhase::Loading { .. } => GemButtonState::Loading,
            GemFiatQuotePhase::Failed { .. } => GemButtonState::Enabled,
            GemFiatQuotePhase::Ready if self.selected_quote().is_some() && self.amount_check() == GemFiatAmountCheck::Valid => GemButtonState::Enabled,
            _ => GemButtonState::Disabled,
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
            rules::FiatAmountInput::Value(value) => match rules::amount_check(&get_fiat_config(), quote_type, value, None, &Default::default(), super::quote::CURRENCY) {
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
            selected_provider: self.selected_provider,
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

    fn on_provider_selected(&self, provider: FiatProviderName) -> Self {
        if !self.quotes.iter().any(|quote| quote.provider.id == provider) {
            return self.clone();
        }
        Self {
            selected_provider: Some(provider),
            ..self.clone()
        }
    }

    fn selected_quote(&self) -> Option<FiatQuote> {
        rules::selected_quote(&self.quotes, self.selected_provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::{Asset, Chain, FiatProviderName};

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
                check: GemFiatAmountCheck::BelowMinimum {
                    minimum: crate::formatted_number::GemFormattedNumber::currency(5.0, primitives::Currency::USD, crate::precision::GemCurrencyStyle::Currency)
                }
            }
        );
        assert_eq!(session.on_amount_changed("4".to_string()).quote_request(), None);
        assert_eq!(session.on_amount_changed("12,5".to_string()).current().phase, GemFiatQuotePhase::InvalidInput);
        assert_eq!(session.on_amount_changed("12".to_string()).current().phase, GemFiatQuotePhase::Loading { amount: 12.0 });
        assert_eq!(session.on_amount_changed("4".to_string()).button_state(false), GemButtonState::Disabled);
    }

    #[test]
    fn test_changing_the_amount_clears_quotes_and_keeps_the_same_amount_untouched() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, None).on_quote_results(GemFiatQuotesResult::mock(vec![FiatQuote {
            crypto_amount: 1.0,
            ..FiatQuote::mock(FiatProviderName::Transak)
        }]));

        assert_eq!(session.on_amount_changed("50".to_string()), session);
        let changed = session.on_amount_changed("75".to_string());
        assert!(changed.current().quotes.is_empty());
        assert_eq!(changed.current().phase, GemFiatQuotePhase::Loading { amount: 75.0 });
        assert_eq!(changed.selected_quote(), None);
        assert_eq!(changed.button_state(false), GemButtonState::Loading);
    }

    #[test]
    fn test_quote_results_only_apply_to_the_amount_still_loading() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, None);
        let stale = session.on_quote_results(GemFiatQuotesResult {
            request: GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Buy,
                amount: 40.0,
            },
            ..GemFiatQuotesResult::mock(vec![FiatQuote {
                crypto_amount: 1.0,
                ..FiatQuote::mock(FiatProviderName::Transak)
            }])
        });
        assert_eq!(stale, session);

        let ready = session.on_quote_results(GemFiatQuotesResult::mock(vec![FiatQuote {
            crypto_amount: 1.0,
            ..FiatQuote::mock(FiatProviderName::Transak)
        }]));
        assert_eq!(ready.current().phase, GemFiatQuotePhase::Ready);
        assert_eq!(ready.selected_quote().map(|quote| quote.provider.id), Some(FiatProviderName::Transak));
        assert_eq!(ready.button_state(false), GemButtonState::Enabled);
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
        assert_eq!(ready.on_quote_results(GemFiatQuotesResult::mock(vec![])), ready);
    }

    #[test]
    fn test_empty_quotes_and_failures_are_distinct_and_only_a_failure_offers_a_retry() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, None);

        let empty = session.on_quote_results(GemFiatQuotesResult::mock(vec![]));
        assert_eq!(empty.current().phase, GemFiatQuotePhase::NoQuotes);
        assert_eq!(empty.button_state(false), GemButtonState::Disabled);
        assert_eq!(empty.button_action(), GemFiatButtonAction::Continue);
        assert!(empty.quote_request().is_some());

        let failed = session.on_quote_results(GemFiatQuotesResult {
            error: Some(GemServiceError::Api { msg: "offline".to_string() }),
            ..GemFiatQuotesResult::mock(vec![FiatQuote {
                crypto_amount: 1.0,
                ..FiatQuote::mock(FiatProviderName::Transak)
            }])
        });
        assert_eq!(
            failed.current().phase,
            GemFiatQuotePhase::Failed {
                error: GemServiceError::Api { msg: "offline".to_string() }
            }
        );
        assert!(failed.current().quotes.is_empty());
        assert_eq!(failed.button_state(false), GemButtonState::Enabled);
        assert_eq!(failed.button_action(), GemFiatButtonAction::RetryQuote);
        assert_eq!(failed.button_state(true), GemButtonState::Loading);
    }

    #[test]
    fn test_the_chosen_provider_survives_a_refresh_and_an_unknown_one_is_ignored() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, None).on_quote_results(GemFiatQuotesResult::mock(vec![
            FiatQuote {
                crypto_amount: 1.0,
                ..FiatQuote::mock(FiatProviderName::Transak)
            },
            FiatQuote {
                crypto_amount: 2.0,
                ..FiatQuote::mock(FiatProviderName::MoonPay)
            },
        ]));
        assert!(session.can_select_provider());

        let selected = session.on_provider_selected(FiatProviderName::MoonPay);
        assert_eq!(selected.selected_quote().map(|quote| quote.provider.id), Some(FiatProviderName::MoonPay));
        assert_eq!(session.on_provider_selected(FiatProviderName::Banxa), session);

        let refreshed = selected
            .on_fetch_started(GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Buy,
                amount: 50.0,
            })
            .on_quote_results(GemFiatQuotesResult::mock(vec![
                FiatQuote {
                    crypto_amount: 3.0,
                    ..FiatQuote::mock(FiatProviderName::MoonPay)
                },
                FiatQuote {
                    crypto_amount: 1.0,
                    ..FiatQuote::mock(FiatProviderName::Transak)
                },
            ]));
        assert_eq!(refreshed.selected_quote().map(|quote| quote.crypto_amount), Some(3.0));

        let gone = selected
            .on_fetch_started(GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Buy,
                amount: 50.0,
            })
            .on_quote_results(GemFiatQuotesResult::mock(vec![FiatQuote {
                crypto_amount: 1.0,
                ..FiatQuote::mock(FiatProviderName::Transak)
            }]));
        assert_eq!(gone.selected_quote().map(|quote| quote.provider.id), Some(FiatProviderName::Transak));
        assert!(!gone.can_select_provider());
    }

    #[test]
    fn test_a_sell_quote_above_the_balance_disables_the_button_until_the_balance_covers_it() {
        let sell = FiatQuote {
            asset: Asset::from_chain(Chain::Ethereum),
            quote_type: FiatQuoteType::Sell,
            crypto_amount: 1.0,
            ..FiatQuote::mock(FiatProviderName::Transak)
        };
        let session = GemFiatSession::new(FiatQuoteType::Sell, None).on_quote_results(GemFiatQuotesResult {
            request: GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Sell,
                amount: 100.0,
            },
            ..GemFiatQuotesResult::mock(vec![sell])
        });

        assert_eq!(session.current().phase, GemFiatQuotePhase::Ready);
        assert!(matches!(session.amount_check(), GemFiatAmountCheck::InsufficientBalance { .. }));
        assert_eq!(session.button_state(false), GemButtonState::Disabled);

        let funded = session.on_balance_changed(BigUint::from(1_000_000_000_000_000_000u64));
        assert_eq!(funded.amount_check(), GemFiatAmountCheck::Valid);
        assert_eq!(funded.button_state(false), GemButtonState::Enabled);
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

    #[test]
    fn test_view_state_carries_the_rows_selection_and_button_at_once() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, Some(100))
            .on_fetch_started(GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Buy,
                amount: 100.0,
            })
            .on_quote_results(GemFiatQuotesResult {
                request: GemFiatQuoteRequest {
                    quote_type: FiatQuoteType::Buy,
                    amount: 100.0,
                },
                ..GemFiatQuotesResult::mock(vec![
                    FiatQuote {
                        crypto_amount: 2.0,
                        ..FiatQuote::mock(FiatProviderName::Banxa)
                    },
                    FiatQuote {
                        crypto_amount: 1.0,
                        ..FiatQuote::mock(FiatProviderName::MoonPay)
                    },
                ])
            });

        let state = session.view_state(Some(50.0), false, true);
        assert_eq!(state.quote_type, FiatQuoteType::Buy);
        assert_eq!(state.amount, "100");
        assert_eq!(state.phase, GemFiatQuotePhase::Ready);
        assert_eq!(state.quote_rows.len(), 2);
        assert_eq!(
            state.rate_row,
            state.selected_quote_row.as_ref().and_then(|row| row.rate.clone()).map(|rate| GemListRow::Rate { title: GemListRowTitle::Rate, rate }),
            "the selected quote's rate is drawn as the shared row"
        );
        assert!(state.rate_row.is_some());
        assert_eq!(state.selected_quote_row.map(|row| row.provider), session.selected_quote().map(|quote| quote.provider.id));
        assert!(state.can_select_provider);
        assert_eq!(state.button_action, GemFiatButtonAction::Continue);
        assert_eq!(state.button_state, GemButtonState::Enabled);
        assert_eq!(session.view_state(None, true, false).button_state, GemButtonState::Loading);
        assert!(!session.view_state(None, false, false).shows_type_picker);
        assert!(session.view_state(None, false, true).shows_type_picker);
    }

    #[test]
    fn test_the_quotes_message_follows_the_phase() {
        let state = |phase| GemFiatViewState {
            quote_type: FiatQuoteType::Buy,
            amount: String::new(),
            phase,
            quote_rows: Vec::new(),
            selected_quote_row: None,
            rate_row: None,
            can_select_provider: false,
            amount_check: GemFiatAmountCheck::Valid,
            button_action: GemFiatButtonAction::Continue,
            button_state: GemButtonState::Disabled,
            shows_type_picker: false,
        };

        assert_eq!(state(GemFiatQuotePhase::NoInput).quotes_message(), Some(GemFiatQuotesMessage::EnterAmount));
        assert_eq!(state(GemFiatQuotePhase::InvalidInput).quotes_message(), Some(GemFiatQuotesMessage::EnterAmount));
        assert_eq!(state(GemFiatQuotePhase::NoQuotes).quotes_message(), Some(GemFiatQuotesMessage::NoResults));
        assert_eq!(state(GemFiatQuotePhase::Ready).quotes_message(), None);
        assert_eq!(
            state(GemFiatQuotePhase::Failed { error: GemServiceError::Offline }).quotes_message(),
            Some(GemFiatQuotesMessage::Failed { error: GemErrorText::NetworkOffline }),
            "a failed quote names its error on both apps instead of a generic one"
        );
    }

    #[test]
    fn test_a_failed_quote_stops_the_clock_and_an_off_screen_session_never_polls() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, Some(50));
        let request = session.quote_request().unwrap();

        assert!(session.refreshes_quotes(true));
        assert!(!session.refreshes_quotes(false), "a backgrounded screen asks for nothing");

        let failed = session.on_fetch_started(request.clone()).on_quote_results(GemFiatQuotesResult {
            request: request.clone(),
            quotes: vec![],
            error: Some(GemServiceError::Offline),
        });
        assert!(!failed.refreshes_quotes(true), "the retry button owns the next request");

        assert!(failed.on_amount_changed("75".to_string()).refreshes_quotes(true), "a new amount starts the clock again");
    }

    #[test]
    fn test_the_other_side_of_the_session_keeps_its_own_clock() {
        let session = GemFiatSession::new(FiatQuoteType::Buy, Some(50));
        let request = session.quote_request().unwrap();
        let failed = session.on_fetch_started(request.clone()).on_quote_results(GemFiatQuotesResult {
            request,
            quotes: vec![],
            error: Some(GemServiceError::Offline),
        });

        assert!(!failed.refreshes_quotes(true));
        assert!(failed.on_type_changed(FiatQuoteType::Sell).refreshes_quotes(true), "the sell side never failed");
    }
}
