mod rules;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use primitives::{Asset, AssetFiatValue, AssetId, Banner, BannerEvent, Currency, PerpetualBalance, TotalFiatValue, Wallet};

use crate::services::asset_discovery::GemAssetDiscoveryService;
use crate::services::assets::model::{GemAssetRowStyle, GemHeaderActions};
use crate::services::assets::rules as asset_rules;
use crate::services::balance::GemBalanceService;
use crate::services::balance::rules as balance_rules;
use crate::services::banner::{GemBannerContent, GemBannerContext, GemBannerKey, GemBannerService};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_preferences::{GemDiscoveryStep, GemWalletPreferencesService};
use crate::services::wallet_session::GemWalletSessionService;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletHomeViewState {
    pub total_value: TotalFiatValue,
    pub shows_pnl: bool,
    pub header_actions: GemHeaderActions,
    pub show_collections: bool,
    pub visible_banners: Vec<Banner>,
}

#[derive(uniffi::Object)]
pub struct GemWalletHomeService {
    balances: Arc<GemBalanceService>,
    discovery: Arc<GemAssetDiscoveryService>,
    banners: Arc<GemBannerService>,
    wallet_preferences: Arc<GemWalletPreferencesService>,
    preferences: Arc<GemPreferencesService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemWalletHomeService {
    #[uniffi::constructor]
    pub fn new(
        balances: Arc<GemBalanceService>,
        discovery: Arc<GemAssetDiscoveryService>,
        banners: Arc<GemBannerService>,
        wallet_preferences: Arc<GemWalletPreferencesService>,
        preferences: Arc<GemPreferencesService>,
        session: Arc<GemWalletSessionService>,
    ) -> Self {
        Self {
            balances,
            discovery,
            banners,
            wallet_preferences,
            preferences,
            session,
        }
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn asset_row_style(&self) -> GemAssetRowStyle {
        asset_rules::wallet_asset_row_style()
    }

    pub fn view_state(
        &self,
        wallet: Wallet,
        balances: Vec<AssetFiatValue>,
        perpetual: Option<PerpetualBalance>,
        banners: Vec<Banner>,
        is_wallet_empty: bool,
    ) -> GemWalletHomeViewState {
        let chains = wallet.chains();
        let total_value = self.total_fiat_value(balances, perpetual);
        GemWalletHomeViewState {
            shows_pnl: balance_rules::shows_pnl(&total_value),
            header_actions: rules::header_actions(wallet.wallet_type, &chains, rules::header_buttons_enabled(&banners)),
            show_collections: self.preferences.show_collections(wallet.wallet_type, chains),
            visible_banners: GemBannerContext::wallet(wallet, is_wallet_empty).visible_banners(banners),
            total_value,
        }
    }

    pub async fn update_balances(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.balances.update(self.session.current_wallet_id()?, asset_ids).await
    }

    pub fn shows_initial_loading(&self) -> bool {
        let Ok(wallet_id) = self.session.current_wallet_id() else {
            return false;
        };
        let completed = self
            .wallet_preferences
            .is_initial_load_completed(wallet_id.clone(), GemDiscoveryStep::Assets)
            .unwrap_or(true);
        rules::shows_initial_loading(completed, self.wallet_preferences.get_assets_timestamp(wallet_id))
    }

    pub async fn refresh(&self) -> Result<(), GemServiceError> {
        let wallet_id = self.session.current_wallet_id()?;
        let (balances, discovery) = futures::join!(self.balances.update_enabled_balances(wallet_id.clone()), self.discovery.discover(wallet_id));
        balances?;
        discovery
    }

    pub async fn set_asset_pinned(&self, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        self.balances.set_asset_pinned(self.session.current_wallet_id()?, asset_id, pinned).await
    }

    pub async fn set_assets_enabled(&self, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(self.session.current_wallet_id()?, asset_ids, enabled).await
    }

    pub fn banner_content(&self, event: BannerEvent, asset: Option<Asset>) -> GemBannerContent {
        self.banners.banner_content(event, asset)
    }

    pub async fn close_banner(&self, key: GemBannerKey) -> Result<(), GemServiceError> {
        self.banners.close(key).await
    }
}

impl GemWalletHomeService {
    fn total_fiat_value(&self, balances: Vec<AssetFiatValue>, perpetual: Option<PerpetualBalance>) -> TotalFiatValue {
        balance_rules::total_fiat_value(&rules::wallet_balances(balances, perpetual.filter(|_| self.includes_perpetual_collateral())))
    }

    fn includes_perpetual_collateral(&self) -> bool {
        self.session
            .get_current_wallet_id()
            .ok()
            .flatten()
            .is_some_and(|wallet_id| self.wallet_preferences.includes_perpetual_collateral(wallet_id))
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::{AssetId, Chain};

    use super::testkit::WalletHomeTestkit;
    use crate::services::wallet_preferences::GemDiscoveryStep;

    #[test]
    fn test_refresh_runs_discovery_even_when_the_balance_update_fails() {
        block_on(async {
            let testkit = WalletHomeTestkit::with_status(503);
            testkit
                .balances
                .enabled_asset_ids
                .lock()
                .unwrap()
                .insert(testkit.wallet_id.clone(), vec![AssetId::from_chain(Chain::Ethereum)]);

            assert!(testkit.service.refresh().await.is_err());

            let paths = testkit.provider.requested_paths();
            assert!(
                paths.iter().any(|path| path.contains("gemnodes.com")),
                "the balance branch never reached the gateway: {paths:?}"
            );
            assert!(paths.iter().any(|path| path.contains("devices/assets")), "the discovery branch never ran: {paths:?}");
        })
    }

    #[test]
    fn test_refresh_leaves_the_discovery_steps_incomplete_when_the_api_fails() {
        block_on(async {
            let testkit = WalletHomeTestkit::with_status(503);

            assert!(testkit.service.refresh().await.is_err());

            for step in [GemDiscoveryStep::Assets, GemDiscoveryStep::Transactions, GemDiscoveryStep::Nfts] {
                assert!(
                    !testkit.wallet_preferences.is_initial_load_completed(testkit.wallet_id.clone(), step).unwrap(),
                    "{step:?} was marked complete after a failed refresh"
                );
            }
        })
    }

    #[test]
    fn test_shows_initial_loading_until_the_assets_step_completes() {
        block_on(async {
            let testkit = WalletHomeTestkit::with_status(503);

            assert!(testkit.service.shows_initial_loading());

            testkit
                .wallet_preferences
                .set_initial_load_completed(testkit.wallet_id.clone(), GemDiscoveryStep::Assets)
                .unwrap();

            assert!(!testkit.service.shows_initial_loading());
        })
    }
}
