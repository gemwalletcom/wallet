use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Currency, Wallet, WalletId};
use swapper::{AssetList, Quote, SwapperError, SwapperSlippage};

use super::rules;
use super::{GemSwapPairSuggestion, GemSwapService, GemSwapTransfer};
use crate::config::swap_config::{get_default_slippage, get_swap_config};
use crate::models::custom_types::GemBigUint;
use crate::models::swap::GemSlippageCheck;
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::stream::GemStreamSubscriptionService;

#[derive(uniffi::Object)]
pub struct GemSwapQuoteService {
    swap: Arc<GemSwapService>,
    preferences: Arc<GemPreferencesService>,
    balances: Arc<GemBalanceService>,
    stream: Arc<GemStreamSubscriptionService>,
}

#[uniffi::export]
impl GemSwapQuoteService {
    #[uniffi::constructor]
    pub fn new(swap: Arc<GemSwapService>, preferences: Arc<GemPreferencesService>, balances: Arc<GemBalanceService>, stream: Arc<GemStreamSubscriptionService>) -> Self {
        Self {
            swap,
            preferences,
            balances,
            stream,
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

    pub async fn get_quotes(
        &self,
        wallet: Wallet,
        from_asset: Asset,
        to_asset: Asset,
        value: GemBigUint,
        use_max_amount: bool,
        slippage_bps: Option<u32>,
    ) -> Result<Vec<Quote>, SwapperError> {
        self.swap.get_quotes(wallet, from_asset, to_asset, value, use_max_amount, slippage_bps).await
    }

    pub async fn suggest_pair(&self, wallet_id: WalletId, pay_asset_id: Option<AssetId>) -> Result<Option<GemSwapPairSuggestion>, GemServiceError> {
        self.swap.suggest_pair(wallet_id, pay_asset_id).await
    }

    pub async fn get_transfer(&self, wallet: Wallet, quote: Quote) -> Result<GemSwapTransfer, SwapperError> {
        self.swap.get_transfer(wallet, quote).await
    }

    pub async fn update_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.balances.update(wallet_id, asset_ids).await
    }

    pub async fn add_prices(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.stream.add_prices(asset_ids).await
    }
}
