use std::sync::Arc;

use axum::extract::State;
use primitives::rewards::{RedemptionRequest, RedemptionResult, RewardRedemptionOption};
use primitives::{ReferralCode, RewardEvent, Rewards, WalletId};
use services::rewards::{RewardsClient, RewardsRedemptionClient};

use crate::auth::device::{AuthenticatedDevice, AuthenticatedDeviceWallet};
use crate::auth::wallet_signed::WalletSigned;
use crate::error::ApiError;
use crate::request::{ClientIp, Path, UserAgent};
use crate::response::ApiResponse;

pub async fn get_rewards(device: AuthenticatedDeviceWallet, State(client): State<Arc<RewardsClient>>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.get_rewards_by_wallet_id(&device.record, device.wallet_id, device.record.device.locale.as_ref()).await?.into())
}

pub async fn get_rewards_events(device: AuthenticatedDeviceWallet, State(client): State<Arc<RewardsClient>>) -> Result<ApiResponse<Vec<RewardEvent>>, ApiError> {
    Ok(client.get_rewards_events_by_wallet_id(device.wallet_id).await?.into())
}

pub async fn get_redemption_option(_device: AuthenticatedDevice, Path(code): Path<String>, State(client): State<Arc<RewardsClient>>) -> Result<ApiResponse<RewardRedemptionOption>, ApiError> {
    Ok(client.get_rewards_redemption_option(&code).await?.into())
}

pub async fn create_referral(device: AuthenticatedDevice, ClientIp(ip): ClientIp, State(client): State<Arc<RewardsClient>>, request: WalletSigned<ReferralCode>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client
        .create_username(&request.address, &request.data.code, device.record.id, &ip.to_string(), device.record.device.locale.as_ref())
        .await?
        .into())
}

pub async fn use_referral_code(device: AuthenticatedDevice, ClientIp(ip): ClientIp, user_agent: UserAgent, State(client): State<Arc<RewardsClient>>, request: WalletSigned<ReferralCode>) -> Result<ApiResponse<bool>, ApiError> {
    client.use_referral_code(&device.record, &request.address, &request.data.code, &ip.to_string(), &user_agent.0).await?;
    Ok(true.into())
}

pub async fn redeem(device: AuthenticatedDeviceWallet, State(client): State<Arc<RewardsRedemptionClient>>, request: WalletSigned<RedemptionRequest>) -> Result<ApiResponse<RedemptionResult>, ApiError> {
    if WalletId::Multicoin(request.address.clone()) != device.wallet_identifier {
        return Err(ApiError::bad_request("Wallet signature mismatch"));
    }
    Ok(client.redeem_by_wallet_id(device.wallet_id, &request.data.id, device.record.id, device.record.device.locale.as_ref()).await?.into())
}
