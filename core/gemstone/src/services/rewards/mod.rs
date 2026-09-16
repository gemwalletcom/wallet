use std::sync::Arc;

use chrono::Utc;
use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::{AuthenticatedRequest, ReferralCode, Rewards, Wallet, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::auth::GemAuthService;
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::wallet_session::rules as session_rules;

pub mod model;
pub mod rules;
#[cfg(test)]
pub(crate) mod testkit;

pub use model::GemRewardsState;

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

    pub fn state(&self, rewards: Option<Rewards>) -> GemRewardsState {
        rules::state(rewards.as_ref(), Utc::now())
    }

    pub async fn get_rewards(&self, wallet_id: WalletId) -> Result<Rewards, GemServiceError> {
        Ok(self.api.client.get_rewards(wallet_id.id()).await.map_err(GemApiError::from)?)
    }

    pub async fn create_referral(&self, wallet: Wallet, code: String) -> Result<Rewards, GemServiceError> {
        let wallet_id = wallet.id.id();
        let request = AuthenticatedRequest {
            auth: self.auth.get_auth_payload(wallet).await?,
            data: ReferralCode { code },
        };
        Ok(self.api.client.create_referral(wallet_id, request).await.map_err(GemApiError::from)?)
    }

    pub async fn use_referral_code(&self, wallet: Wallet, code: String) -> Result<(), GemServiceError> {
        let wallet_id = wallet.id.id();
        let request = AuthenticatedRequest {
            auth: self.auth.get_auth_payload(wallet).await?,
            data: ReferralCode { code },
        };
        self.api.client.use_referral_code(wallet_id, request).await.map_err(GemApiError::from)?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use futures::executor::block_on;
    use num_bigint::BigUint;
    use primitives::rewards::{RedemptionStatus, RewardRedemption, RewardRedemptionOption, RewardRedemptionType};
    use primitives::{Asset, Chain};

    use super::testkit::RewardsTestkit;
    use super::*;
    use crate::testkit::TestAlienProvider;

    const NONCE: &str = r#"{"nonce":"nonce-1","timestamp":1}"#;

    fn redemption(asset: Option<Asset>) -> RedemptionResult {
        RedemptionResult {
            redemption: RewardRedemption {
                id: 7,
                option: RewardRedemptionOption {
                    id: "option-1".to_string(),
                    redemption_type: RewardRedemptionType::Asset,
                    points: 100,
                    asset,
                    value: BigUint::from(1u32),
                    remaining: None,
                },
                status: RedemptionStatus::Completed,
                transaction_id: None,
                created_at: Utc.timestamp_opt(0, 0).unwrap(),
            },
        }
    }

    fn provider(result: &RedemptionResult) -> Arc<TestAlienProvider> {
        Arc::new(TestAlienProvider::with_json_by_path(
            200,
            &[("auth/nonce", NONCE), ("rewards/redeem", &serde_json::to_string(result).unwrap())],
        ))
    }

    #[test]
    fn test_redeeming_an_asset_enables_its_balance() {
        block_on(async {
            let asset = Asset::from_chain(Chain::Ethereum);
            let result = redemption(Some(asset.clone()));
            let testkit = RewardsTestkit::with_provider(provider(&result)).await;

            let redeemed = testkit.service.redeem(testkit.wallet.clone(), "option-1".to_string()).await.unwrap();

            assert_eq!(redeemed.redemption.id, 7);
            let writes = testkit.balances.enable_writes.lock().unwrap();
            assert_eq!(*writes, vec![(vec![asset.id.clone()], true)]);
        })
    }

    #[test]
    fn test_redeeming_points_touches_no_balance() {
        block_on(async {
            let result = redemption(None);
            let testkit = RewardsTestkit::with_provider(provider(&result)).await;

            testkit.service.redeem(testkit.wallet.clone(), "option-1".to_string()).await.unwrap();

            assert!(testkit.balances.enable_writes.lock().unwrap().is_empty());
        })
    }

    #[test]
    fn test_a_redemption_that_fails_enables_nothing() {
        block_on(async {
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(
                200,
                &[("auth/nonce", NONCE), ("rewards/redeem", "not json")],
            )))
            .await;

            assert!(testkit.service.redeem(testkit.wallet.clone(), "option-1".to_string()).await.is_err());
            assert!(testkit.balances.enable_writes.lock().unwrap().is_empty());
        })
    }

    #[test]
    fn test_a_referral_call_signs_with_the_wallet_before_it_reaches_the_api() {
        block_on(async {
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(200, &[("auth/nonce", NONCE), ("referrals/use", "true")]))).await;

            testkit.service.use_referral_code(testkit.wallet.clone(), "code".to_string()).await.unwrap();

            let paths = testkit.provider.requested_paths();
            let nonce = paths.iter().position(|path| path.contains("auth/nonce"));
            let referral = paths.iter().position(|path| path.contains("referrals/use"));
            assert!(nonce < referral, "the referral call did not wait for the nonce: {paths:?}");
        })
    }

    #[test]
    fn test_a_wallet_with_no_auth_account_never_reaches_the_api() {
        block_on(async {
            let testkit = RewardsTestkit::with_provider(Arc::new(TestAlienProvider::with_json_by_path(200, &[("auth/nonce", NONCE)]))).await;
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
