use crate::services::amount::model::GemNumberFormat;
use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Currency};
use swapper::{Quote, SwapperError, SwapperSlippage};

use super::slippage::{GemSlippageSelection, GemSlippageSession};

use super::model::{GemSwapPairSelection, GemSwapSide};
use super::rules;
use super::{GemSwapPairSuggestion, GemSwapService, GemSwapSession, GemSwapTransfer};
use crate::config::swap_config::get_default_slippage;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::failures::{StepFailure, record_result};
use crate::services::preferences::GemPreferencesService;
use crate::services::stream::GemStreamSubscriptionService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSwapPairStep {
    AddPrices,
    UpdateBalances,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemSwapPairFailure {
    pub step: GemSwapPairStep,
    pub message: String,
}

impl StepFailure for GemSwapPairFailure {
    type Step = GemSwapPairStep;

    fn new(step: GemSwapPairStep, message: String) -> Self {
        Self { step, message }
    }
}

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
    pub fn new(swap: Arc<GemSwapService>, preferences: Arc<GemPreferencesService>, balances: Arc<GemBalanceService>, stream: Arc<GemStreamSubscriptionService>, session: Arc<GemWalletSessionService>) -> Self {
        Self {
            swap,
            preferences,
            balances,
            stream,
            session,
        }
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn new_session(&self) -> GemSwapSession {
        GemSwapSession::default()
    }

    pub fn slippage_bps(&self) -> Option<u32> {
        self.preferences.get_swap_slippage_bps()
    }

    pub fn set_slippage_bps(&self, bps: Option<u32>) -> Result<(), GemServiceError> {
        self.preferences.set_swap_slippage_bps(bps)
    }

    pub fn select_pair_asset(&self, selection: GemSwapPairSelection, side: GemSwapSide, asset_id: AssetId) -> GemSwapPairSelection {
        rules::select_pair_asset(selection, side, asset_id)
    }

    pub fn new_slippage_session(&self, selection: GemSlippageSelection) -> GemSlippageSession {
        GemSlippageSession::new(selection)
    }

    pub fn slippage_bps_from_percent(&self, percent: f64) -> Option<u32> {
        rules::slippage_bps_from_percent(percent)
    }

    pub fn amount_for_percent(&self, available: GemBigInt, percent: u32) -> GemBigInt {
        rules::amount_for_percent(&available, percent)
    }

    pub fn slippage_percent(&self, bps: u32) -> f64 {
        rules::slippage_percent(bps)
    }

    pub fn slippage_percent_text(&self, bps: u32, format: GemNumberFormat) -> String {
        rules::slippage_percent_text(bps, &format.decimal_separator)
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

    pub async fn get_quotes(&self, from_asset: Asset, to_asset: Asset, value: GemBigUint, use_max_amount: bool, slippage_bps: Option<u32>) -> Result<Vec<Quote>, SwapperError> {
        let wallet = self.session.require_current_wallet().await.map_err(|error| SwapperError::ComputeQuoteError(error.to_string()))?;
        self.swap.get_quotes(wallet, from_asset, to_asset, value, use_max_amount, slippage_bps).await
    }

    pub async fn suggest_pair(&self, pay_asset_id: Option<AssetId>) -> Option<GemSwapPairSuggestion> {
        let Ok(wallet) = self.session.require_current_wallet().await else { return None };
        self.swap.suggest_pair(wallet, pay_asset_id).await.ok().flatten()
    }

    pub async fn get_transfer(&self, quote: Quote) -> Result<GemSwapTransfer, SwapperError> {
        let wallet = self.session.require_current_wallet().await.map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        self.swap.get_transfer(wallet, quote).await
    }

    pub async fn refresh_pair(&self, asset_ids: Vec<AssetId>) -> Vec<GemSwapPairFailure> {
        let mut failures = Vec::new();
        if asset_ids.is_empty() {
            return failures;
        }
        let wallet_id = self.session.current_wallet_id();
        let (prices, balances) = futures::join!(self.stream.add_prices(asset_ids.clone()), async {
            match wallet_id {
                Ok(wallet_id) => self.balances.update(wallet_id, asset_ids).await,
                Err(error) => Err(error),
            }
        });
        record_result(&mut failures, GemSwapPairStep::AddPrices, prices);
        record_result(&mut failures, GemSwapPairStep::UpdateBalances, balances);
        failures
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use std::sync::atomic::Ordering;

    use super::super::testkit::SwapQuoteTestkit;
    use super::*;

    fn pair() -> Vec<AssetId> {
        vec![AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Solana)]
    }

    #[test]
    fn test_a_changed_pair_updates_the_balances_the_screen_spends_from() {
        block_on(async {
            let testkit = SwapQuoteTestkit::with_status(503);

            let failures = testkit.service.refresh_pair(pair()).await;

            let steps: Vec<GemSwapPairStep> = failures.iter().map(|failure| failure.step).collect();
            assert!(steps.contains(&GemSwapPairStep::UpdateBalances), "{failures:?}");
            assert_eq!(testkit.connection.messages(), vec![("subscribe", pair())], "the prices are asked for once, for the whole pair");
        })
    }

    #[test]
    fn test_a_failed_subscription_still_updates_the_balances() {
        block_on(async {
            let testkit = SwapQuoteTestkit::with_status(503);
            testkit.connection.fail_next_send.store(true, Ordering::SeqCst);

            let failures = testkit.service.refresh_pair(pair()).await;

            let steps: Vec<GemSwapPairStep> = failures.iter().map(|failure| failure.step).collect();
            assert!(steps.contains(&GemSwapPairStep::AddPrices), "{failures:?}");
            assert!(steps.contains(&GemSwapPairStep::UpdateBalances), "{failures:?}");
        })
    }

    #[test]
    fn test_without_a_current_wallet_only_the_balances_fail() {
        block_on(async {
            let testkit = SwapQuoteTestkit::with_status(200);
            testkit.discovery.session.set_current_wallet_id(None).unwrap();

            let failures = testkit.service.refresh_pair(pair()).await;

            assert_eq!(failures.len(), 1);
            assert_eq!(failures[0].step, GemSwapPairStep::UpdateBalances);
            assert_eq!(testkit.connection.messages(), vec![("subscribe", pair())]);
        })
    }

    #[test]
    fn test_an_empty_pair_asks_for_nothing() {
        block_on(async {
            let testkit = SwapQuoteTestkit::with_status(503);

            assert!(testkit.service.refresh_pair(vec![]).await.is_empty());
            assert!(testkit.connection.messages().is_empty());
        })
    }
}
