use std::sync::Arc;

use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::{AuthenticatedRequest, Feature, ReferralCode, Rewards, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::models::state::GemLoadState;
use crate::services::auth::GemAuthService;
use crate::services::balance::GemBalanceService;
use crate::services::config::GemConfigService;
use crate::services::error::GemServiceError;
use crate::services::wallet_session::GemWalletSessionService;

pub mod model;
pub mod rules;
pub mod session;
#[cfg(test)]
pub(crate) mod testkit;

pub use model::{GemIncomingCode, GemRewardsResult, GemRewardsState, GemRewardsViewState, GemRewardsWallet};
pub use session::GemRewardsSession;

#[derive(uniffi::Object)]
pub struct GemRewardsService {
    api: Arc<GemDeviceApiClient>,
    auth: Arc<GemAuthService>,
    balance: Arc<GemBalanceService>,
    session: Arc<GemWalletSessionService>,
    config: Arc<GemConfigService>,
}

#[uniffi::export]
impl GemRewardsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, auth: Arc<GemAuthService>, balance: Arc<GemBalanceService>, session: Arc<GemWalletSessionService>, config: Arc<GemConfigService>) -> Self {
        Self { api, auth, balance, session, config }
    }

    pub fn is_available(&self) -> bool {
        self.config.is_feature_enabled(Feature::Rewards)
    }

    pub async fn refresh(&self, wallet_id: WalletId) -> GemRewardsResult {
        let rewards = self.get_rewards(wallet_id.clone()).await;
        GemRewardsResult {
            wallet_id,
            state: GemLoadState::of(&rewards),
            rewards: rewards.ok(),
        }
    }

    pub async fn create_referral(&self, wallet_id: WalletId, code: String) -> Result<Rewards, GemServiceError> {
        let wallet = self.session.require_wallet(wallet_id).await?;
        let wallet_id = wallet.id.id();
        let request = AuthenticatedRequest {
            auth: self.auth.get_auth_payload(wallet).await?,
            data: ReferralCode { code: code.trim().to_string() },
        };
        Ok(self.api.client.create_referral(wallet_id, request).await.map_err(GemApiError::from)?)
    }

    pub async fn use_referral_code(&self, wallet_id: WalletId, code: String) -> Result<Rewards, GemServiceError> {
        let wallet = self.session.require_wallet(wallet_id.clone()).await?;
        let request = AuthenticatedRequest {
            auth: self.auth.get_auth_payload(wallet).await?,
            data: ReferralCode { code: code.trim().to_string() },
        };
        self.api.client.use_referral_code(wallet_id.id(), request).await.map_err(GemApiError::from)?;
        self.get_rewards(wallet_id).await
    }

    pub async fn redeem(&self, wallet_id: WalletId, redemption_id: String) -> Result<RedemptionResult, GemServiceError> {
        let wallet = self.session.require_wallet(wallet_id.clone()).await?;
        let request = AuthenticatedRequest {
            auth: self.auth.get_auth_payload(wallet).await?,
            data: RedemptionRequest { id: redemption_id },
        };
        let result = self.api.client.redeem_rewards(wallet_id.id(), request).await.map_err(GemApiError::from)?;
        if let Some(asset) = &result.redemption.option.asset {
            self.balance.enable_assets(wallet_id, vec![asset.id.clone()]).await?;
        }
        Ok(result)
    }
}

