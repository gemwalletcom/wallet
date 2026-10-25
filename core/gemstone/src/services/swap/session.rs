use crate::models::button::GemButtonState;
use crate::services::assets::icon::asset_icon;
use number_formatter::BigNumberFormatter;
use number_formatter::precision::round_to_places;
use primitives::{Asset, AssetData, AssetId, Currency};
use swapper::config::quote_preferences;
use swapper::{Quote as SwapperQuote, QuoteRequest as SwapperQuoteRequest, SwapperError, SwapperProvider, rank_quotes};

use super::model::{GemSwapButtonAction, GemSwapButtonInput, GemSwapDetails, quote_details};
use super::rules;
use crate::config::swap_config::get_swap_config;
use crate::formatted_number::GemFormattedNumber;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::list::{GemInfoTopic, GemProviderKind, GemProviderLoadingRow, GemProviderRow};
use crate::models::swap::GemSwapValue;
use crate::precision::{GemCurrencyStyle, GemValueStyle};
use crate::services::amount::model::GemNumberFormat;
use crate::services::assets::rules::fiat_amount_of;
use crate::services::balance::GemAssetBalance;
use crate::services::balance::rules::available_balance_text;
use crate::services::localization::GemLocalizedText;

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
pub struct GemSwapQuoteProviders {
    pub request: SwapperQuoteRequest,
    pub providers: Vec<SwapperProvider>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapProviderAnswer {
    Quote { provider: SwapperProvider, quote: SwapperQuote, leaves_no_dust: bool },
    Failed { provider: SwapperProvider, error: SwapperError },
}

impl GemSwapProviderAnswer {
    fn provider(&self) -> SwapperProvider {
        match self {
            Self::Quote { provider, .. } | Self::Failed { provider, .. } => *provider,
        }
    }

    pub(super) fn quote(&self) -> Option<&SwapperQuote> {
        match self {
            Self::Quote { quote, .. } => Some(quote),
            Self::Failed { .. } => None,
        }
    }

    fn error(&self) -> Option<&SwapperError> {
        match self {
            Self::Quote { .. } => None,
            Self::Failed { error, .. } => Some(error),
        }
    }

