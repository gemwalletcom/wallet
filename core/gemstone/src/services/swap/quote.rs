use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Currency};
use swapper::{AssetList, Quote, SwapperError, SwapperProvider, SwapperSlippage};

use super::rules;
use super::{GemSwapPairSuggestion, GemSwapService, GemSwapTransfer};
use crate::config::swap_config::{get_default_slippage, get_swap_config};
use crate::models::custom_types::GemBigUint;
use crate::models::swap::GemSlippageCheck;
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::stream::GemStreamSubscriptionService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemSwapQuoteService {
    swap: Arc<GemSwapService>,
    preferences: Arc<GemPreferencesService>,
    balances: Arc<GemBalanceService>,
    stream: Arc<GemStreamSubscriptionService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemSwapQuoteService {
    #[uniffi::constructor]
    pub fn new(
        swap: Arc<GemSwapService>,
        preferences: Arc<GemPreferencesService>,
        balances: Arc<GemBalanceService>,
        stream: Arc<GemStreamSubscriptionService>,
        session: Arc<GemWalletSessionService>,
    ) -> Self {
        Self {
            swap,
            preferences,
            balances,
            stream,
            session,
        }
    }

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn slippage_bps(&self) -> Option<u32> {
        self.preferences.get_swap_slippage_bps()
    }

    pub fn set_slippage_bps(&self, bps: Option<u32>) -> Result<(), GemServiceError> {
        self.preferences.set_swap_slippage_bps(bps)
    }

    pub fn slippage_check(&self, bps: u32) -> GemSlippageCheck {
        rules::slippage_check(bps, &get_swap_config())
    }

    pub fn default_slippage(&self, chain: Chain) -> SwapperSlippage {
        get_default_slippage(&chain)
    }

    pub fn refresh_interval_milliseconds(&self) -> u64 {
        rules::quote_refresh_interval_milliseconds()
    }

    pub fn quote_debounce_milliseconds(&self) -> u64 {
        rules::quote_debounce_milliseconds()
    }

    pub fn supported_assets(&self, asset_id: AssetId) -> AssetList {
        self.swap.supported_assets(asset_id)
    }

    pub async fn get_quotes(&self, from_asset: Asset, to_asset: Asset, value: GemBigUint, use_max_amount: bool, slippage_bps: Option<u32>) -> Result<Vec<Quote>, SwapperError> {
        let wallet = self.session.current_wallet().map_err(|error| SwapperError::ComputeQuoteError(error.to_string()))?;
        self.swap.get_quotes(wallet, from_asset, to_asset, value, use_max_amount, slippage_bps).await
    }

    pub async fn suggest_pair(&self, pay_asset_id: Option<AssetId>) -> Result<Option<GemSwapPairSuggestion>, GemServiceError> {
        self.swap.suggest_pair(self.session.current_wallet_id()?, pay_asset_id).await
    }

    pub fn selected_quote(&self, quotes: Vec<Quote>, preferred: Option<SwapperProvider>) -> Option<Quote> {
        rules::selected_quote(&quotes, preferred)
    }

    pub async fn get_transfer(&self, quote: Quote) -> Result<GemSwapTransfer, SwapperError> {
        let wallet = self.session.current_wallet().map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        self.swap.get_transfer(wallet, quote).await
    }

    pub async fn update_balances(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.balances.update(self.session.current_wallet_id()?, asset_ids).await
    }

    pub async fn add_prices(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.stream.add_prices(asset_ids).await
    }
}