impl GemRewardsService {
    async fn get_rewards(&self, wallet_id: WalletId) -> Result<Rewards, GemServiceError> {
        Ok(self.api.client.get_rewards(wallet_id.id()).await.map_err(GemApiError::from)?)
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::{Asset, Chain};

    use super::testkit::{RewardsTestkit, TEST_NONCE};
    use super::*;
    use crate::testkit::TestAlienProvider;

    #[test]
    fn test_redeeming_an_asset_enables_its_balance() {
        block_on(async {
            let asset = Asset::from_chain(Chain::Ethereum);
            let testkit = RewardsTestkit::with_redemption(&RedemptionResult::mock(Some(asset.clone()))).await;

            let redeemed = testkit.service.redeem(testkit.wallet.id.clone(), "option-1".to_string()).await.unwrap();

            assert_eq!(redeemed.redemption.id, 7);
            let writes = testkit.balances.enable_writes.lock().unwrap();
            assert_eq!(*writes, vec![(vec![asset.id.clone()], true)]);
        })
    }

    #[test]
    fn test_redeeming_points_touches_no_balance() {
        block_on(async {
            let testkit = RewardsTestkit::with_redemption(&RedemptionResult::mock(None)).await;

            testkit.service.redeem(testkit.wallet.id.clone(), "option-1".to_string()).await.unwrap();

            assert!(testkit.balances.enable_writes.lock().unwrap().is_empty());
        })
    }

    #[test]
    fn test_a_redemption_that_fails_enables_nothing() {
        block_on(async {
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(200, &[("auth/nonce", TEST_NONCE), ("rewards/redemptions", "not json")]))).await;

            assert!(testkit.service.redeem(testkit.wallet.id.clone(), "option-1".to_string()).await.is_err());
            assert!(testkit.balances.enable_writes.lock().unwrap().is_empty());
        })
    }

    #[test]
    fn test_a_referral_call_signs_with_the_wallet_before_it_reaches_the_api() {
        block_on(async {
            let used = Rewards {
                used_referral_code: Some("code".to_string()),
                ..Rewards::default()
            };
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(
                200,
                &[("auth/nonce", TEST_NONCE), ("referrals/use", "true"), ("devices/rewards", &serde_json::to_string(&used).unwrap())],
            )))
            .await;

            let rewards = testkit.service.use_referral_code(testkit.wallet.id.clone(), "code".to_string()).await.unwrap();

            assert_eq!(rewards.used_referral_code.as_deref(), Some("code"), "the used code answers with the state it produced");
            let paths = testkit.provider.requested_paths();
            let nonce = paths.iter().position(|path| path.contains("auth/nonce"));
            let referral = paths.iter().position(|path| path.contains("referrals/use"));
            assert!(nonce < referral, "the referral call did not wait for the nonce: {paths:?}");
        })
    }

    #[test]
    fn test_a_refresh_answers_for_the_wallet_it_was_asked_about() {
        block_on(async {
            let rewards = Rewards {
                code: Some("GEM123".to_string()),
                ..Rewards::default()
            };
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(200, &[("devices/rewards", &serde_json::to_string(&rewards).unwrap())]))).await;

            let result = testkit.service.refresh(testkit.wallet.id.clone()).await;

            assert_eq!(result.wallet_id, testkit.wallet.id);
            assert_eq!(result.state, GemLoadState::Data);
            assert_eq!(result.rewards.and_then(|rewards| rewards.code).as_deref(), Some("GEM123"));
        })
    }

    #[test]
    fn test_a_rewards_error_answered_with_ok_status_surfaces_its_message() {
        block_on(async {
            let username_error = r#"{"error":{"message":"Username must contain only letters and digits"}}"#;
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(
                200,
                &[("auth/nonce", TEST_NONCE), ("referrals/use", username_error), ("rewards/referrals", username_error)],
            )))
            .await;
            let expected = GemServiceError::Api {
                msg: "Username must contain only letters and digits".to_string(),
            };

            let created = testkit.service.create_referral(testkit.wallet.id.clone(), "code".to_string()).await.unwrap_err();
            let used = testkit.service.use_referral_code(testkit.wallet.id.clone(), "code".to_string()).await.unwrap_err();

            assert_eq!(created, expected);
            assert_eq!(used, expected);
        })
    }

    #[test]
    fn test_a_wallet_with_no_auth_account_never_reaches_the_api() {
        block_on(async {
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(200, &[("auth/nonce", TEST_NONCE)]))).await;
            testkit.wallets.wallets.wallets.lock().unwrap().iter_mut().for_each(|wallet| wallet.accounts.clear());

            assert!(testkit.service.create_referral(testkit.wallet.id.clone(), "code".to_string()).await.is_err());
            assert!(!testkit.provider.requested_paths().iter().any(|path| path.contains("rewards/referrals")));
            assert!(testkit.wallets.keystore_path(&testkit.wallet).exists());
        })
    }
}
