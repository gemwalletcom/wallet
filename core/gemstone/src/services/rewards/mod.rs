use std::sync::Arc;

use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::{AuthenticatedRequest, ReferralCode, Rewards, Wallet, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::config::rewards::get_referral_url;
use crate::services::auth::GemAuthService;
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::wallet_session::{GemWalletSessionService, rules as session_rules};

#[derive(uniffi::Object)]
pub struct GemRewardsService {
    api: Arc<GemDeviceApiClient>,
    auth: Arc<GemAuthService>,
    balance: Arc<GemBalanceService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemRewardsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, auth: Arc<GemAuthService>, balance: Arc<GemBalanceService>, session: Arc<GemWalletSessionService>) -> Self {
        Self { api, auth, balance, session }
    }

    pub fn wallets(&self, wallets: Vec<Wallet>) -> Vec<Wallet> {
        session_rules::rewards_wallets(wallets)
    }

    pub fn selected_wallet(&self, current: Option<Wallet>, wallets: Vec<Wallet>) -> Option<Wallet> {
        session_rules::rewards_wallet(current, &self.wallets(wallets))
    }

    pub fn referral_link(&self, code: String) -> String {
        get_referral_url(&code)
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