    fn leaves_no_dust(&self) -> bool {
        match self {
            Self::Quote { leaves_no_dust, .. } => *leaves_no_dust,
            Self::Failed { .. } => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapQuoteAnswers {
    pub request: GemSwapRequest,
    pub providers: Vec<SwapperProvider>,
    pub answers: Vec<GemSwapProviderAnswer>,
    pub first_quote_wait_ended: bool,
}

impl GemSwapQuoteAnswers {
    fn new(request: GemSwapRequest, providers: Vec<SwapperProvider>) -> Self {
        Self {
            request,
            providers,
            answers: vec![],
            first_quote_wait_ended: false,
        }
    }

    fn with_answer(&self, answer: GemSwapProviderAnswer) -> Option<Self> {
        self.pending().contains(&answer.provider()).then(|| Self {
            answers: [self.answers.clone(), vec![answer]].concat(),
            ..self.clone()
        })
    }

    fn is_complete(&self) -> bool {
        self.answers.len() == self.providers.len()
    }

    fn has_quote(&self) -> bool {
        self.answers.iter().any(|answer| answer.quote().is_some())
    }

    fn shows_first_quote(&self) -> bool {
        let round = get_swap_config().quote_round;
        let answered = self.answers.len();
        let enough_answered = answered >= round.answered_minimum as usize && self.providers.len() - answered <= round.pending_allowed as usize;
        self.has_quote() && (self.is_complete() || enough_answered || self.first_quote_wait_ended)
    }

    fn pending(&self) -> Vec<SwapperProvider> {
        self.providers.iter().filter(|provider| self.answers.iter().all(|answer| answer.provider() != **provider)).copied().collect()
    }

    fn is_pending(&self, provider: Option<SwapperProvider>) -> bool {
        provider.is_some_and(|provider| self.pending().contains(&provider))
    }

    fn first_quote_wait(&self) -> u64 {
        get_swap_config().quote_round.timing(self.request.pay_asset_id.chain, self.request.receive_asset_id.chain).first_quote_wait_ms
    }

    fn results(&self) -> GemSwapQuotesResult {
        let quotes: Vec<SwapperQuote> = self.answers.iter().filter_map(GemSwapProviderAnswer::quote).cloned().collect();
        let dust_free: Vec<SwapperProvider> = self.answers.iter().filter(|answer| answer.leaves_no_dust()).map(GemSwapProviderAnswer::provider).collect();
        let quotes = match quotes.first().map(|quote| quote.request.clone()) {
            Some(request) => rank_quotes(&request, quotes, &quote_preferences(), |quote| dust_free.contains(&quote.data.provider.id)),
            None => quotes,
        };
        GemSwapQuotesResult {
            request: self.request.clone(),
            error: quotes.is_empty().then(|| SwapperError::combined(self.answers.iter().filter_map(GemSwapProviderAnswer::error).cloned().collect())),
            quotes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapFirstQuoteWait {
    pub request: GemSwapRequest,
    pub milliseconds: u64,
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
    fn is_transfer_loading(&self) -> bool {
        matches!(self.transfer_phase, GemSwapTransferPhase::Loading { .. })
    }

    fn quote_error(&self) -> Option<SwapperError> {
        match &self.quote_phase {
            GemSwapQuotePhase::Failed { error, .. } => Some(error.clone()),
            GemSwapQuotePhase::NoInput | GemSwapQuotePhase::Loading { .. } | GemSwapQuotePhase::Ready => None,
        }
    }

    fn allows_provider_selection(&self) -> bool {
        let pending = self.open_answers().map(|answers| answers.pending().len()).unwrap_or_default();
        self.quotes.as_ref().is_some_and(|quotes| quotes.quotes.len() + pending > 1) && !self.is_transfer_loading()
    }

    fn open_answers(&self) -> Option<&GemSwapQuoteAnswers> {
        self.answers.as_ref().filter(|answers| !answers.is_complete() && self.accepts_quotes())
    }

    fn first_quote_wait(&self) -> Option<GemSwapFirstQuoteWait> {
        let answers = self.answers.as_ref().filter(|answers| answers.has_quote() && !answers.shows_first_quote())?;
        self.is_quote_loading().then(|| GemSwapFirstQuoteWait {
            request: answers.request.clone(),
            milliseconds: answers.first_quote_wait(),
        })
    }

    fn is_quote_loading(&self) -> bool {
        matches!(self.quote_phase, GemSwapQuotePhase::Loading { .. })
    }

    fn is_input_empty(&self) -> bool {
        matches!(self.quote_phase, GemSwapQuotePhase::NoInput)
    }

    fn button_action(&self, available_balance: GemBigInt) -> GemSwapButtonAction {
        GemSwapButtonInput {
            value: self.input.as_ref().map(|input| GemBigInt::from(input.request.value.clone())).unwrap_or_default(),
            available_balance,
            quote_error: self.quote_error(),
        }
        .action()
    }

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
pub struct GemSwapSideState {
    pub interaction: GemSwapSideInteraction,
    pub title: GemLocalizedText,
    pub amount_placeholder: String,
    pub icon: Option<crate::services::assets::icon::GemAssetIcon>,
    pub balance: Option<GemLocalizedText>,
    pub fiat: Option<GemFormattedNumber>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapViewState {
    pub quotes_state: GemSwapQuotesState,
    pub action: GemSwapSessionAction,
    pub button_action: GemSwapButtonAction,
    pub button_state: GemButtonState,
    pub quote: Option<SwapperQuote>,
    pub is_quote_loading: bool,
    pub is_transfer_loading: bool,
    pub allows_provider_selection: bool,
    pub is_input_empty: bool,
    pub shows_button: bool,
    pub pay: GemSwapSideState,
    pub receive: GemSwapSideState,
    pub is_receive_loading: bool,
    pub receive_amount: Option<GemFormattedNumber>,
    pub first_quote_wait: Option<GemSwapFirstQuoteWait>,
    pub providers: Vec<GemProviderRow>,
    pub pending_providers: Vec<GemProviderLoadingRow>,
    pub details: Option<GemSwapDetails>,
    pub shows_details: bool,
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
    #[uniffi(default = None)]
    pub pay_value: Option<GemBigUint>,
    #[uniffi(default = None)]
    pub answers: Option<GemSwapQuoteAnswers>,
}

fn side_title(data: Option<&AssetData>) -> GemLocalizedText {
    match data {
        Some(data) => GemLocalizedText::Text { text: data.asset.symbol.clone() },
        None => GemLocalizedText::SelectAsset,
    }
}

fn side_amount_placeholder(data: Option<&AssetData>) -> String {
    data.map(|_| "0".to_string()).unwrap_or_default()
}

pub(super) fn provider_row(provider: SwapperProvider, name: String, to_value: &GemBigUint, receive_asset: &Asset, receive_price: Option<f64>, currency: &Currency, is_selected: bool) -> GemProviderRow {
    let value = BigNumberFormatter::f64_value(to_value, receive_asset.decimals);
    GemProviderRow {
        kind: GemProviderKind::Swap { provider },
        name,
        amount: GemFormattedNumber::amount(value, Some(receive_asset.symbol.clone()), GemValueStyle::Auto),
        fiat: receive_price.map(|price| GemFormattedNumber::currency(value * price, currency.clone(), GemCurrencyStyle::Currency)),
        is_selected,
    }
}

#[uniffi::export]
impl GemSwapSession {
    pub fn minimum_amount_text(&self, pay_asset: Asset, format: GemNumberFormat) -> Option<String> {
        let minimum = rules::minimum_amount(self.quote_error().as_ref())?;
        format.input_text(minimum.to_string(), pay_asset.decimals)
    }

    pub fn on_input_changed(&self, amount: String, pay_asset: Option<Asset>, receive_asset: Option<Asset>, available_value: GemBigInt, slippage_bps: Option<u32>, format: GemNumberFormat) -> GemSwapSession {
        let input = match (&pay_asset, &receive_asset) {
            (Some(pay), Some(receive)) => rules::quote_input(pay, receive, &amount, &available_value, slippage_bps, &format),
            _ => None,
        };
        let request = input.as_ref().map(|input| input.request.clone());
        let session = if request.as_ref() == self.current_request() { self.clone() } else { self.on_request_changed(request) };
        GemSwapSession {
            pay_value: pay_asset.as_ref().and_then(|pay| rules::pay_value(pay, &amount, &format)),
            input,
            ..session
        }
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
            answers: None,
            ..self.clone()
        }
    }

    pub fn on_quotes_requested(&self, request: GemSwapRequest, providers: Vec<SwapperProvider>) -> GemSwapSession {
        if !self.accepts_quote_phase() || self.current_request() != Some(&request) {
            return self.clone();
        }
        GemSwapSession {
            quote_phase: GemSwapQuotePhase::Loading { request: request.clone() },
            answers: Some(GemSwapQuoteAnswers::new(request, providers)),
            ..self.clone()
        }
    }

    pub fn on_provider_answer(&self, request: GemSwapRequest, answer: GemSwapProviderAnswer) -> GemSwapSession {
        match self.answers.as_ref().filter(|answers| answers.request == request).and_then(|answers| answers.with_answer(answer)) {
            Some(answers) => self.on_answers(answers),
            None => self.clone(),
        }
    }

    pub fn on_first_quote_wait_ended(&self, request: GemSwapRequest) -> GemSwapSession {
        match self.answers.as_ref().filter(|answers| answers.request == request) {
            Some(answers) => self.on_answers(GemSwapQuoteAnswers {
                first_quote_wait_ended: true,
                ..answers.clone()
            }),
            None => self.clone(),
        }
    }

    pub fn on_quotes_failed(&self, request: GemSwapRequest, error: SwapperError) -> GemSwapSession {
        self.on_quote_results(GemSwapQuotesResult { request, quotes: vec![], error: Some(error) })
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

    pub fn start_transfer(&self, quote: SwapperQuote) -> Option<GemSwapSession> {
        if self.is_transfer_loading() || self.current_quote() != Some(&quote) {
            return None;
        }
        let request = self.quotes.as_ref()?.request.clone();
        Some(GemSwapSession {
            transfer_phase: GemSwapTransferPhase::Loading { request, provider: quote.data.provider.id },
            quote_phase: GemSwapQuotePhase::Ready,
            answers: None,
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

    pub fn quote(&self) -> Option<SwapperQuote> {
        self.current_quote().cloned()
    }

    pub fn refreshes_quotes(&self, is_screen_active: bool) -> bool {
        is_screen_active && !self.refresh_paused_until_restart && !self.is_transfer_loading() && !self.quote_phase.is_failed()
    }

    pub fn view_state(&self, pay: Option<AssetData>, receive: Option<AssetData>, currency: Currency) -> GemSwapViewState {
        match self.held_answers(pay.as_ref(), receive.as_ref()) {
            Some(answers) => GemSwapViewState {
                quotes_state: self.quotes_state(pay.as_ref().map(|pay| &pay.asset)),
                details: None,
                ..GemSwapSession {
                    quote_phase: GemSwapQuotePhase::Loading { request: answers.request.clone() },
                    ..self.clone()
                }
                .shown_view_state(pay, receive, currency)
            },
            None => self.shown_view_state(pay, receive, currency),
        }
    }
}

fn receive_amount(quote: &SwapperQuote) -> GemFormattedNumber {
    GemFormattedNumber::amount(BigNumberFormatter::f64_value(&quote.to_value, quote.request.to_asset.decimals), None, GemValueStyle::Auto)
}

impl GemSwapSession {
    fn shown_view_state(&self, pay: Option<AssetData>, receive: Option<AssetData>, currency: Currency) -> GemSwapViewState {
        let available_balance = pay.as_ref().map(|pay| GemBigInt::from(pay.balance.available.clone())).unwrap_or_default();
        let button_action = self.button_action(available_balance);
        let is_transfer_loading = self.is_transfer_loading();
        let quote = self.pair_quote(pay.as_ref(), receive.as_ref());
        let is_receive_loading = self.is_quote_loading() && !is_transfer_loading;
        let receive_quote = quote.filter(|_| !is_receive_loading);
        let details = pay
            .as_ref()
            .zip(receive.as_ref())
            .zip(quote)
            .map(|((pay, receive), quote)| quote_details(rules::swap_quote(quote), pay.asset.clone(), receive.asset.clone(), price_value(pay), price_value(receive), &currency));
        let shows_details = details.is_some() && !self.is_quote_loading();
        GemSwapViewState {
            quotes_state: self.quotes_state(pay.as_ref().map(|pay| &pay.asset)),
            action: self.action(),
            button_state: self.button_state(button_action.clone()),
            button_action,
            is_quote_loading: self.is_quote_loading(),
            is_transfer_loading,
            allows_provider_selection: self.allows_provider_selection(),
            is_input_empty: self.is_input_empty(),
            shows_button: !self.is_input_empty(),
            pay: GemSwapSideState {
                title: side_title(pay.as_ref()),
                amount_placeholder: side_amount_placeholder(pay.as_ref()),
                icon: pay.as_ref().map(|pay| asset_icon(&pay.asset.id)),
                interaction: GemSwapSideInteraction {
                    is_amount_editable: !is_transfer_loading,
                    is_asset_selectable: !is_transfer_loading,
                    is_balance_action_enabled: !is_transfer_loading && pay.is_some(),
                },
                balance: pay.as_ref().map(|pay| available_balance_text(pay.asset.clone(), GemAssetBalance::from(pay))),
                fiat: pay
                    .as_ref()
                    .zip(self.pay_value.as_ref())
                    .and_then(|(pay, value)| fiat_amount_of(&pay.asset, value, price_value(pay), currency.clone(), GemCurrencyStyle::Currency)),
            },
            receive: GemSwapSideState {
                title: side_title(receive.as_ref()),
                amount_placeholder: side_amount_placeholder(receive.as_ref().filter(|_| !is_receive_loading)),
                icon: receive.as_ref().map(|receive| asset_icon(&receive.asset.id)),
                interaction: GemSwapSideInteraction {
                    is_amount_editable: false,
                    is_asset_selectable: !is_transfer_loading,
                    is_balance_action_enabled: false,
                },
                balance: receive.as_ref().map(|receive| available_balance_text(receive.asset.clone(), GemAssetBalance::from(receive))),
                fiat: receive
                    .as_ref()
                    .zip(receive_quote)
                    .and_then(|(receive, quote)| fiat_amount_of(&receive.asset, &quote.to_value, price_value(receive), currency.clone(), GemCurrencyStyle::Currency)),
            },
            is_receive_loading,
            receive_amount: receive_quote.map(receive_amount),
            first_quote_wait: self.first_quote_wait(),
            providers: receive
                .as_ref()
                .filter(|_| quote.is_some())
                .map(|receive| self.provider_rows(&receive.asset, price_value(receive), &currency))
                .unwrap_or_default(),
            pending_providers: quote.map(|_| self.pending_provider_rows()).unwrap_or_default(),
            details,
            shows_details,
            quote: quote.cloned(),
        }
    }

    fn held_answers(&self, pay: Option<&AssetData>, receive: Option<&AssetData>) -> Option<&GemSwapQuoteAnswers> {
        let answers = self.open_answers()?;
        let (pay, receive) = (pay?, receive?);
        let quote = self.pair_quote(Some(pay), Some(receive)).filter(|quote| !self.is_quote_loading() && self.selected_provider != Some(quote.data.provider.id))?;
        let paid = GemSwapValue::new(quote.from_value.clone(), pay.asset.decimals, price_value(pay));
        let impact = paid.price_impact(&GemSwapValue::new(quote.to_value.clone(), receive.asset.decimals, price_value(receive)))?;
        let hold_percent = f64::from(get_swap_config().quote_round.hold_price_impact_percent);
        (round_to_places(impact.percentage, 2) < -hold_percent).then_some(answers)
    }

    fn pending_provider_rows(&self) -> Vec<GemProviderLoadingRow> {
        self.open_answers()
            .map(GemSwapQuoteAnswers::pending)
            .unwrap_or_default()
            .into_iter()
            .map(|provider| GemProviderLoadingRow {
                kind: GemProviderKind::Swap { provider },
                name: provider.name().to_string(),
            })
            .collect()
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

    fn button_state(&self, action: GemSwapButtonAction) -> GemButtonState {
        match action {
            GemSwapButtonAction::InsufficientBalance => GemButtonState::Disabled,
            _ if self.is_quote_loading() || self.is_transfer_loading() => GemButtonState::Loading,
            GemSwapButtonAction::Swap if self.current_quote().is_none() => GemButtonState::Disabled,
            GemSwapButtonAction::Swap | GemSwapButtonAction::RetryQuote | GemSwapButtonAction::UseMinimumAmount { .. } => GemButtonState::Enabled,
        }
    }

    fn provider_rows(&self, receive_asset: &Asset, receive_price: Option<f64>, currency: &Currency) -> Vec<GemProviderRow> {
        let selected = self.current_quote().map(|quote| quote.data.provider.id);
        self.quotes
            .iter()
            .flat_map(|quotes| &quotes.quotes)
            .map(|quote| {
                provider_row(
                    quote.data.provider.id,
                    quote.data.provider.protocol.clone(),
                    &quote.to_value,
                    receive_asset,
                    receive_price,
                    currency,
                    Some(quote.data.provider.id) == selected,
                )
            })
            .collect()
    }

    fn pair_quote(&self, pay: Option<&AssetData>, receive: Option<&AssetData>) -> Option<&SwapperQuote> {
        let request = &self.quotes.as_ref()?.request;
        let is_current_pair = pay?.asset.id == request.pay_asset_id && receive?.asset.id == request.receive_asset_id;
        self.current_quote().filter(|_| is_current_pair)
    }

    pub(crate) fn on_request_changed(&self, request: Option<GemSwapRequest>) -> GemSwapSession {
        let Some(request) = request else {
            return GemSwapSession::default();
        };
        let same_pair = self
            .current_request()
            .is_some_and(|current| current.pay_asset_id == request.pay_asset_id && current.receive_asset_id == request.receive_asset_id);
        GemSwapSession {
            quotes: None,
            answers: None,
            selected_provider: self.selected_provider.filter(|_| same_pair),
            ..self.on_refresh_requested(request)
        }
    }

    pub(super) fn on_quote_results(&self, results: GemSwapQuotesResult) -> GemSwapSession {
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

    fn on_answers(&self, answers: GemSwapQuoteAnswers) -> GemSwapSession {
        let session = GemSwapSession {
            answers: Some(answers.clone()),
            ..self.clone()
        };
        if session.is_quote_loading() && !answers.is_complete() && (!answers.shows_first_quote() || answers.is_pending(self.selected_provider)) {
            return session;
        }
        session.on_quote_results(answers.results())
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

fn price_value(data: &AssetData) -> Option<f64> {
    data.price.as_ref().map(|price| price.price)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::swap::testkit::mock_asset_data;
    use chrono::Utc;
    use num_bigint::BigUint;
    use primitives::{Asset, Chain, Price, PriceProvider};

    fn view(session: &GemSwapSession, available: u64) -> GemSwapViewState {
        session.view_state(Some(mock_asset_data(Chain::Ethereum, available)), Some(mock_asset_data(Chain::Solana, 0)), Currency::USD)
    }

    #[test]
    fn test_the_minimum_amount_is_typed_in_the_pay_assets_decimals_and_the_callers_separator() {
        let request = GemSwapRequest {
            pay_asset_id: AssetId::from_chain(Chain::Ethereum),
            receive_asset_id: AssetId::from_chain(Chain::Solana),
            value: GemBigUint::from(1u32),
            slippage_bps: None,
        };
        let failed = |min_amount: Option<&str>| GemSwapSession {
            quote_phase: GemSwapQuotePhase::Failed {
                request: request.clone(),
                error: SwapperError::InputAmountError { min_amount: min_amount.map(str::to_string) },
            },
            ..GemSwapSession::default()
        };
        let asset = Asset {
            decimals: 6,
            ..Asset::from_chain(Chain::Ethereum)
        };
        let format = GemNumberFormat { decimal_separator: ",".to_string() };

        assert_eq!(failed(Some("1500000")).minimum_amount_text(asset.clone(), format.clone()), Some("1,5".to_string()));
        assert_eq!(failed(None).minimum_amount_text(asset.clone(), format.clone()), None);
        assert_eq!(GemSwapSession::default().minimum_amount_text(asset, format), None);
    }

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

        let rows = session.provider_rows(&asset, Some(2.0), &Currency::USD);

        assert_eq!(rows.len(), 2, "no provider is dropped from the list");
        assert!(rows[0].is_selected);
        assert!(!rows[1].is_selected);
        assert_eq!(rows[0].amount.unit, crate::formatted_number::GemNumberUnit::Symbol { symbol: asset.symbol.clone() });
        assert_eq!(rows[0].fiat.as_ref().map(|fiat| fiat.value), Some(rows[0].amount.value * 2.0));
        assert_eq!(session.provider_rows(&asset, None, &Currency::USD)[0].fiat, None);
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
    fn test_a_transfer_starts_only_for_the_quote_on_screen() {
        let ready = GemSwapSession::mock_ready();
        let shown = ready.quote().unwrap();

        assert!(
            ready.start_transfer(SwapperQuote::mock_with_provider(SwapperProvider::Jupiter, "99")).is_none(),
            "a quote that is no longer the one shown is not traded"
        );
        let refreshing = ready
            .on_refresh_requested(GemSwapRequest::mock())
            .start_transfer(shown.clone())
            .expect("the quote the user saw still trades while a refresh loads");
        let failed = refreshing.on_transfer_failed(refreshing.transfer_phase.clone(), SwapperError::TransactionError("boom".into()));
        assert!(!view(&failed, 1000).is_quote_loading, "the abandoned refresh does not leave the screen spinning after the transfer");
        let replaced = ready.on_quote_results(GemSwapQuotesResult::mock(vec![SwapperQuote::mock_with_provider(SwapperProvider::Okx, "11")]));
        assert!(replaced.start_transfer(shown).is_none(), "once a new quote takes its place, the old one no longer trades");
    }

    #[test]
    fn test_the_details_row_hides_while_the_receive_side_loads() {
        let ready = GemSwapSession::mock_ready();
        assert!(view(&ready, 1000).shows_details);

        let refreshing = view(&ready.on_refresh_requested(GemSwapRequest::mock()), 1000);
        assert!(!refreshing.shows_details, "the last round's rate does not show next to the spinner");
        assert!(refreshing.details.is_some(), "an open Details sheet keeps its details through a refresh");

        assert!(view(&start(&ready).unwrap(), 1000).shows_details, "a transfer in flight keeps its details");
    }

    #[test]
    fn test_a_loading_transfer_keeps_its_quote_and_ignores_new_results() {
        let started = start(&GemSwapSession::mock_ready()).unwrap();
        assert!(started.is_transfer_loading());
        assert_eq!(started.action(), GemSwapSessionAction::TransferLoading);
        assert!(start(&started).is_none());
        assert!(!started.accepts_quotes());

        let refreshed = started.on_quote_results(GemSwapQuotesResult::mock(vec![SwapperQuote::mock_with_provider(SwapperProvider::Okx, "99")]));
        assert_eq!(refreshed.quote().unwrap().to_value, BigUint::from(10u32));
        assert!(refreshed.is_transfer_loading());
        assert_eq!(refreshed.on_fetch_started(GemSwapRequest::mock()), refreshed);

        assert!(start(&GemSwapSession::default()).is_none());
    }

    #[test]
    fn test_the_receive_amount_is_the_selected_quote_in_the_receive_asset_units() {
        let ready = GemSwapSession::mock_ready();
        let quote = ready.quote().unwrap();
        let amount = receive_amount(&quote);

        assert_eq!(amount.value, BigNumberFormatter::f64_value(&quote.to_value, quote.request.to_asset.decimals));
        assert_eq!(amount.unit, crate::formatted_number::GemNumberUnit::Plain, "the receive field shows the number without a symbol");
        assert!(GemSwapSession::default().view_state(None, None, Currency::USD).receive_amount.is_none());
    }

    #[test]
    fn test_a_loading_transfer_locks_both_sides_and_the_receive_side_is_never_typed_into() {
        let ready = view(&GemSwapSession::mock_ready(), 1000);
        assert_eq!(
            (ready.pay.interaction, ready.receive.interaction),
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

        let transfer = view(&start(&GemSwapSession::mock_ready()).unwrap(), 1000);
        assert!(!transfer.pay.interaction.is_amount_editable && !transfer.pay.interaction.is_asset_selectable && !transfer.receive.interaction.is_asset_selectable);
        assert!(!transfer.is_receive_loading, "a transfer in flight is not a quote loading");
    }

    #[test]
    fn test_transfer_outcomes_apply_only_to_the_transfer_that_started() {
        let started = start(&GemSwapSession::mock_ready()).unwrap();
        let transfer = started.transfer_phase.clone();

        let failed = started.on_transfer_failed(transfer.clone(), SwapperError::TransactionError("boom".into()));
        let failed_view = view(&failed, 1000);
        assert_eq!(failed_view.quotes_state, GemSwapQuotesState::Quotes, "a quote-data failure does not become an inline quote error");
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
        let started = start(&GemSwapSession::mock_ready()).unwrap();
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

        let started = start(&session).unwrap();
        assert_eq!(started.button_state(GemSwapButtonAction::Swap), GemButtonState::Loading);
        let failed = started.on_transfer_failed(started.transfer_phase.clone(), SwapperError::TransactionError("boom".into()));
        assert_eq!(failed.button_action(GemBigInt::from(200)), GemSwapButtonAction::Swap, "a failed transfer keeps the Swap button, so pressing it swaps again");
        assert_eq!(failed.button_state(GemSwapButtonAction::Swap), GemButtonState::Enabled);
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
    fn test_a_side_without_an_asset_asks_for_one_and_shows_no_zero() {
        let state = GemSwapSession::default().view_state(Some(mock_asset_data(Chain::Ethereum, 0)), None, Currency::USD);

        assert_eq!((state.pay.title, state.pay.amount_placeholder.as_str()), (GemLocalizedText::Text { text: "ETH".to_string() }, "0"));
        assert_eq!((state.receive.title, state.receive.amount_placeholder.as_str()), (GemLocalizedText::SelectAsset, ""));
    }

    #[test]
    fn test_view_state_carries_the_quote_and_button_at_once() {
        let idle = view(&GemSwapSession::default(), 0);
        assert!(idle.is_input_empty);
        assert!(!idle.shows_button, "an empty amount hides the button");
        assert_eq!(idle.button_state, GemButtonState::Disabled);

        let session = GemSwapSession::mock_ready();
        let state = view(&session, 200);
        assert!(state.shows_button);
        assert_eq!(state.quote, session.quote());
        assert_eq!(state.action, GemSwapSessionAction::Ready);
        assert_eq!(state.button_action, GemSwapButtonAction::Swap);
        assert_eq!(state.button_state, GemButtonState::Enabled);
        assert!(!state.is_quote_loading);
        assert_eq!(state.quotes_state, GemSwapQuotesState::Quotes);
        assert_eq!(view(&session, 50).button_action, GemSwapButtonAction::InsufficientBalance);
    }

    #[test]
    fn test_quotes_state_puts_loading_before_an_error_and_an_error_before_quotes() {
        assert_eq!(view(&GemSwapSession::default(), 0).quotes_state, GemSwapQuotesState::Empty);

        let loading = GemSwapSession::mock_ready().on_fetch_started(GemSwapRequest::mock());
        assert_eq!(view(&loading, 0).quotes_state, GemSwapQuotesState::Loading);

        let failed = GemSwapSession {
            quote_phase: GemSwapQuotePhase::Failed {
                request: GemSwapRequest::mock(),
                error: SwapperError::NoQuoteAvailable,
            },
            ..GemSwapSession::mock_ready()
        };
        assert!(matches!(view(&failed, 0).quotes_state, GemSwapQuotesState::Failed { .. }));
    }

    #[test]
    fn test_each_side_reads_its_balance_and_value_and_the_pay_value_waits_for_no_receive_asset() {
        let pay = AssetData {
            price: Some(Price::new(2.0, 0.0, Utc::now(), PriceProvider::Coingecko)),
            ..mock_asset_data(Chain::Ethereum, 1000)
        };
        let typed = GemSwapSession::default().on_input_changed("1".to_string(), Some(pay.asset.clone()), None, GemBigInt::from(1000), None, GemNumberFormat { decimal_separator: ".".to_string() });
        let state = typed.view_state(Some(pay), None, Currency::USD);

        assert_eq!(state.pay.fiat.map(|fiat| fiat.value), Some(2.0));
        assert!(state.pay.balance.is_some());
        assert!(state.pay.interaction.is_balance_action_enabled);
        assert_eq!((state.receive.balance, state.receive.fiat), (None, None));
        assert!(!GemSwapSession::default().view_state(None, None, Currency::USD).pay.interaction.is_balance_action_enabled, "no asset has no balance to fill in");
    }

    #[test]
    fn test_a_quote_shows_only_for_the_pair_it_was_asked_for() {
        let session = GemSwapSession::mock_ready();
        let quote = session.quote().unwrap();
        let receive = AssetData {
            price: Some(Price::new(100.0, 0.0, Utc::now(), PriceProvider::Coingecko)),
            ..mock_asset_data(Chain::Solana, 0)
        };
        let state = session.view_state(Some(mock_asset_data(Chain::Ethereum, 1000)), Some(receive.clone()), Currency::USD);

        assert_eq!(state.quote.as_ref(), Some(&quote));
        assert_eq!(state.receive_amount, Some(receive_amount(&quote)));
        assert_eq!(
            state.receive.fiat,
            fiat_amount_of(&receive.asset, &quote.to_value, Some(100.0), Currency::USD, GemCurrencyStyle::Currency),
            "the receive value comes from the exact amount, not the rounded text"
        );
        assert_eq!(state.providers.len(), 2);
        let details = state.details.unwrap();
        assert_eq!(details.provider.kind, GemProviderKind::Swap { provider: quote.data.provider.id });
        assert!(!details.provider.is_selected);

        let stale = session.view_state(Some(mock_asset_data(Chain::Ethereum, 1000)), Some(mock_asset_data(Chain::Bitcoin, 0)), Currency::USD);
        assert_eq!((stale.quote, stale.receive_amount, stale.details, stale.receive.fiat), (None, None, None, None));
        assert!(stale.providers.is_empty(), "a quote for another pair is not shown while the new one loads");
    }

    fn start(session: &GemSwapSession) -> Option<GemSwapSession> {
        session.start_transfer(session.quote()?)
    }

    fn quote_answer(provider: SwapperProvider, to_value: &str) -> GemSwapProviderAnswer {
        GemSwapProviderAnswer::Quote {
            provider,
            quote: SwapperQuote::mock_with_provider(provider, to_value),
            leaves_no_dust: false,
        }
    }

    fn failed_answer(provider: SwapperProvider, error: SwapperError) -> GemSwapProviderAnswer {
        GemSwapProviderAnswer::Failed { provider, error }
    }

    fn requested(request: &GemSwapRequest, providers: &[SwapperProvider]) -> GemSwapSession {
        GemSwapSession::default().on_request_changed(Some(request.clone())).on_quotes_requested(request.clone(), providers.to_vec())
    }

    fn answered(session: GemSwapSession, answers: impl IntoIterator<Item = GemSwapProviderAnswer>) -> GemSwapSession {
        answers.into_iter().fold(session, |session, answer| session.on_provider_answer(GemSwapRequest::mock(), answer))
    }

    fn shown_provider(session: &GemSwapSession) -> Option<SwapperProvider> {
        session.quote().filter(|_| !session.is_quote_loading()).map(|quote| quote.data.provider.id)
    }

    fn two_of_three() -> GemSwapSession {
        answered(
            requested(&GemSwapRequest::mock(), &[SwapperProvider::Relay, SwapperProvider::Mayan, SwapperProvider::NearIntents]),
            [quote_answer(SwapperProvider::Relay, "10"), quote_answer(SwapperProvider::Mayan, "9")],
        )
    }

    #[test]
    fn test_the_first_quote_shows_once_every_provider_or_all_but_one_answered() {
        let shown_after = |providers: &[SwapperProvider], count: usize| {
            let session = answered(requested(&GemSwapRequest::mock(), providers), providers.iter().take(count).map(|provider| quote_answer(*provider, "10")));
            shown_provider(&session).is_some()
        };
        let four = [SwapperProvider::Relay, SwapperProvider::Mayan, SwapperProvider::Chainflip, SwapperProvider::NearIntents];
        let solana = [
            SwapperProvider::Jupiter,
            SwapperProvider::Orca,
            SwapperProvider::RaydiumClmm,
            SwapperProvider::PumpSwap,
            SwapperProvider::MeteoraDammV2,
            SwapperProvider::Okx,
        ];

        assert!(shown_after(&[SwapperProvider::Okx], 1), "a single provider shows when it answers");
        assert!(!shown_after(&[SwapperProvider::Okx, SwapperProvider::Relay], 1), "one of two waits for the other or the wait");
        assert!(shown_after(&[SwapperProvider::Okx, SwapperProvider::Relay], 2));
        assert!(!shown_after(&four, 2));
        assert!(shown_after(&four, 3));
        assert!(!shown_after(&solana, 4));
        assert!(shown_after(&solana, 5));
    }

    #[test]
    fn test_the_wait_after_the_first_quote_shows_it_and_depends_on_the_pair() {
        let providers = [SwapperProvider::Okx, SwapperProvider::Relay];
        let held = answered(requested(&GemSwapRequest::mock(), &providers), [quote_answer(SwapperProvider::Okx, "10")]);
        assert_eq!(shown_provider(&held), None);
        assert_eq!(
            view(&held, 0).first_quote_wait,
            Some(GemSwapFirstQuoteWait {
                request: GemSwapRequest::mock(),
                milliseconds: 1_000
            }),
            "a pair across networks waits a second"
        );

        let shown = held.on_first_quote_wait_ended(GemSwapRequest::mock());
        assert_eq!(shown_provider(&shown), Some(SwapperProvider::Okx));
        assert_eq!(view(&shown, 0).first_quote_wait, None);

        let on_chain = GemSwapRequest {
            receive_asset_id: Asset::mock_ethereum_usdc().id,
            ..GemSwapRequest::mock()
        };
        let on_chain_held = requested(&on_chain, &providers).on_provider_answer(on_chain.clone(), quote_answer(SwapperProvider::Okx, "10"));
        assert_eq!(view(&on_chain_held, 0).first_quote_wait.map(|wait| wait.milliseconds), Some(500), "a pair on one network waits half a second");

        let failure = answered(requested(&GemSwapRequest::mock(), &providers), [failed_answer(SwapperProvider::Okx, SwapperError::NoQuoteAvailable)]);
        assert_eq!(view(&failure, 0).first_quote_wait, None, "a failure has no quote to wait with");
    }

    #[test]
    fn test_a_late_better_quote_replaces_the_shown_one() {
        let first = two_of_three();
        let state = view(&first, 1000);
        assert_eq!(shown_provider(&first), Some(SwapperProvider::Relay));
        assert!(state.shows_details, "Details shows the rate while the round is open");
        assert_eq!(state.button_state, GemButtonState::Enabled, "Swap works while the round is open");

        let better = first.on_provider_answer(GemSwapRequest::mock(), quote_answer(SwapperProvider::NearIntents, "11"));
        assert_eq!(shown_provider(&better), Some(SwapperProvider::NearIntents));

        let left_out = first.on_provider_answer(GemSwapRequest::mock(), failed_answer(SwapperProvider::NearIntents, SwapperError::NoQuoteAvailable));
        assert_eq!(shown_provider(&left_out), Some(SwapperProvider::Relay), "a provider silent at the close is left out");
        assert!(view(&left_out, 1000).pending_providers.is_empty());
    }

    fn priced_view(session: &GemSwapSession) -> GemSwapViewState {
        let price = |price| Some(Price::new(price, 0.0, Utc::now(), PriceProvider::Coingecko));
        let pay = AssetData {
            price: price(10_000_000_000.0),
            ..mock_asset_data(Chain::Ethereum, 1000)
        };
        let receive = AssetData {
            price: price(950_000.0),
            ..mock_asset_data(Chain::Solana, 0)
        };
        session.view_state(Some(pay), Some(receive), Currency::USD)
    }

    #[test]
    fn test_a_quote_losing_more_than_the_hold_waits_and_a_better_late_quote_shows_at_once() {
        let providers = [SwapperProvider::Relay, SwapperProvider::Mayan, SwapperProvider::Chainflip, SwapperProvider::NearIntents];
        let bad = answered(requested(&GemSwapRequest::mock(), &providers), [quote_answer(SwapperProvider::Relay, "10")]).on_first_quote_wait_ended(GemSwapRequest::mock());

        let held = priced_view(&bad);
        assert!(held.is_receive_loading, "a quote losing 5% keeps the spinner while providers are still quoting");
        assert_eq!((held.receive_amount, held.details, held.button_state), (None, None, GemButtonState::Loading));
        assert!(!held.shows_details, "Details stays hidden, as while waiting");
        assert_eq!((held.quotes_state, held.providers.len()), (GemSwapQuotesState::Quotes, 1), "Providers lists the quotes so far");
        assert!(view(&bad, 1000).details.is_some(), "a pair without prices is never held");

        let picked = bad.on_provider_selected(SwapperProvider::Relay);
        assert!(priced_view(&picked).details.is_some(), "a provider the user picked is never held");

        let better = priced_view(&bad.on_provider_answer(GemSwapRequest::mock(), quote_answer(SwapperProvider::Mayan, "11")));
        assert!(!better.is_receive_loading, "a quote within the hold shows as soon as it arrives");
        assert_eq!(better.quote.map(|quote| quote.data.provider.id), Some(SwapperProvider::Mayan));
        assert!(better.shows_details && better.details.is_some());
    }

    #[test]
    fn test_a_held_quote_shows_once_the_round_ends() {
        let providers = [SwapperProvider::Relay, SwapperProvider::NearIntents];
        let bad = answered(requested(&GemSwapRequest::mock(), &providers), [quote_answer(SwapperProvider::Relay, "10")]).on_first_quote_wait_ended(GemSwapRequest::mock());
        assert!(priced_view(&bad).is_receive_loading);

        let closed = priced_view(&bad.on_provider_answer(GemSwapRequest::mock(), failed_answer(SwapperProvider::NearIntents, SwapperError::NoQuoteAvailable)));
        assert!(!closed.is_receive_loading, "every provider answered or the close came, so the best quote shows");
        assert!(closed.shows_details);
        assert!(closed.details.unwrap().summary.price_impact_row.is_some(), "Details shows the loss");
    }

    #[test]
    fn test_the_providers_list_shows_a_loading_row_for_each_provider_still_quoting() {
        let state = view(&two_of_three(), 1000);
        assert_eq!(state.providers.len(), 2);
        assert_eq!(
            state.pending_providers,
            vec![GemProviderLoadingRow {
                kind: GemProviderKind::Swap { provider: SwapperProvider::NearIntents },
                name: SwapperProvider::NearIntents.name().to_string(),
            }]
        );

        let one_answered = answered(requested(&GemSwapRequest::mock(), &[SwapperProvider::Relay, SwapperProvider::NearIntents]), [quote_answer(SwapperProvider::Relay, "10")]).on_first_quote_wait_ended(GemSwapRequest::mock());
        assert!(view(&one_answered, 1000).allows_provider_selection, "one answer and one provider still quoting open Providers");
    }

    #[test]
    fn test_the_users_choice_wins_over_a_later_answer() {
        let picked = two_of_three().on_provider_selected(SwapperProvider::Mayan);
        let late = picked.on_provider_answer(GemSwapRequest::mock(), quote_answer(SwapperProvider::NearIntents, "11"));
        assert_eq!(shown_provider(&late), Some(SwapperProvider::Mayan), "a picked provider stays picked when a better quote arrives");

        let started = start(&two_of_three()).unwrap();
        let after_swap = started.on_provider_answer(GemSwapRequest::mock(), quote_answer(SwapperProvider::NearIntents, "11"));
        assert_eq!(after_swap.quote(), started.quote(), "once Swap is tapped no later answer changes the trade");
    }

    #[test]
    fn test_a_picked_provider_still_quoting_keeps_the_round_loading() {
        let request = GemSwapRequest {
            value: BigUint::from(200u32),
            ..GemSwapRequest::mock()
        };
        let requoting = two_of_three()
            .on_provider_selected(SwapperProvider::Mayan)
            .on_request_changed(Some(request.clone()))
            .on_quotes_requested(request.clone(), vec![SwapperProvider::Relay, SwapperProvider::Mayan, SwapperProvider::NearIntents]);
        let others = [quote_answer(SwapperProvider::Relay, "10"), quote_answer(SwapperProvider::NearIntents, "11")]
            .into_iter()
            .fold(requoting, |session, answer| session.on_provider_answer(request.clone(), answer));
        assert_eq!(shown_provider(&others), None, "no other provider's quote takes the place of the one the user picked");
        assert_eq!(shown_provider(&others.on_first_quote_wait_ended(request.clone())), None);

        let mayan = others.on_provider_answer(request.clone(), quote_answer(SwapperProvider::Mayan, "9"));
        assert_eq!(shown_provider(&mayan), Some(SwapperProvider::Mayan));

        let failed = others.on_provider_answer(request, failed_answer(SwapperProvider::Mayan, SwapperError::NoQuoteAvailable));
        assert_eq!(shown_provider(&failed), Some(SwapperProvider::NearIntents), "a picked provider with no quote gives way to the best one");
    }

    #[test]
    fn test_answers_for_an_old_request_are_thrown_away() {
        let first = two_of_three();
        let outdated = GemSwapRequest {
            value: BigUint::from(200u32),
            ..GemSwapRequest::mock()
        };

        assert_eq!(first.on_provider_answer(outdated.clone(), quote_answer(SwapperProvider::NearIntents, "11")), first);
        assert_eq!(first.on_first_quote_wait_ended(outdated.clone()), first);
        let changed = first.on_request_changed(Some(outdated));
        assert_eq!(changed.on_provider_answer(GemSwapRequest::mock(), quote_answer(SwapperProvider::NearIntents, "11")), changed);
    }

    #[test]
    fn test_a_new_fetch_drops_the_answers_of_the_one_before() {
        let restarted = two_of_three().on_fetch_started(GemSwapRequest::mock());
        let late = restarted.on_provider_answer(GemSwapRequest::mock(), failed_answer(SwapperProvider::NearIntents, SwapperError::NoQuoteAvailable));
        assert_eq!(late, restarted, "an answer from the fetch before is not counted");

        let requested = late.on_quotes_requested(GemSwapRequest::mock(), vec![SwapperProvider::Relay, SwapperProvider::NearIntents]);
        let failure = requested.on_provider_answer(GemSwapRequest::mock(), failed_answer(SwapperProvider::Relay, SwapperError::NoQuoteAvailable));
        assert_eq!(failure.quote_error(), None, "one failure while others are quoting is no error");
        assert!(failure.is_quote_loading());
    }

    #[test]
    fn test_an_answer_counts_for_the_provider_that_was_asked() {
        let answer = GemSwapProviderAnswer::Quote {
            provider: SwapperProvider::Relay,
            quote: SwapperQuote::mock_with_provider(SwapperProvider::Okx, "10"),
            leaves_no_dust: false,
        };
        let session = answered(requested(&GemSwapRequest::mock(), &[SwapperProvider::Relay]), [answer]);

        assert_eq!(shown_provider(&session), Some(SwapperProvider::Okx), "the round completes even when the quote names another provider");
    }

    #[test]
    fn test_no_quote_once_every_provider_answered_is_the_error() {
        let minimum = SwapperError::InputAmountError { min_amount: Some("100".into()) };
        let failed = answered(
            requested(&GemSwapRequest::mock(), &[SwapperProvider::Okx, SwapperProvider::Relay]),
            [failed_answer(SwapperProvider::Okx, SwapperError::NoQuoteAvailable), failed_answer(SwapperProvider::Relay, minimum.clone())],
        );

        assert_eq!(failed.quote_error(), Some(minimum));
        assert!(failed.quote().is_none());
        assert!(!failed.refreshes_quotes(true));

        let silent = answered(requested(&GemSwapRequest::mock(), &[SwapperProvider::Okx]), [failed_answer(SwapperProvider::Okx, SwapperError::NoQuoteAvailable)]);
        assert_eq!(silent.quote_error(), Some(SwapperError::NoQuoteAvailable), "no answer by the close is No quote");
    }

    #[test]
    fn test_a_refresh_shows_the_loading_indicator_without_the_old_quote() {
        let ready = GemSwapSession::mock_ready();
        let request = ready.quotes.as_ref().unwrap().request.clone();
        let receive = AssetData {
            price: Some(Price::new(100.0, 0.0, Utc::now(), PriceProvider::Coingecko)),
            ..mock_asset_data(Chain::Solana, 0)
        };
        let shown = ready.view_state(Some(mock_asset_data(Chain::Ethereum, 1000)), Some(receive.clone()), Currency::USD);
        let refreshing = ready.on_fetch_started(request).view_state(Some(mock_asset_data(Chain::Ethereum, 1000)), Some(receive), Currency::USD);

        assert!(shown.receive_amount.is_some() && shown.receive.fiat.is_some());
        assert!(refreshing.is_receive_loading);
        assert_eq!((refreshing.receive_amount, refreshing.receive.fiat, refreshing.receive.amount_placeholder.as_str()), (None, None, ""));
    }
}
