use primitives::AssetId;
use swapper::{Quote as SwapperQuote, SwapperError, SwapperProvider};

use super::model::{GemSwapButtonAction, GemSwapButtonInput};
use super::rules;
use crate::models::custom_types::{GemBigInt, GemBigUint};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSwapButtonState {
    Disabled,
    Loading,
    Enabled,
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
}

#[uniffi::export]
impl GemSwapSession {
    pub fn on_request_changed(&self, request: Option<GemSwapRequest>) -> GemSwapSession {
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
        let error = results.error.clone().or_else(|| results.quotes.is_empty().then_some(SwapperError::NoQuoteAvailable));
        let quotes = if self.accepts_quotes() {
            error.is_none().then_some(results.clone())
        } else {
            self.quotes.clone()
        };
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

    pub fn on_quote_invalidated(&self) -> GemSwapSession {
        GemSwapSession {
            transfer_phase: GemSwapTransferPhase::Idle,
            refresh_paused_until_restart: false,
            ..self.clone()
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
            transfer_phase: GemSwapTransferPhase::Loading {
                request,
                provider: quote.data.provider.id,
            },
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

    pub fn on_transfer_abandoned(&self, transfer: GemSwapTransferPhase) -> GemSwapSession {
        if !matches!(transfer, GemSwapTransferPhase::Loading { .. }) || self.transfer_phase != transfer {
            return self.clone();
        }
        self.on_quote_invalidated()
    }

    pub fn quote(&self) -> Option<SwapperQuote> {
        self.quotes.as_ref()?;
        self.selected_quote.clone()
    }

    pub fn quote_error(&self) -> Option<SwapperError> {
        match &self.quote_phase {
            GemSwapQuotePhase::Failed { error, .. } => Some(error.clone()),
            GemSwapQuotePhase::NoInput | GemSwapQuotePhase::Loading { .. } | GemSwapQuotePhase::Ready => None,
        }
    }

    pub fn transfer_error(&self) -> Option<SwapperError> {
        match &self.transfer_phase {
            GemSwapTransferPhase::Failed { error, .. } => Some(error.clone()),
            GemSwapTransferPhase::Idle | GemSwapTransferPhase::Loading { .. } => None,
        }
    }

    pub fn is_quote_loading(&self) -> bool {
        matches!(self.quote_phase, GemSwapQuotePhase::Loading { .. })
    }

    pub fn is_transfer_loading(&self) -> bool {
        matches!(self.transfer_phase, GemSwapTransferPhase::Loading { .. })
    }

    pub fn is_input_empty(&self) -> bool {
        matches!(self.quote_phase, GemSwapQuotePhase::NoInput)
    }

    pub fn accepts_quotes(&self) -> bool {
        matches!(self.transfer_phase, GemSwapTransferPhase::Idle)
    }

    pub fn refreshes_quotes(&self, is_screen_active: bool) -> bool {
        is_screen_active && !self.refresh_paused_until_restart && !self.is_transfer_loading()
    }

    pub fn action(&self) -> GemSwapSessionAction {
        match (&self.transfer_phase, &self.quote_phase) {
            (GemSwapTransferPhase::Loading { .. }, _) => GemSwapSessionAction::TransferLoading,
            (GemSwapTransferPhase::Failed { error, .. }, _) => GemSwapSessionAction::TransferError { error: error.clone() },
            (GemSwapTransferPhase::Idle, GemSwapQuotePhase::Loading { .. }) => GemSwapSessionAction::QuoteLoading,
            (GemSwapTransferPhase::Idle, GemSwapQuotePhase::Failed { error, .. }) => GemSwapSessionAction::QuoteError { error: error.clone() },
            (GemSwapTransferPhase::Idle, GemSwapQuotePhase::Ready | GemSwapQuotePhase::NoInput) => {
                if self.quote().is_some() {
                    GemSwapSessionAction::Ready
                } else {
                    GemSwapSessionAction::None
                }
            }
        }
    }

    pub fn button_action(&self, value: GemBigInt, available_balance: GemBigInt) -> GemSwapButtonAction {
        GemSwapButtonInput {
            value,
            available_balance,
            quote_error: self.quote_error(),
            transfer_error: self.transfer_error(),
        }
        .action()
    }

    pub fn button_state(&self, action: GemSwapButtonAction) -> GemSwapButtonState {
        match action {
            GemSwapButtonAction::InsufficientBalance => GemSwapButtonState::Disabled,
            _ if self.is_quote_loading() || self.is_transfer_loading() => GemSwapButtonState::Loading,
            GemSwapButtonAction::Swap if self.quote().is_none() => GemSwapButtonState::Disabled,
            GemSwapButtonAction::Swap | GemSwapButtonAction::RetryQuote | GemSwapButtonAction::RetryTransfer | GemSwapButtonAction::UseMinimumAmount { .. } => {
                GemSwapButtonState::Enabled
            }
        }
    }
}

impl GemSwapSession {
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
    use primitives::{Account, Asset, AssetType, Chain, Wallet};

    fn request(value: u32) -> GemSwapRequest {
        GemSwapRequest {
            pay_asset_id: AssetId::from_chain(Chain::Ethereum),
            receive_asset_id: AssetId::from_chain(Chain::Solana),
            value: BigUint::from(value),
            slippage_bps: None,
        }
    }

    fn quote(provider: SwapperProvider, to_value: u32) -> SwapperQuote {
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "ethereum-address"), Account::mock(Chain::Solana, "solana-address")]);
        let asset = |chain: Chain| Asset::new(AssetId::from_chain(chain), chain.to_string(), chain.to_string().to_uppercase(), 18, AssetType::NATIVE);
        SwapperQuote {
            from_value: BigUint::from(100u32),
            min_from_value: None,
            to_value: BigUint::from(to_value),
            data: swapper::ProviderData {
                provider: swapper::ProviderType::new(provider),
                slippage_bps: 50,
                routes: vec![],
            },
            request: rules::quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), BigUint::from(100u32), false, None).unwrap(),
            eta_in_seconds: None,
        }
    }

    fn results(quotes: Vec<SwapperQuote>) -> GemSwapQuotesResult {
        GemSwapQuotesResult {
            request: request(100),
            quotes,
            error: None,
        }
    }

    fn ready() -> GemSwapSession {
        GemSwapSession::default()
            .on_request_changed(Some(request(100)))
            .on_quote_results(results(vec![quote(SwapperProvider::Okx, 10), quote(SwapperProvider::Jupiter, 9)]))
    }

    #[test]
    fn test_a_changed_request_loads_and_an_empty_one_resets() {
        let loading = GemSwapSession::default().on_request_changed(Some(request(100)));
        assert_eq!(loading.quote_phase, GemSwapQuotePhase::Loading { request: request(100) });
        assert!(loading.quotes.is_none());
        assert_eq!(loading.action(), GemSwapSessionAction::QuoteLoading);

        let reset = ready().on_request_changed(None);
        assert_eq!(reset, GemSwapSession::default());
        assert!(reset.is_input_empty());
        assert_eq!(reset.action(), GemSwapSessionAction::None);
    }

    #[test]
    fn test_quote_results_pick_the_best_quote_and_report_failures() {
        let session = ready();
        assert_eq!(session.quote().unwrap().data.provider.id, SwapperProvider::Okx);
        assert_eq!(session.action(), GemSwapSessionAction::Ready);

        let failed = session.on_quote_results(GemSwapQuotesResult {
            error: Some(SwapperError::NoAvailableProvider),
            ..results(vec![])
        });
        assert!(failed.quotes.is_none());
        assert!(failed.quote().is_none());
        assert_eq!(failed.quote_error(), Some(SwapperError::NoAvailableProvider));
        assert_eq!(
            failed.action(),
            GemSwapSessionAction::QuoteError {
                error: SwapperError::NoAvailableProvider
            }
        );

        let empty = session.on_quote_results(results(vec![]));
        assert_eq!(empty.quote_error(), Some(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_the_chosen_provider_survives_a_refresh_and_falls_back_when_it_disappears() {
        let chosen = ready().on_provider_selected(SwapperProvider::Jupiter);
        assert_eq!(chosen.quote().unwrap().data.provider.id, SwapperProvider::Jupiter);

        let refreshed = chosen.on_quote_results(results(vec![quote(SwapperProvider::Okx, 11), quote(SwapperProvider::Jupiter, 8)]));
        assert_eq!(refreshed.quote().unwrap().to_value, BigUint::from(8u32));

        let without = refreshed.on_quote_results(results(vec![quote(SwapperProvider::Okx, 12)]));
        assert_eq!(without.quote().unwrap().data.provider.id, SwapperProvider::Okx);

        let new_amount = chosen.on_request_changed(Some(request(200)));
        assert!(new_amount.quote().is_none());
        assert_eq!(new_amount.selected_provider, Some(SwapperProvider::Jupiter));

        let new_pair = chosen.on_request_changed(Some(GemSwapRequest {
            receive_asset_id: AssetId::from_chain(Chain::Bitcoin),
            ..request(200)
        }));
        assert_eq!(new_pair.selected_provider, None);
    }

    #[test]
    fn test_a_loading_transfer_keeps_its_quote_and_ignores_new_results() {
        let started = ready().start_transfer().unwrap();
        assert!(started.is_transfer_loading());
        assert_eq!(started.action(), GemSwapSessionAction::TransferLoading);
        assert!(started.start_transfer().is_none());
        assert!(!started.accepts_quotes());

        let refreshed = started.on_quote_results(results(vec![quote(SwapperProvider::Okx, 99)]));
        assert_eq!(refreshed.quote().unwrap().to_value, BigUint::from(10u32));
        assert!(refreshed.is_transfer_loading());
        assert_eq!(refreshed.on_fetch_started(request(100)), refreshed);

        assert!(GemSwapSession::default().start_transfer().is_none());
    }

    #[test]
    fn test_transfer_outcomes_apply_only_to_the_transfer_that_started() {
        let started = ready().start_transfer().unwrap();
        let transfer = started.transfer_phase.clone();

        let failed = started.on_transfer_failed(transfer.clone(), SwapperError::TransactionError("boom".into()));
        assert_eq!(failed.transfer_error(), Some(SwapperError::TransactionError("boom".into())));
        assert_eq!(
            failed.action(),
            GemSwapSessionAction::TransferError {
                error: SwapperError::TransactionError("boom".into())
            }
        );
        assert!(failed.quote().is_some());
        assert_eq!(failed.on_transfer_failed(transfer.clone(), SwapperError::NoQuoteAvailable), failed);

        let handed_off = started.on_transfer_handed_off(transfer.clone());
        assert_eq!(handed_off.transfer_phase, GemSwapTransferPhase::Idle);
        assert!(handed_off.refresh_paused_until_restart);
        assert!(!handed_off.refreshes_quotes(true));
        assert!(handed_off.on_refresh_resumed().refreshes_quotes(true));
        assert!(!handed_off.on_refresh_resumed().refreshes_quotes(false));
        assert_eq!(handed_off.on_fetch_started(request(100)), handed_off);

        let abandoned = started.on_transfer_abandoned(transfer);
        assert_eq!(abandoned.transfer_phase, GemSwapTransferPhase::Idle);
        assert!(!abandoned.refresh_paused_until_restart);

        assert_eq!(ready().on_transfer_handed_off(GemSwapTransferPhase::Idle), ready());
    }

    #[test]
    fn test_quote_changes_clear_a_failed_transfer() {
        let started = ready().start_transfer().unwrap();
        let failed = started.on_transfer_failed(started.transfer_phase.clone(), SwapperError::NoQuoteAvailable);

        assert_eq!(failed.on_provider_selected(SwapperProvider::Jupiter).transfer_phase, GemSwapTransferPhase::Idle);
        assert_eq!(failed.on_refresh_requested(request(100)).transfer_phase, GemSwapTransferPhase::Idle);
        assert_eq!(failed.on_quote_invalidated().transfer_phase, GemSwapTransferPhase::Idle);
    }

    #[test]
    fn test_button_state_follows_the_phases() {
        let idle = GemSwapSession::default();
        assert_eq!(idle.button_state(GemSwapButtonAction::Swap), GemSwapButtonState::Disabled);

        let loading = idle.on_request_changed(Some(request(100)));
        assert_eq!(loading.button_state(GemSwapButtonAction::Swap), GemSwapButtonState::Loading);

        let session = ready();
        assert_eq!(session.button_action(GemBigInt::from(1), GemBigInt::from(2)), GemSwapButtonAction::Swap);
        assert_eq!(session.button_state(GemSwapButtonAction::Swap), GemSwapButtonState::Enabled);
        assert_eq!(session.button_action(GemBigInt::from(3), GemBigInt::from(2)), GemSwapButtonAction::InsufficientBalance);
        assert_eq!(session.button_state(GemSwapButtonAction::InsufficientBalance), GemSwapButtonState::Disabled);

        let started = session.start_transfer().unwrap();
        assert_eq!(started.button_state(GemSwapButtonAction::Swap), GemSwapButtonState::Loading);
        let failed = started.on_transfer_failed(started.transfer_phase.clone(), SwapperError::TransactionError("boom".into()));
        assert_eq!(failed.button_action(GemBigInt::from(1), GemBigInt::from(2)), GemSwapButtonAction::RetryTransfer);
        assert_eq!(failed.button_state(GemSwapButtonAction::RetryTransfer), GemSwapButtonState::Enabled);
    }
}
