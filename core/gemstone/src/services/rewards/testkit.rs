use std::sync::Arc;

use primitives::{RedemptionResult, Wallet};

use super::GemRewardsService;
use crate::api::GemDeviceApiClient;
use crate::services::asset_discovery::testkit::DiscoveryTestkit;
use crate::services::auth::GemAuthService;
use crate::services::balance::testkit::MemoryBalanceStore;
use crate::services::device::GemDeviceKeyService;
use crate::services::wallet::testkit::{PHRASE, WalletTestkit};
use crate::testkit::{EmptyPreferences, TestAlienProvider};

pub const TEST_NONCE: &str = r#"{"nonce":"nonce-1","timestamp":1}"#;

pub struct RewardsTestkit {
    pub service: GemRewardsService,
    pub balances: Arc<MemoryBalanceStore>,
    pub wallet: Wallet,
    pub provider: Arc<TestAlienProvider>,
    pub wallets: WalletTestkit,
}

impl RewardsTestkit {
    pub async fn with_redemption(result: &RedemptionResult) -> Self {
        Self::with_provider(Arc::new(TestAlienProvider::with_json_by_path(200, &[("auth/nonce", TEST_NONCE), ("rewards/redeem", &serde_json::to_string(result).unwrap())]))).await
    }

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
