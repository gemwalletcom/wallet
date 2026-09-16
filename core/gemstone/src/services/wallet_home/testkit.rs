use std::sync::Arc;

use primitives::WalletId;

use super::GemWalletHomeService;
use crate::services::asset_discovery::testkit::DiscoveryTestkit;
use crate::services::balance::testkit::RecordingBalanceStore;
use crate::services::banner::GemBannerService;
use crate::services::banner::testkit::MemoryBannerStore;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::testkit::TestAlienProvider;

pub struct WalletHomeTestkit {
    pub service: GemWalletHomeService,
    pub provider: Arc<TestAlienProvider>,
    pub balances: Arc<RecordingBalanceStore>,
    pub wallet_preferences: Arc<GemWalletPreferencesService>,
    pub wallet_id: WalletId,
}

impl WalletHomeTestkit {
    pub fn with_status(status: u16) -> Self {
        let discovery = DiscoveryTestkit::with_status(status);
        let service = GemWalletHomeService::new(
            discovery.balance.clone(),
            discovery.discovery.clone(),
            Arc::new(GemBannerService::new(Arc::new(MemoryBannerStore::default()))),
            discovery.wallet_preferences.clone(),
            discovery.preferences.clone(),
            discovery.session.clone(),
        );
        Self {
            service,
            provider: discovery.provider,
            balances: discovery.balances,
            wallet_preferences: discovery.wallet_preferences,
            wallet_id: discovery.wallet_id,
        }
    }
}
