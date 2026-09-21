pub mod rules;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::services::localization::GemLocalizedText;
use primitives::{AssetFiatValue, AssetId, Banner, Currency, TotalFiatValue, Wallet, WalletId};

use crate::services::asset_discovery::GemAssetDiscoveryService;
use crate::services::assets::model::{GemAssetRowStyle, GemHeaderActions};
use crate::services::assets::rules as asset_rules;
use crate::services::balance::GemBalanceService;
use crate::services::balance::rules as balance_rules;
use crate::services::banner::{GemBannerContext, GemBannerKey, GemBannerRow, GemBannerService};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_preferences::{GemDiscoveryStep, GemWalletPreferencesService};
use crate::services::wallet_session::GemWalletSessionService;
pub use rules::GemPerpetualCollateral;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletHomeViewState {
    pub total_value: TotalFiatValue,
    pub total: GemFormattedNumber,
    pub pnl: Option<GemLocalizedText>,
    pub pnl_tone: GemValueTone,
    pub shows_pnl: bool,
    pub header_actions: GemHeaderActions,
    pub show_collections: bool,
    pub shows_perpetuals: bool,
    pub visible_banners: Vec<GemBannerRow>,
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

    pub fn view_state(&self, wallet: Wallet, balances: Vec<AssetFiatValue>, perpetual: Option<GemPerpetualCollateral>, banners: Vec<Banner>) -> GemWalletHomeViewState {
        let chains = wallet.chains();
        let wallet_type = wallet.wallet_type;
        let is_wallet_empty = balances.iter().all(|balance| balance.amount == 0.0);
        let total_value = self.total_fiat_value(wallet.id.clone(), balances, perpetual);
        let visible_banners = GemBannerContext::wallet(wallet, is_wallet_empty).visible_banners(banners);
        let currency = self.preferences.get_currency();
        let header = balance_rules::total_header(&total_value, currency);
        GemWalletHomeViewState {
            total: header.total,
            pnl: header.pnl,
            pnl_tone: header.pnl_tone,
            shows_pnl: balance_rules::shows_pnl(&total_value),
            header_actions: rules::header_actions(wallet_type, &chains, rules::header_buttons_enabled(&visible_banners)),
            show_collections: self.preferences.show_collections(wallet_type, chains.clone()),
            shows_perpetuals: self.preferences.show_perpetuals(wallet_type, chains),
            visible_banners,
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
        let completed = self.wallet_preferences.is_initial_load_completed(wallet_id.clone(), GemDiscoveryStep::Assets).unwrap_or(true);
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

    pub async fn close_banner(&self, key: GemBannerKey) -> Result<(), GemServiceError> {
        self.banners.close(key).await
    }
}

impl GemWalletHomeService {
    fn total_fiat_value(&self, wallet_id: WalletId, balances: Vec<AssetFiatValue>, collateral: Option<GemPerpetualCollateral>) -> TotalFiatValue {
        let collateral = collateral.filter(|_| self.wallet_preferences.includes_perpetual_collateral(wallet_id));
        balance_rules::total_fiat_value(&rules::wallet_balances(balances, collateral))
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::{AssetId, Chain};

    use super::testkit::WalletHomeTestkit;
    use crate::services::assets::model::GemHeaderActions;
    use crate::services::wallet_preferences::GemDiscoveryStep;
    use primitives::{AssetFiatValue, Banner, BannerEvent, BannerState, Wallet};

    #[test]
    fn test_the_home_state_answers_whether_perpetuals_show() {
        let testkit = WalletHomeTestkit::with_status(200);

        let state = testkit.service.view_state(Wallet::mock(), vec![], None, vec![]);

        assert_eq!(
            state.shows_perpetuals,
            testkit.preferences.show_perpetuals(Wallet::mock().wallet_type, Wallet::mock().chains()),
            "the screen reads the flag from the state instead of asking a second service"
        );
    }

    #[test]
    fn test_the_header_buttons_follow_the_banners_the_screen_shows() {
        let testkit = WalletHomeTestkit::with_status(200);
        let warning = |state| Banner::mock(BannerEvent::AccountBlockedMultiSignature, state);
        let buttons_enabled = |banners: Vec<Banner>| match testkit.service.view_state(Wallet::mock(), vec![], None, banners).header_actions {
            GemHeaderActions::Buttons { buttons } => buttons.iter().all(|button| button.is_enabled),
            GemHeaderActions::WatchOnly => true,
        };

        assert!(!buttons_enabled(vec![warning(BannerState::AlwaysActive)]));
        assert!(buttons_enabled(vec![warning(BannerState::Cancelled)]), "a warning the screen does not show cannot block the buttons");
    }

    #[test]
    fn test_the_onboarding_banner_shows_only_while_every_balance_is_zero() {
        let testkit = WalletHomeTestkit::with_status(200);
        let onboarding = Banner {
            asset: None,
            ..Banner::mock(BannerEvent::Onboarding, BannerState::Active)
        };
        let value = |amount: f64| AssetFiatValue {
            amount,
            price: 1.0,
            price_change_percentage_24h: 0.0,
        };
        let shown = |balances: Vec<AssetFiatValue>| testkit.service.view_state(Wallet::mock(), balances, None, vec![onboarding.clone()]).visible_banners.len();

        assert_eq!(shown(vec![value(0.0)]), 1);
        assert_eq!(shown(vec![value(0.0), value(2.0)]), 0);
    }

    #[test]
    fn test_refresh_runs_discovery_even_when_the_balance_update_fails() {
        block_on(async {
            let testkit = WalletHomeTestkit::with_status(503);
            testkit.balances.enabled_asset_ids.lock().unwrap().insert(testkit.wallet_id.clone(), vec![AssetId::from_chain(Chain::Ethereum)]);

            assert!(testkit.service.refresh().await.is_err());

            let paths = testkit.provider.requested_paths();
            assert!(paths.iter().any(|path| path.contains("gemnodes.com")), "the balance branch never reached the gateway: {paths:?}");
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

            testkit.wallet_preferences.set_initial_load_completed(testkit.wallet_id.clone(), GemDiscoveryStep::Assets).unwrap();

            assert!(!testkit.service.shows_initial_loading());
        })
    }
}
