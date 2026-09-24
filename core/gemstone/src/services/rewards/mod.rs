use std::sync::Arc;

use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::{AuthenticatedRequest, ReferralCode, Rewards, Wallet, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::models::state::GemLoadState;
use crate::services::auth::GemAuthService;
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::wallet_session::rules as session_rules;

pub mod model;
pub mod rules;
pub mod session;
#[cfg(test)]
pub(crate) mod testkit;

pub use model::{GemIncomingCode, GemRewardsResult, GemRewardsState, GemRewardsViewState};
pub use session::GemRewardsSession;

#[uniffi::export]
pub fn incoming_referral_code(code: Option<String>, wallets: Vec<Wallet>) -> Option<GemIncomingCode> {
    rules::incoming_code(code.as_deref(), &session_rules::rewards_wallets(wallets))
}

#[derive(uniffi::Object)]
pub struct GemRewardsService {
    api: Arc<GemDeviceApiClient>,
    auth: Arc<GemAuthService>,
    balance: Arc<GemBalanceService>,
}

#[uniffi::export]
impl GemRewardsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, auth: Arc<GemAuthService>, balance: Arc<GemBalanceService>) -> Self {
        Self { api, auth, balance }
    }

    pub fn wallets(&self, wallets: Vec<Wallet>) -> Vec<Wallet> {
        session_rules::rewards_wallets(wallets)
    }

    pub fn selected_wallet(&self, current: Option<Wallet>, wallets: Vec<Wallet>) -> Option<Wallet> {
        session_rules::rewards_wallet(current, &self.wallets(wallets))
    }

    pub async fn refresh(&self, wallet_id: WalletId) -> GemRewardsResult {
        let rewards = self.get_rewards(wallet_id.clone()).await;
        GemRewardsResult {
            wallet_id,
            state: GemLoadState::of(&rewards),
            rewards: rewards.ok(),
        }
    }

    pub async fn create_referral(&self, wallet: Wallet, code: String) -> Result<Rewards, GemServiceError> {
        let wallet_id = wallet.id.id();
        let request = AuthenticatedRequest {
            auth: self.auth.get_auth_payload(wallet).await?,
            data: ReferralCode { code: code.trim().to_string() },
        };
        Ok(self.api.client.create_referral(wallet_id, request).await.map_err(GemApiError::from)?)
    }

    pub async fn use_referral_code(&self, wallet: Wallet, code: String) -> Result<Rewards, GemServiceError> {
        let wallet_id = wallet.id.clone();
        let request = AuthenticatedRequest {
            auth: self.auth.get_auth_payload(wallet).await?,
            data: ReferralCode { code: code.trim().to_string() },
        };
        self.api.client.use_referral_code(wallet_id.id(), request).await.map_err(GemApiError::from)?;
        self.get_rewards(wallet_id).await
    }

    pub async fn redeem(&self, wallet: Wallet, redemption_id: String) -> Result<RedemptionResult, GemServiceError> {
        let wallet_id = wallet.id.clone();
        let request = AuthenticatedRequest {
            auth: self.auth.get_auth_payload(wallet).await?,
            data: RedemptionRequest { id: redemption_id },
        };
        let result = self.api.client.redeem_rewards(wallet_id.id(), request).await.map_err(GemApiError::from)?;
        if let Some(asset) = &result.redemption.option.asset {
            self.balance.set_assets_enabled(wallet_id, vec![asset.id.clone()], true).await?;
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

            let redeemed = testkit.service.redeem(testkit.wallet.clone(), "option-1".to_string()).await.unwrap();

            assert_eq!(redeemed.redemption.id, 7);
            let writes = testkit.balances.enable_writes.lock().unwrap();
            assert_eq!(*writes, vec![(vec![asset.id.clone()], true)]);
        })
    }

    #[test]
    fn test_redeeming_points_touches_no_balance() {
        block_on(async {
            let testkit = RewardsTestkit::with_redemption(&RedemptionResult::mock(None)).await;

            testkit.service.redeem(testkit.wallet.clone(), "option-1".to_string()).await.unwrap();

            assert!(testkit.balances.enable_writes.lock().unwrap().is_empty());
        })
    }

    #[test]
    fn test_a_redemption_that_fails_enables_nothing() {
        block_on(async {
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(200, &[("auth/nonce", TEST_NONCE), ("rewards/redeem", "not json")]))).await;

            assert!(testkit.service.redeem(testkit.wallet.clone(), "option-1".to_string()).await.is_err());
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

            let rewards = testkit.service.use_referral_code(testkit.wallet.clone(), "code".to_string()).await.unwrap();

            assert_eq!(rewards.used_referral_code.as_deref(), Some("code"), "the used code answers with the state it produced");
            let paths = testkit.provider.requested_paths();
            let nonce = paths.iter().position(|path| path.contains("auth/nonce"));
            let referral = paths.iter().position(|path| path.contains("referrals/use"));
            assert!(nonce < referral, "the referral call did not wait for the nonce: {paths:?}");
        })
    }

    #[test]
    fn test_an_incoming_code_is_activated_with_one_wallet_and_confirmed_with_more() {
        block_on(async {
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(200, &[("auth/nonce", TEST_NONCE)]))).await;
            let other = Wallet::mock_with_id(primitives::WalletId::Multicoin("0x2".to_string()), &[Chain::Ethereum]);

            assert_eq!(incoming_referral_code(Some("friend".to_string()), vec![testkit.wallet.clone()]), Some(GemIncomingCode::Activate { code: "friend".to_string() }));
            assert_eq!(
                incoming_referral_code(Some("friend".to_string()), vec![testkit.wallet.clone(), other]),
                Some(GemIncomingCode::Confirm { code: "friend".to_string() })
            );
            assert_eq!(incoming_referral_code(Some("  ".to_string()), vec![testkit.wallet.clone()]), None);
            assert_eq!(incoming_referral_code(Some("friend".to_string()), vec![]), None, "no wallet decides nothing yet");
            assert_eq!(incoming_referral_code(None, vec![testkit.wallet.clone()]), None);
        })
    }

    #[test]
    fn test_a_rewards_error_answered_with_ok_status_surfaces_its_message() {
        block_on(async {
            let username_error = r#"{"error":{"message":"Username must contain only letters and digits"}}"#;
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(
                200,
                &[("auth/nonce", TEST_NONCE), ("referrals/create", username_error), ("referrals/use", username_error)],
            )))
            .await;
            let expected = GemServiceError::Api {
                msg: "Username must contain only letters and digits".to_string(),
            };

            let created = testkit.service.create_referral(testkit.wallet.clone(), "code".to_string()).await.unwrap_err();
            let used = testkit.service.use_referral_code(testkit.wallet.clone(), "code".to_string()).await.unwrap_err();

            assert_eq!(created, expected);
            assert_eq!(used, expected);
        })
    }

    #[test]
    fn test_a_wallet_with_no_auth_account_never_reaches_the_api() {
        block_on(async {
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(200, &[("auth/nonce", TEST_NONCE)]))).await;
            let wallet = Wallet {
                accounts: Vec::new(),
                ..testkit.wallet.clone()
            };

            assert!(testkit.service.create_referral(wallet, "code".to_string()).await.is_err());
            assert!(!testkit.provider.requested_paths().iter().any(|path| path.contains("referrals/create")));
            assert!(testkit.wallets.keystore_path(&testkit.wallet).exists());
        })
    }
}
