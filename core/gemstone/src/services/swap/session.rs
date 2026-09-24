use crate::models::button::GemButtonState;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, AssetId, Currency};
use swapper::{Quote as SwapperQuote, SwapperError, SwapperProvider};

use super::model::{GemSwapButtonAction, GemSwapButtonInput};
use super::rules;
use crate::formatted_number::GemFormattedNumber;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::list::GemInfoTopic;
use crate::precision::{GemCurrencyStyle, GemValueStyle};
use crate::services::amount::model::GemNumberFormat;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapRequest {
    pub pay_asset_id: AssetId,
    pub receive_asset_id: AssetId,
    pub value: GemBigUint,
    pub slippage_bps: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapQuotesResult {
    pub request: GemSwapRequest,
    pub quotes: Vec<SwapperQuote>,
    pub error: Option<SwapperError>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapQuoteInput {
    pub request: GemSwapRequest,
    pub use_max_amount: bool,
}

#[derive(Debug, Clone, PartialEq, Default, uniffi::Enum)]
pub enum GemSwapQuotePhase {
    #[default]
    NoInput,
    Loading {
        request: GemSwapRequest,
    },
    Ready,
    Failed {
        request: GemSwapRequest,
        error: SwapperError,
    },
}

impl GemSwapQuotePhase {
    fn is_failed(&self) -> bool {
        match self {
            Self::Failed { .. } => true,
            Self::NoInput | Self::Loading { .. } | Self::Ready => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, uniffi::Enum)]
pub enum GemSwapTransferPhase {
    #[default]
    Idle,
    Loading {
        request: GemSwapRequest,
        provider: SwapperProvider,
    },
    Failed {
        request: GemSwapRequest,
        provider: SwapperProvider,
        error: SwapperError,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapSessionAction {
    None,
    QuoteLoading,
    Ready,
    TransferLoading,
    QuoteError { error: SwapperError },
    TransferError { error: SwapperError },
}

#[uniffi::export]
pub fn swap_error_display(error: SwapperError, pay_asset: Option<Asset>) -> GemSwapErrorDisplay {
    GemSwapErrorDisplay::new(&error, pay_asset.as_ref())
}

impl GemSwapSession {
    fn quotes_state(&self, pay_asset: Option<&Asset>) -> GemSwapQuotesState {
        if self.is_quote_loading() {
            return GemSwapQuotesState::Loading;
        }
        if let Some(error) = self.quote_error_display(pay_asset) {
            return GemSwapQuotesState::Failed { error };
        }
        match &self.quotes {
            Some(_) => GemSwapQuotesState::Quotes,
            None => GemSwapQuotesState::Empty,
        }
    }

    fn quote_error_display(&self, pay_asset: Option<&Asset>) -> Option<GemSwapErrorDisplay> {
        self.quote_error().as_ref().map(|error| GemSwapErrorDisplay::new(error, pay_asset))
    }

    fn error_display(&self, pay_asset: Option<&Asset>) -> Option<GemSwapErrorDisplay> {
        self.error().as_ref().map(|error| GemSwapErrorDisplay::new(error, pay_asset))
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapErrorDisplay {
    NotSupportedAsset,
    NoQuote,
    Offline,
    MinimumAmount { minimum: GemFormattedNumber },
    AmountTooSmall,
}

#[uniffi::export]
impl GemSwapErrorDisplay {
    pub fn info(&self) -> Option<GemInfoTopic> {
        match self {
            Self::NoQuote => Some(GemInfoTopic::NoQuote),
            Self::NotSupportedAsset | Self::Offline | Self::MinimumAmount { .. } | Self::AmountTooSmall => None,
        }
    }
}

impl GemSwapErrorDisplay {
    fn new(error: &SwapperError, pay_asset: Option<&Asset>) -> Self {
        match error {
            SwapperError::NotSupportedChain | SwapperError::NotSupportedAsset => Self::NotSupportedAsset,
            SwapperError::NoQuoteAvailable | SwapperError::NoAvailableProvider | SwapperError::InvalidRoute | SwapperError::ComputeQuoteError(_) | SwapperError::TransactionError(_) => Self::NoQuote,
            SwapperError::Offline => Self::Offline,
            SwapperError::InputAmountError { .. } => match (pay_asset, rules::minimum_amount(Some(error))) {
                (Some(asset), Some(min_amount)) => Self::MinimumAmount {
                    minimum: GemFormattedNumber::asset_amount(&min_amount, asset, GemValueStyle::Auto),
                },
                _ => Self::AmountTooSmall,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapViewState {
    pub quotes_state: GemSwapQuotesState,
    pub action: GemSwapSessionAction,
    pub button_action: GemSwapButtonAction,
    pub button_state: GemButtonState,
    pub quote: Option<SwapperQuote>,
    pub quote_error: Option<GemSwapErrorDisplay>,
    pub error: Option<GemSwapErrorDisplay>,
    pub is_quote_loading: bool,
    pub is_transfer_loading: bool,
    pub allows_provider_selection: bool,
    pub is_input_empty: bool,
    pub pay: GemSwapSideInteraction,
    pub receive: GemSwapSideInteraction,
    pub is_receive_loading: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemSwapSideInteraction {
    pub is_amount_editable: bool,
    pub is_asset_selectable: bool,
    pub is_balance_action_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapQuotesState {
    Loading,
    Failed { error: GemSwapErrorDisplay },
    Quotes,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Default, uniffi::Record)]
pub struct GemSwapSession {
    #[uniffi(default = None)]
    pub quotes: Option<GemSwapQuotesResult>,
    #[uniffi(default = None)]
    pub selected_provider: Option<SwapperProvider>,
    #[uniffi(default = None)]
    pub selected_quote: Option<SwapperQuote>,
    pub quote_phase: GemSwapQuotePhase,
    pub transfer_phase: GemSwapTransferPhase,
    #[uniffi(default = false)]
    pub refresh_paused_until_restart: bool,
    #[uniffi(default = None)]
    pub input: Option<GemSwapQuoteInput>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapProviderRow {
    pub provider: SwapperProvider,
    pub title: String,
    pub amount: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
    pub is_selected: bool,
}

#[uniffi::export]
pub fn swap_provider_row(provider: SwapperProvider, title: String, to_value: GemBigUint, receive_asset: Asset, receive_price: Option<f64>, currency: Currency, is_selected: bool) -> GemSwapProviderRow {
    let value = BigNumberFormatter::f64_value(to_value.to_string(), receive_asset.decimals as u32);
    GemSwapProviderRow {
        provider,
        title,
        amount: GemFormattedNumber::amount(value, Some(receive_asset.symbol), GemValueStyle::Auto),
        fiat: receive_price.map(|price| GemFormattedNumber::currency(value * price, currency, GemCurrencyStyle::Currency)),
        is_selected,
    }
}

#[uniffi::export]
impl GemSwapSession {
    pub fn provider_rows(&self, receive_asset: Asset, receive_price: Option<f64>, currency: Currency) -> Vec<GemSwapProviderRow> {
        let selected = self.current_quote().map(|quote| quote.data.provider.id);
        self.quotes
            .as_ref()
            .map(|quotes| quotes.quotes.clone())
            .unwrap_or_default()
            .iter()
            .map(|quote| {
                swap_provider_row(
                    quote.data.provider.id,
                    quote.data.provider.protocol.clone(),
                    quote.to_value.clone(),
                    receive_asset.clone(),
                    receive_price,
                    currency.clone(),
                    Some(quote.data.provider.id) == selected,
                )
            })
            .collect()
    }

    pub fn on_input_changed(&self, amount: String, pay_asset: Option<Asset>, receive_asset: Option<Asset>, available_value: GemBigInt, slippage_bps: Option<u32>, format: GemNumberFormat) -> GemSwapSession {
        let input = match (pay_asset, receive_asset) {
            (Some(pay), Some(receive)) => rules::quote_input(&pay, &receive, &amount, &available_value, slippage_bps, &format),
            _ => None,
        };
        let request = input.as_ref().map(|input| input.request.clone());
        let session = if request.as_ref() == self.current_request() { self.clone() } else { self.on_request_changed(request) };
        GemSwapSession { input, ..session }
    }

    pub fn on_refresh_requested(&self, request: GemSwapRequest) -> GemSwapSession {
        GemSwapSession {
            quote_phase: GemSwapQuotePhase::Loading { request },
            ..self.on_quote_invalidated()
        }
    }

    pub fn on_fetch_started(&self, request: GemSwapRequest) -> GemSwapSession {
        if !self.accepts_quote_phase() {
            return self.clone();
        }
        GemSwapSession {
            quote_phase: GemSwapQuotePhase::Loading { request },
            ..self.clone()
        }
    }

    pub fn on_quote_results(&self, results: GemSwapQuotesResult) -> GemSwapSession {
        if self.current_request() != Some(&results.request) {
            return self.clone();
        }
        let error = results.error.clone().or_else(|| results.quotes.is_empty().then_some(SwapperError::NoQuoteAvailable));
        let quotes = if self.accepts_quotes() { error.is_none().then_some(results.clone()) } else { self.quotes.clone() };
        let quote_phase = if self.accepts_quote_phase() {
            match error {
                Some(error) => GemSwapQuotePhase::Failed { request: results.request, error },
                None => GemSwapQuotePhase::Ready,
            }
        } else {
            self.quote_phase.clone()
        };
        GemSwapSession {
            selected_quote: quotes.as_ref().and_then(|quotes| rules::selected_quote(&quotes.quotes, self.selected_provider)),
            quotes,
            quote_phase,
            ..self.clone()
        }
    }

    pub fn on_provider_selected(&self, provider: SwapperProvider) -> GemSwapSession {
        GemSwapSession {
            selected_provider: Some(provider),
            selected_quote: self.quotes.as_ref().and_then(|quotes| rules::selected_quote(&quotes.quotes, Some(provider))),
            ..self.on_quote_invalidated()
        }
    }

    pub fn on_refresh_resumed(&self) -> GemSwapSession {
        GemSwapSession {
            refresh_paused_until_restart: false,
            ..self.clone()
        }
    }

    pub fn start_transfer(&self) -> Option<GemSwapSession> {
        if matches!(self.transfer_phase, GemSwapTransferPhase::Loading { .. }) {
            return None;
        }
        let request = self.quotes.as_ref()?.request.clone();
        let quote = self.quote()?;
        Some(GemSwapSession {
            transfer_phase: GemSwapTransferPhase::Loading { request, provider: quote.data.provider.id },
            ..self.clone()
        })
    }

    pub fn on_transfer_failed(&self, transfer: GemSwapTransferPhase, error: SwapperError) -> GemSwapSession {
        let GemSwapTransferPhase::Loading { request, provider } = &transfer else {
            return self.clone();
        };
        if self.transfer_phase != transfer {
            return self.clone();
        }
        GemSwapSession {
            transfer_phase: GemSwapTransferPhase::Failed {
                request: request.clone(),
                provider: *provider,
                error,
            },
            ..self.clone()
        }
    }

    pub fn on_transfer_handed_off(&self, transfer: GemSwapTransferPhase) -> GemSwapSession {
        if !matches!(transfer, GemSwapTransferPhase::Loading { .. }) || self.transfer_phase != transfer {
            return self.clone();
        }
        GemSwapSession {
            transfer_phase: GemSwapTransferPhase::Idle,
            refresh_paused_until_restart: true,
            ..self.clone()
        }
    }

    pub fn view_state(&self, available_balance: GemBigInt, pay_asset: Option<Asset>) -> GemSwapViewState {
        let button_action = self.button_action(available_balance);
        GemSwapViewState {
            quotes_state: self.quotes_state(pay_asset.as_ref()),
            action: self.action(),
            button_state: self.button_state(button_action.clone()),
            button_action,
            quote: self.quote(),
            quote_error: self.quote_error_display(pay_asset.as_ref()),
            error: self.error_display(pay_asset.as_ref()),
            is_quote_loading: self.is_quote_loading(),
            is_transfer_loading: self.is_transfer_loading(),
            allows_provider_selection: self.allows_provider_selection(),
            is_input_empty: self.is_input_empty(),
            pay: GemSwapSideInteraction {
                is_amount_editable: !self.is_transfer_loading(),
                is_asset_selectable: !self.is_transfer_loading(),
                is_balance_action_enabled: !self.is_transfer_loading(),
            },
            receive: GemSwapSideInteraction {
                is_amount_editable: false,
                is_asset_selectable: !self.is_transfer_loading(),
                is_balance_action_enabled: false,
            },
            is_receive_loading: self.is_quote_loading() && !self.is_transfer_loading(),
        }
    }

    pub fn quote(&self) -> Option<SwapperQuote> {
        self.current_quote().cloned()
    }

    fn quote_error(&self) -> Option<SwapperError> {
        match &self.quote_phase {
            GemSwapQuotePhase::Failed { error, .. } => Some(error.clone()),
            GemSwapQuotePhase::NoInput | GemSwapQuotePhase::Loading { .. } | GemSwapQuotePhase::Ready => None,
        }
    }

    fn error(&self) -> Option<SwapperError> {
        self.transfer_error().or_else(|| self.quote_error())
    }

    fn allows_provider_selection(&self) -> bool {
        self.quotes.as_ref().is_some_and(|quotes| quotes.quotes.len() > 1) && !self.is_transfer_loading()
    }

    fn is_quote_loading(&self) -> bool {
        matches!(self.quote_phase, GemSwapQuotePhase::Loading { .. })
    }

    pub fn is_transfer_loading(&self) -> bool {
        matches!(self.transfer_phase, GemSwapTransferPhase::Loading { .. })
    }

    fn is_input_empty(&self) -> bool {
        matches!(self.quote_phase, GemSwapQuotePhase::NoInput)
    }

    pub fn refreshes_quotes(&self, is_screen_active: bool) -> bool {
        is_screen_active && !self.refresh_paused_until_restart && !self.is_transfer_loading() && !self.quote_phase.is_failed()
    }

    fn action(&self) -> GemSwapSessionAction {
        match (&self.transfer_phase, &self.quote_phase) {
            (GemSwapTransferPhase::Loading { .. }, _) => GemSwapSessionAction::TransferLoading,
            (GemSwapTransferPhase::Failed { error, .. }, _) => GemSwapSessionAction::TransferError { error: error.clone() },
            (GemSwapTransferPhase::Idle, GemSwapQuotePhase::Loading { .. }) => GemSwapSessionAction::QuoteLoading,
            (GemSwapTransferPhase::Idle, GemSwapQuotePhase::Failed { error, .. }) => GemSwapSessionAction::QuoteError { error: error.clone() },
            (GemSwapTransferPhase::Idle, GemSwapQuotePhase::Ready | GemSwapQuotePhase::NoInput) => {
                if self.current_quote().is_some() {
                    GemSwapSessionAction::Ready
                } else {
                    GemSwapSessionAction::None
                }
            }
        }
    }

    fn button_action(&self, available_balance: GemBigInt) -> GemSwapButtonAction {
        GemSwapButtonInput {
            value: self.input.as_ref().map(|input| GemBigInt::from(input.request.value.clone())).unwrap_or_default(),
            available_balance,
            quote_error: self.quote_error(),
            transfer_error: self.transfer_error(),
        }
        .action()
    }

    fn button_state(&self, action: GemSwapButtonAction) -> GemButtonState {
        match action {
            GemSwapButtonAction::InsufficientBalance => GemButtonState::Disabled,
            _ if self.is_quote_loading() || self.is_transfer_loading() => GemButtonState::Loading,
            GemSwapButtonAction::Swap if self.current_quote().is_none() => GemButtonState::Disabled,
            GemSwapButtonAction::Swap | GemSwapButtonAction::RetryQuote | GemSwapButtonAction::RetryTransfer | GemSwapButtonAction::UseMinimumAmount { .. } => GemButtonState::Enabled,
        }
    }
}

impl GemSwapSession {
    pub(crate) fn on_request_changed(&self, request: Option<GemSwapRequest>) -> GemSwapSession {
        let Some(request) = request else {
            return GemSwapSession::default();
        };
        let same_pair = self
            .current_request()
            .is_some_and(|current| current.pay_asset_id == request.pay_asset_id && current.receive_asset_id == request.receive_asset_id);
        GemSwapSession {
            quotes: None,
            selected_provider: self.selected_provider.filter(|_| same_pair),
            ..self.on_refresh_requested(request)
        }
    }

    fn on_quote_invalidated(&self) -> GemSwapSession {
        GemSwapSession {
            transfer_phase: GemSwapTransferPhase::Idle,
            refresh_paused_until_restart: false,
            ..self.clone()
        }
    }

    fn current_quote(&self) -> Option<&SwapperQuote> {
        self.quotes.as_ref()?;
        self.selected_quote.as_ref()
    }

    pub(crate) fn transfer_error(&self) -> Option<SwapperError> {
        match &self.transfer_phase {
            GemSwapTransferPhase::Failed { error, .. } => Some(error.clone()),
            GemSwapTransferPhase::Idle | GemSwapTransferPhase::Loading { .. } => None,
        }
    }

    fn accepts_quotes(&self) -> bool {
        matches!(self.transfer_phase, GemSwapTransferPhase::Idle)
    }

    fn accepts_quote_phase(&self) -> bool {
        !self.is_transfer_loading() && !self.refresh_paused_until_restart
    }

    fn current_request(&self) -> Option<&GemSwapRequest> {
        match &self.quote_phase {
            GemSwapQuotePhase::Loading { request } | GemSwapQuotePhase::Failed { request, .. } => Some(request),
            GemSwapQuotePhase::Ready | GemSwapQuotePhase::NoInput => self.quotes.as_ref().map(|quotes| &quotes.request),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::{Asset, Chain};

    #[test]
    fn test_the_error_display_collapses_every_no_quote_cause_into_one() {
        for error in [
            SwapperError::NoQuoteAvailable,
            SwapperError::NoAvailableProvider,
            SwapperError::InvalidRoute,
            SwapperError::ComputeQuoteError("status 500".to_string()),
            SwapperError::TransactionError("bad transaction".to_string()),
        ] {
            assert_eq!(GemSwapErrorDisplay::new(&error, None), GemSwapErrorDisplay::NoQuote);
        }
        assert_eq!(GemSwapErrorDisplay::new(&SwapperError::NotSupportedChain, None), GemSwapErrorDisplay::NotSupportedAsset);
        assert_eq!(GemSwapErrorDisplay::new(&SwapperError::NotSupportedAsset, None), GemSwapErrorDisplay::NotSupportedAsset);
        assert_eq!(GemSwapErrorDisplay::new(&SwapperError::Offline, None), GemSwapErrorDisplay::Offline);
    }

    #[test]
    fn test_only_a_missing_quote_offers_an_info_sheet() {
        assert_eq!(GemSwapErrorDisplay::NoQuote.info(), Some(GemInfoTopic::NoQuote));
        assert_eq!(GemSwapErrorDisplay::NotSupportedAsset.info(), None);
        assert_eq!(GemSwapErrorDisplay::Offline.info(), None);
        assert_eq!(GemSwapErrorDisplay::AmountTooSmall.info(), None);
        assert_eq!(
            GemSwapErrorDisplay::MinimumAmount {
                minimum: GemFormattedNumber::asset_amount(&GemBigInt::from(1), &Asset::from_chain(Chain::Ethereum), GemValueStyle::Auto),
            }
            .info(),
            None
        );
    }

    #[test]
    fn test_a_minimum_only_reaches_the_app_with_an_asset_and_a_positive_amount() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let display = |min_amount: Option<&str>, pay_asset: Option<&Asset>| GemSwapErrorDisplay::new(&SwapperError::InputAmountError { min_amount: min_amount.map(str::to_string) }, pay_asset);
        assert_eq!(
            display(Some("123456"), Some(&asset)),
            GemSwapErrorDisplay::MinimumAmount {
                minimum: GemFormattedNumber::asset_amount(&GemBigInt::from(123456), &asset, GemValueStyle::Auto),
            }
        );
        assert_eq!(display(Some("123456"), None), GemSwapErrorDisplay::AmountTooSmall);
        assert_eq!(display(Some("0"), Some(&asset)), GemSwapErrorDisplay::AmountTooSmall);
        assert_eq!(display(Some("not a number"), Some(&asset)), GemSwapErrorDisplay::AmountTooSmall);
        assert_eq!(display(None, Some(&asset)), GemSwapErrorDisplay::AmountTooSmall);
    }

    #[test]
    fn test_a_changed_request_loads_and_an_empty_one_resets() {
        let loading = GemSwapSession::default().on_request_changed(Some(GemSwapRequest::mock()));
        assert_eq!(loading.quote_phase, GemSwapQuotePhase::Loading { request: GemSwapRequest::mock() });
        assert!(loading.quotes.is_none());
        assert_eq!(loading.action(), GemSwapSessionAction::QuoteLoading);

        let reset = GemSwapSession::mock_ready().on_request_changed(None);
        assert_eq!(reset, GemSwapSession::default());
        assert!(reset.is_input_empty());
        assert_eq!(reset.action(), GemSwapSessionAction::None);
    }

    #[test]
    fn test_quote_results_pick_the_best_quote_and_report_failures() {
        let session = GemSwapSession::mock_ready();
        assert_eq!(session.quote().unwrap().data.provider.id, SwapperProvider::Okx);
        assert_eq!(session.action(), GemSwapSessionAction::Ready);

        let failed = session.on_quote_results(GemSwapQuotesResult {
            error: Some(SwapperError::NoAvailableProvider),
            ..GemSwapQuotesResult::mock(vec![])
        });
        assert!(failed.quotes.is_none());
        assert!(failed.quote().is_none());
        assert_eq!(failed.quote_error(), Some(SwapperError::NoAvailableProvider));
        assert_eq!(failed.action(), GemSwapSessionAction::QuoteError { error: SwapperError::NoAvailableProvider });

        let empty = session.on_quote_results(GemSwapQuotesResult::mock(vec![]));
        assert_eq!(empty.quote_error(), Some(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_every_quote_becomes_a_provider_row_and_the_chosen_one_is_marked() {
        let session = GemSwapSession::mock_ready().on_quote_results(GemSwapQuotesResult::mock(vec![
            SwapperQuote::mock_with_provider(SwapperProvider::Okx, "11"),
            SwapperQuote::mock_with_provider(SwapperProvider::Jupiter, "9"),
        ]));
        let asset = Asset::mock_ethereum_usdc();

        let rows = session.provider_rows(asset.clone(), Some(2.0), Currency::USD);

        assert_eq!(rows.len(), 2, "no provider is dropped from the list");
        assert!(rows[0].is_selected);
        assert!(!rows[1].is_selected);
        assert_eq!(rows[0].amount.unit, crate::formatted_number::GemNumberUnit::Symbol { symbol: asset.symbol });
        assert_eq!(rows[0].fiat.as_ref().map(|fiat| fiat.value), Some(rows[0].amount.value * 2.0));
        assert_eq!(session.provider_rows(Asset::mock_ethereum_usdc(), None, Currency::USD)[0].fiat, None);
    }

    #[test]
    fn test_results_for_an_outdated_request_are_ignored() {
        let outdated = GemSwapRequest {
            value: BigUint::from(200u32),
            ..GemSwapRequest::mock()
        };
        let loading = GemSwapSession::default().on_request_changed(Some(GemSwapRequest::mock()));

        let late_success = loading.on_quote_results(GemSwapQuotesResult {
            request: outdated.clone(),
            ..GemSwapQuotesResult::mock(vec![SwapperQuote::mock_with_provider(SwapperProvider::Okx, "11")])
        });
        assert_eq!(late_success, loading);

        let late_failure = loading.on_quote_results(GemSwapQuotesResult {
            request: outdated,
            quotes: vec![],
            error: Some(SwapperError::Offline),
        });
        assert_eq!(late_failure, loading);
    }

    #[test]
    fn test_a_failed_quote_stops_the_automatic_refresh_until_a_retry_or_new_input() {
        let request = GemSwapRequest::mock();
        let failed = GemSwapSession::default().on_request_changed(Some(request.clone())).on_quote_results(GemSwapQuotesResult {
            request: request.clone(),
            quotes: vec![],
            error: Some(SwapperError::ComputeQuoteError("offline".into())),
        });

        assert!(!failed.refreshes_quotes(true));
        assert!(failed.on_refresh_requested(request).refreshes_quotes(true));
        assert!(failed.on_request_changed(None).refreshes_quotes(true));
    }

    #[test]
    fn test_the_chosen_provider_survives_a_refresh_and_falls_back_when_it_disappears() {
        let chosen = GemSwapSession::mock_ready().on_provider_selected(SwapperProvider::Jupiter);
        assert_eq!(chosen.quote().unwrap().data.provider.id, SwapperProvider::Jupiter);

        let refreshed = chosen.on_quote_results(GemSwapQuotesResult::mock(vec![
            SwapperQuote::mock_with_provider(SwapperProvider::Okx, "11"),
            SwapperQuote::mock_with_provider(SwapperProvider::Jupiter, "8"),
        ]));
        assert_eq!(refreshed.quote().unwrap().to_value, BigUint::from(8u32));

        let without = refreshed.on_quote_results(GemSwapQuotesResult::mock(vec![SwapperQuote::mock_with_provider(SwapperProvider::Okx, "12")]));
        assert_eq!(without.quote().unwrap().data.provider.id, SwapperProvider::Okx);

        let new_amount = chosen.on_request_changed(Some(GemSwapRequest {
            value: BigUint::from(200u32),
            ..GemSwapRequest::mock()
        }));
        assert!(new_amount.quote().is_none());
        assert_eq!(new_amount.selected_provider, Some(SwapperProvider::Jupiter));

        let new_pair = chosen.on_request_changed(Some(GemSwapRequest {
            receive_asset_id: AssetId::from_chain(Chain::Bitcoin),
            value: BigUint::from(200u32),
            ..GemSwapRequest::mock()
        }));
        assert_eq!(new_pair.selected_provider, None);
    }

    #[test]
    fn test_a_loading_transfer_keeps_its_quote_and_ignores_new_results() {
        let started = GemSwapSession::mock_ready().start_transfer().unwrap();
        assert!(started.is_transfer_loading());
        assert_eq!(started.action(), GemSwapSessionAction::TransferLoading);
        assert!(started.start_transfer().is_none());
        assert!(!started.accepts_quotes());

        let refreshed = started.on_quote_results(GemSwapQuotesResult::mock(vec![SwapperQuote::mock_with_provider(SwapperProvider::Okx, "99")]));
        assert_eq!(refreshed.quote().unwrap().to_value, BigUint::from(10u32));
        assert!(refreshed.is_transfer_loading());
        assert_eq!(refreshed.on_fetch_started(GemSwapRequest::mock()), refreshed);

        assert!(GemSwapSession::default().start_transfer().is_none());
    }

    #[test]
    fn test_a_loading_transfer_locks_both_sides_and_the_receive_side_is_never_typed_into() {
        let ready = GemSwapSession::mock_ready().view_state(GemBigInt::from(1000), None);
        assert_eq!(
            (ready.pay, ready.receive),
            (
                GemSwapSideInteraction {
                    is_amount_editable: true,
                    is_asset_selectable: true,
                    is_balance_action_enabled: true
                },
                GemSwapSideInteraction {
                    is_amount_editable: false,
                    is_asset_selectable: true,
                    is_balance_action_enabled: false
                },
            )
        );

        let transfer = GemSwapSession::mock_ready().start_transfer().unwrap().view_state(GemBigInt::from(1000), None);
        assert!(!transfer.pay.is_amount_editable && !transfer.pay.is_asset_selectable && !transfer.receive.is_asset_selectable);
        assert!(!transfer.is_receive_loading, "a transfer in flight is not a quote loading");
    }

    #[test]
    fn test_transfer_outcomes_apply_only_to_the_transfer_that_started() {
        let started = GemSwapSession::mock_ready().start_transfer().unwrap();
        let transfer = started.transfer_phase.clone();

        let failed = started.on_transfer_failed(transfer.clone(), SwapperError::TransactionError("boom".into()));
        assert_eq!(failed.transfer_error(), Some(SwapperError::TransactionError("boom".into())));
        assert_eq!(failed.error(), failed.transfer_error(), "a failed transfer outranks the quote it came from");
        assert_eq!(
            failed.action(),
            GemSwapSessionAction::TransferError {
                error: SwapperError::TransactionError("boom".into())
            }
        );
        assert!(failed.quote().is_some());
        assert_eq!(failed.on_transfer_failed(transfer.clone(), SwapperError::NoQuoteAvailable), failed);

        let handed_off = started.on_transfer_handed_off(transfer);
        assert_eq!(handed_off.transfer_phase, GemSwapTransferPhase::Idle);
        assert!(handed_off.refresh_paused_until_restart);
        assert!(!handed_off.refreshes_quotes(true));
        assert!(handed_off.on_refresh_resumed().refreshes_quotes(true));
        assert!(!handed_off.on_refresh_resumed().refreshes_quotes(false));
        assert_eq!(handed_off.on_fetch_started(GemSwapRequest::mock()), handed_off);

        assert_eq!(GemSwapSession::mock_ready().on_transfer_handed_off(GemSwapTransferPhase::Idle), GemSwapSession::mock_ready());
    }

    #[test]
    fn test_quote_changes_clear_a_failed_transfer() {
        let started = GemSwapSession::mock_ready().start_transfer().unwrap();
        let failed = started.on_transfer_failed(started.transfer_phase.clone(), SwapperError::NoQuoteAvailable);

        assert_eq!(failed.on_provider_selected(SwapperProvider::Jupiter).transfer_phase, GemSwapTransferPhase::Idle);
        assert_eq!(failed.on_refresh_requested(GemSwapRequest::mock()).transfer_phase, GemSwapTransferPhase::Idle);
        assert_eq!(failed.on_quote_invalidated().transfer_phase, GemSwapTransferPhase::Idle);
    }

    #[test]
    fn test_button_state_follows_the_phases() {
        let idle = GemSwapSession::default();
        assert_eq!(idle.button_state(GemSwapButtonAction::Swap), GemButtonState::Disabled);

        let loading = idle.on_request_changed(Some(GemSwapRequest::mock()));
        assert_eq!(loading.button_state(GemSwapButtonAction::Swap), GemButtonState::Loading);

        let session = GemSwapSession::mock_ready();
        assert_eq!(session.button_action(GemBigInt::from(200)), GemSwapButtonAction::Swap);
        assert_eq!(session.button_state(GemSwapButtonAction::Swap), GemButtonState::Enabled);
        assert_eq!(session.button_action(GemBigInt::from(50)), GemSwapButtonAction::InsufficientBalance);
        assert_eq!(session.button_state(GemSwapButtonAction::InsufficientBalance), GemButtonState::Disabled);

        let started = session.start_transfer().unwrap();
        assert_eq!(started.button_state(GemSwapButtonAction::Swap), GemButtonState::Loading);
        let failed = started.on_transfer_failed(started.transfer_phase.clone(), SwapperError::TransactionError("boom".into()));
        assert_eq!(failed.button_action(GemBigInt::from(200)), GemSwapButtonAction::RetryTransfer);
        assert_eq!(failed.button_state(GemSwapButtonAction::RetryTransfer), GemButtonState::Enabled);
    }

    #[test]
    fn test_input_changes_drive_the_request_and_dedupe() {
        let pay = Asset::from_chain(Chain::Ethereum);
        let receive = Asset::from_chain(Chain::Bitcoin);
        let format = GemNumberFormat { decimal_separator: ".".to_string() };
        let available = GemBigInt::from(2_000_000_000_000_000_000u128);
        let changed = |session: &GemSwapSession, amount: &str| session.on_input_changed(amount.to_string(), Some(pay.clone()), Some(receive.clone()), available.clone(), None, format.clone());

        assert!(GemSwapSession::default().input.is_none(), "no amount, no input");

        let typed = changed(&GemSwapSession::default(), "1");
        assert!(matches!(typed.quote_phase, GemSwapQuotePhase::Loading { .. }), "a valid input starts loading");
        let input = typed.input.clone().unwrap();
        assert_eq!(input.request.value, GemBigUint::from(1_000_000_000_000_000_000u128));
        assert!(!input.use_max_amount);

        assert_eq!(changed(&typed, "1"), typed, "re-deriving the same request changes nothing");
        assert!(changed(&typed, "2").input.unwrap().use_max_amount);

        let cleared = changed(&typed, "");
        assert!(cleared.is_input_empty());
        assert!(cleared.input.is_none());

        let same_asset = typed.on_input_changed("1".to_string(), Some(pay.clone()), Some(pay.clone()), available.clone(), None, format.clone());
        assert!(same_asset.input.is_none(), "an asset does not swap into itself");
    }

    #[test]
    fn test_a_selected_quote_without_results_is_not_a_quote() {
        let stale = GemSwapSession {
            quotes: None,
            ..GemSwapSession::mock_ready()
        };

        assert!(stale.selected_quote.is_some(), "the selection outlives the results it came from");
        assert!(stale.quote().is_none());
        assert_eq!(stale.action(), GemSwapSessionAction::None);
        assert_eq!(stale.button_state(GemSwapButtonAction::Swap), GemButtonState::Disabled);
    }

    #[test]
    fn test_view_state_carries_the_quote_and_button_at_once() {
        let idle = GemSwapSession::default().view_state(GemBigInt::from(0), None);
        assert!(idle.is_input_empty);
        assert_eq!(idle.button_state, GemButtonState::Disabled);

        let session = GemSwapSession::mock_ready();
        let state = session.view_state(GemBigInt::from(200), None);
        assert_eq!(state.quote, session.quote());
        assert_eq!(state.action, GemSwapSessionAction::Ready);
        assert_eq!(state.button_action, GemSwapButtonAction::Swap);
        assert_eq!(state.button_state, GemButtonState::Enabled);
        assert!(!state.is_quote_loading);
        assert_eq!(state.quotes_state, GemSwapQuotesState::Quotes);
        assert_eq!(session.view_state(GemBigInt::from(50), None).button_action, GemSwapButtonAction::InsufficientBalance);
    }

    #[test]
    fn test_quotes_state_puts_loading_before_an_error_and_an_error_before_quotes() {
        assert_eq!(GemSwapSession::default().view_state(GemBigInt::from(0), None).quotes_state, GemSwapQuotesState::Empty);

        let loading = GemSwapSession::mock_ready().on_fetch_started(GemSwapRequest::mock());
        assert_eq!(loading.view_state(GemBigInt::from(0), None).quotes_state, GemSwapQuotesState::Loading);

        let failed = GemSwapSession {
            quote_phase: GemSwapQuotePhase::Failed {
                request: GemSwapRequest::mock(),
                error: SwapperError::NoQuoteAvailable,
            },
            ..GemSwapSession::mock_ready()
        };
        assert!(matches!(failed.view_state(GemBigInt::from(0), None).quotes_state, GemSwapQuotesState::Failed { .. }));
    }
}
