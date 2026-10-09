use std::sync::Arc;

use primitives::{Asset, AssetId, Currency, Feature, Wallet};
use swapper::{FetchQuoteData, Quote, SwapperError};

use super::model::{GemSwapPairSelection, GemSwapSide};
use super::rules;
use super::session::GemSwapQuoteInput;
use super::{GemSwapPairSuggestion, GemSwapService, GemSwapSession};
use crate::models::custom_types::GemBigInt;
use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::config::GemConfigService;
use crate::services::error::GemServiceError;
use crate::services::failures::{StepFailure, record_result};
use crate::services::preferences::GemPreferencesService;
use crate::services::stream::GemStreamSubscriptionService;
use crate::services::transfer::GemTransferData;
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
    assets: Arc<GemAssetsService>,
    preferences: Arc<GemPreferencesService>,
    balances: Arc<GemBalanceService>,
    stream: Arc<GemStreamSubscriptionService>,
    session: Arc<GemWalletSessionService>,
    config: Arc<GemConfigService>,
}

#[uniffi::export]
impl GemSwapQuoteService {
    #[uniffi::constructor]
    pub fn new(
        swap: Arc<GemSwapService>,
        assets: Arc<GemAssetsService>,
        preferences: Arc<GemPreferencesService>,
        balances: Arc<GemBalanceService>,
        stream: Arc<GemStreamSubscriptionService>,
        session: Arc<GemWalletSessionService>,
        config: Arc<GemConfigService>,
    ) -> Self {
        Self {
            swap,
            assets,
            preferences,
            balances,
            stream,
            session,
            config,
        }
    }

    pub fn is_available(&self) -> bool {
        self.config.is_feature_enabled(Feature::Swap)
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

    pub fn amount_for_percent(&self, available: GemBigInt, percent: u32) -> GemBigInt {
        rules::amount_for_percent(&available, percent)
    }

    pub async fn get_quotes(&self, input: GemSwapQuoteInput) -> Result<Vec<Quote>, SwapperError> {
        let request = input.request;
        let (wallet, from_asset, to_asset) = self.pair(request.pay_asset_id, request.receive_asset_id).await.map_err(|error| SwapperError::ComputeQuoteError(error.to_string()))?;
        self.swap.get_quotes(wallet, from_asset, to_asset, request.value, input.use_max_amount, request.slippage_bps).await
    }

    pub async fn suggest_pair(&self, pay_asset_id: Option<AssetId>) -> Option<GemSwapPairSuggestion> {
        let Ok(wallet) = self.session.require_current_wallet().await else { return None };
        self.swap.suggest_pair(wallet, pay_asset_id).await.ok().flatten()
    }

    pub async fn transfer_data(&self, quote: Quote) -> Result<GemTransferData, SwapperError> {
        let from_asset_id = AssetId::new(&quote.request.from_asset.id).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset_id = AssetId::new(&quote.request.to_asset.id).ok_or(SwapperError::NotSupportedAsset)?;
        let (wallet, from_asset, to_asset) = self.pair(from_asset_id, to_asset_id).await.map_err(|error| SwapperError::TransactionError(error.to_string()))?;
        self.swap.build_transfer(&wallet, &quote, (from_asset, to_asset), FetchQuoteData::None).await
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

impl GemSwapQuoteService {
    async fn pair(&self, from_asset_id: AssetId, to_asset_id: AssetId) -> Result<(Wallet, Asset, Asset), GemServiceError> {
        futures::try_join!(self.session.require_current_wallet(), self.assets.ensure_asset(from_asset_id), self.assets.ensure_asset(to_asset_id))
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::Chain;
    use std::sync::atomic::Ordering;

    use primitives::TransactionInputType;
    use swapper::testkit::MockSwapper;
    use swapper::{FetchQuoteData, SwapperProvider};

    use super::super::session::GemSwapRequest;
    use super::super::testkit::SwapQuoteTestkit;
    use super::*;

    fn pair() -> Vec<AssetId> {
        vec![AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Solana)]
    }

    fn echoing_swapper() -> MockSwapper {
        MockSwapper::new(SwapperProvider::UniswapV3, |request| Ok(Quote::mock_with_request(request)))
    }

    const ONE_ETH: u64 = 1_000_000_000_000_000_000;

    fn eth_to_usdc() -> GemSwapQuoteInput {
        GemSwapQuoteInput {
            request: GemSwapRequest {
                pay_asset_id: Asset::mock_eth().id,
                receive_asset_id: Asset::mock_ethereum_usdc().id,
                value: ONE_ETH.into(),
                slippage_bps: Some(150),
            },
            use_max_amount: true,
        }
    }

    #[test]
    fn test_quotes_ask_for_the_stored_pair_the_input_names() {
        block_on(async {
            let testkit = SwapQuoteTestkit::with_swapper(echoing_swapper(), vec![Asset::mock_eth(), Asset::mock_ethereum_usdc()]);

            let quotes = testkit.service.get_quotes(eth_to_usdc()).await.unwrap();

            let wallet = testkit.discovery.session.require_current_wallet().await.unwrap();
            let direct = testkit.swap.get_quotes(wallet, Asset::mock_eth(), Asset::mock_ethereum_usdc(), ONE_ETH.into(), true, Some(150)).await.unwrap();
            assert_eq!(quotes, direct, "the input names the same quotes the assets used to");
        })
    }

    #[test]
    fn test_the_transfer_carries_the_stored_assets_of_the_quote() {
        block_on(async {
            let swapper = echoing_swapper();
            let builds = swapper.builds();
            let testkit = SwapQuoteTestkit::with_swapper(swapper, vec![Asset::mock_eth(), Asset::mock_ethereum_usdc()]);
            let quote = testkit.service.get_quotes(eth_to_usdc()).await.unwrap().remove(0);

            let transfer = testkit.service.transfer_data(quote).await.unwrap();

            let TransactionInputType::Swap { from_asset, to_asset, .. } = &transfer.input_type else {
                panic!("a swap builds swap data: {:?}", transfer.input_type)
            };
            assert_eq!(from_asset, &Asset::mock_eth());
            assert_eq!(to_asset, &Asset::mock_ethereum_usdc());
            assert_eq!(builds.lock().unwrap().clone(), vec![FetchQuoteData::None], "the transfer is built without a signature");
        })
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
