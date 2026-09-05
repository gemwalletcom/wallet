mod rules;

use std::sync::Arc;

use primitives::{Asset, AssetFiatValue, AssetId, BannerEvent, Currency, TotalFiatValue, Wallet};

use crate::services::asset_discovery::GemAssetDiscoveryService;
use crate::services::assets::model::GemHeaderButton;
use crate::services::balance::GemBalanceService;
use crate::services::balance::rules as balance_rules;
use crate::services::banner::{GemBannerAction, GemBannerContent, GemBannerKey, GemBannerService};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_preferences::{GemDiscoveryStep, GemWalletPreferencesService};
use crate::services::wallet_session::GemWalletSessionService;

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

    pub fn total_fiat_value(&self, balances: Vec<AssetFiatValue>) -> TotalFiatValue {
        balance_rules::total_fiat_value(&balances)
    }

    pub fn shows_pnl(&self, total: TotalFiatValue) -> bool {
        balance_rules::shows_pnl(&total)
    }

    pub fn header_buttons(&self, wallet: Wallet, is_enabled: bool) -> Vec<GemHeaderButton> {
        rules::header_buttons(&wallet, is_enabled)
    }

    pub async fn update_balances(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.balances.update(self.session.current_wallet_id()?, asset_ids).await
    }

    pub fn includes_perpetual_collateral(&self) -> bool {
        self.session
            .get_current_wallet_id()
            .ok()
            .flatten()
            .is_some_and(|wallet_id| self.wallet_preferences.includes_perpetual_collateral(wallet_id))
    }

    pub fn shows_initial_loading(&self) -> Result<bool, GemServiceError> {
        let wallet_id = self.session.current_wallet_id()?;
        let completed = self.wallet_preferences.is_initial_load_completed(wallet_id.clone(), GemDiscoveryStep::Assets)?;
        Ok(rules::shows_initial_loading(completed, self.wallet_preferences.get_assets_timestamp(wallet_id)))
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

    pub async fn apply_banner_action(&self, key: GemBannerKey, action: GemBannerAction) -> Result<(), GemServiceError> {
        self.banners.apply_action(key, action).await
    }
}
