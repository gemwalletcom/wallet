use std::sync::Arc;

use primitives::{Wallet, WalletId};

use super::GemWalletHomeService;
use crate::services::asset_discovery::testkit::DiscoveryTestkit;
use crate::services::balance::testkit::MemoryBalanceStore;
use crate::services::banner::GemBannerService;
use crate::services::banner::testkit::MemoryBannerStore;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::testkit::TestAlienProvider;

pub struct WalletHomeTestkit {
    pub service: GemWalletHomeService,
    pub provider: Arc<TestAlienProvider>,
    pub balances: Arc<MemoryBalanceStore>,
    pub wallet_preferences: Arc<GemWalletPreferencesService>,
    pub preferences: Arc<GemPreferencesService>,
    pub wallet_id: WalletId,
}

impl WalletHomeTestkit {
    pub fn with_status(status: u16) -> Self {
        Self::with_wallet(status, Wallet::mock())
    }

    pub fn with_wallet(status: u16, wallet: Wallet) -> Self {
        let discovery = DiscoveryTestkit::with_provider(Arc::new(TestAlienProvider::with_status(status)), wallet);
        let service = GemWalletHomeService::new(
            discovery.balance.clone(),
            discovery.discovery.clone(),
            Arc::new(GemBannerService::new(Arc::new(MemoryBannerStore::default()), primitives::Platform::IOS)),
            discovery.wallet_preferences.clone(),
            discovery.preferences.clone(),
            discovery.session.clone(),
        );
        Self {
            service,
            provider: discovery.provider,
            balances: discovery.balances,
            wallet_preferences: discovery.wallet_preferences,
            preferences: discovery.preferences,
            wallet_id: discovery.wallet_id,
        }
    }
}
