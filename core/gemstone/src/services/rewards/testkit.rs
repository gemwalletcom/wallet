use std::sync::Arc;

use primitives::Wallet;

use super::GemRewardsService;
use crate::api::GemDeviceApiClient;
use crate::gateway::EmptyPreferences;
use crate::services::asset_discovery::testkit::DiscoveryTestkit;
use crate::services::auth::GemAuthService;
use crate::services::balance::testkit::RecordingBalanceStore;
use crate::services::device::GemDeviceKeyService;
use crate::services::wallet::testkit::{PHRASE, WalletTestkit};
use crate::testkit::TestAlienProvider;

pub struct RewardsTestkit {
    pub service: GemRewardsService,
    pub balances: Arc<RecordingBalanceStore>,
    pub wallet: Wallet,
    pub provider: Arc<TestAlienProvider>,
    pub wallets: WalletTestkit,
}

impl RewardsTestkit {
    pub async fn with_provider(provider: Arc<TestAlienProvider>) -> Self {
        let wallets = WalletTestkit::new();
        let wallet = wallets.import("Rewards", PHRASE).await;
        let discovery = DiscoveryTestkit::with_provider(provider.clone(), wallet.clone());
        let api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let auth = Arc::new(GemAuthService::new(
            api.clone(),
            wallets.keystore.clone(),
            wallets.passwords.clone(),
            Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences))),
        ));
        let service = GemRewardsService::new(api, auth, discovery.balance.clone());
        Self {
            service,
            balances: discovery.balances,
            wallet,
            provider,
            wallets,
        }
    }
}
