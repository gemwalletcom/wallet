use std::sync::Arc;

use primitives::rewards::{RedemptionRequest, RedemptionResult};
use primitives::{AuthPayload, AuthenticatedRequest, ReferralCode, Rewards, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::config::rewards::get_referral_url;

#[derive(Debug, uniffi::Object)]
pub struct GemRewardsService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemRewardsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub fn referral_link(&self, code: String) -> String {
        get_referral_url(&code)
    }

    pub async fn get_rewards(&self, wallet_id: WalletId) -> Result<Rewards, GemApiError> {
        Ok(self.api.client.get_rewards(wallet_id.id()).await?)
    }

    pub async fn create_referral(&self, wallet_id: WalletId, auth: AuthPayload, code: String) -> Result<Rewards, GemApiError> {
        let request = AuthenticatedRequest {
            auth,
            data: ReferralCode { code },
        };
        Ok(self.api.client.create_referral(wallet_id.id(), request).await?)
    }

    pub async fn use_referral_code(&self, wallet_id: WalletId, auth: AuthPayload, code: String) -> Result<(), GemApiError> {
        let request = AuthenticatedRequest {
            auth,
            data: ReferralCode { code },
        };
        self.api.client.use_referral_code(wallet_id.id(), request).await?;
        Ok(())
    }

    pub async fn redeem(&self, wallet_id: WalletId, auth: AuthPayload, redemption_id: String) -> Result<RedemptionResult, GemApiError> {
        let request = AuthenticatedRequest {
            auth,
            data: RedemptionRequest { id: redemption_id },
        };
        Ok(self.api.client.redeem_rewards(wallet_id.id(), request).await?)
    }
}
